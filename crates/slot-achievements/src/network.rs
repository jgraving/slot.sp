use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use zeroize::Zeroize;

use crate::storage::{self, Auth, Config, Game, Store, Unlock};
use crate::{Notice, Status, SyncStatus};

const API: &str = "https://retroachievements.org/dorequest.php";
const VERSION: &str = "12.2.1";

pub(crate) struct Load {
    pub generation: u64,
    pub path: PathBuf,
}

pub(crate) struct Prepared {
    pub generation: u64,
    pub hash: String,
    pub game: Game,
    pub unlocked: BTreeSet<u32>,
    pub cached: bool,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Failure {
    Network,
    Rejected,
    Authentication,
    Invalid,
}

pub(crate) trait Transport {
    fn network_problem(&self)->&'static str {"RA server unavailable"}
    fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, Failure>;
    fn badge(&mut self, _name: &str) -> Result<Vec<u8>, Failure> {
        Err(Failure::Invalid)
    }
}

/// Dropping the account cannot retract an in-flight HTTPS request. Reject its
/// result and gate every subsequent request, including nested prefetch/artwork
/// calls, so signing out never starts more work with the retired credentials.
struct WhileActive<T> {
    inner: T,
    status: Arc<Status>,
}
impl<T: Transport> Transport for WhileActive<T> {
    fn network_problem(&self)->&'static str {self.inner.network_problem()}
    fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, Failure> {
        if self.status.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Failure::Network);
        }
        let mut response = self.inner.call(fields);
        if self.status.stopped.load(std::sync::atomic::Ordering::Acquire) {
            if let Ok(value) = &mut response { wipe_strings(value); }
            return Err(Failure::Network);
        }
        response
    }
    fn badge(&mut self, name: &str) -> Result<Vec<u8>, Failure> {
        if self.status.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Failure::Network);
        }
        let response = self.inner.badge(name);
        if self.status.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Failure::Network);
        }
        response
    }
}
fn wipe_strings(value: &mut Value) {
    match value {
        Value::String(text) => text.zeroize(),
        Value::Array(values) => values.iter_mut().for_each(wipe_strings),
        Value::Object(values) => values.values_mut().for_each(wipe_strings),
        _ => {},
    }
}
/// Clear our password/token copies after transport has consumed the form, also
/// on errors. TLS/HTTP implementations may own other transient copies.
fn send_sensitive(http: &mut impl Transport, mut fields: Vec<(&str, String)>) -> Result<Value, Failure> {
    let result = http.call(&fields);
    for (_, value) in &mut fields { value.zeroize(); }
    result
}

pub(crate) struct Http(ureq::Agent, &'static str);
fn transport_problem(error:&ureq::Error)->&'static str {
    match error {
        ureq::Error::HostNotFound=>"DNS lookup failed",
        ureq::Error::Timeout(_)=>"Server request timed out",
        ureq::Error::Tls(_)|ureq::Error::Rustls(_)=>"TLS failed · check date/time",
        ureq::Error::StatusCode(429)=>"Server rate limit · retrying",
        ureq::Error::StatusCode(500..=599)=>"Server temporarily unavailable",
        ureq::Error::StatusCode(_)=>"Server refused the request",
        _=>"Network connection failed",
    }
}

impl Http {
    pub(crate) fn new() -> Self {
        Self(
            ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(15)))
                .max_redirects(0)
                .user_agent(concat!(
                    "slot/",
                    env!("CARGO_PKG_VERSION"),
                    " rcheevos/12.2.1"
                ))
                .build()
                .into(),
            "Network connection failed",
        )
    }
}

impl Transport for Http {
    fn network_problem(&self)->&'static str {self.1}
    fn badge(&mut self, name: &str) -> Result<Vec<u8>, Failure> {
        crate::badges::path(std::path::Path::new(""), name).ok_or(Failure::Invalid)?;
        self.0
            .get(format!(
                "https://media.retroachievements.org/Badge/{name}.png"
            ))
            .call()
            .map_err(|_| Failure::Network)?
            .body_mut()
            .with_config()
            .limit(crate::badges::MAX_BYTES)
            .read_to_vec()
            .map_err(|_| Failure::Invalid)
    }
    fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, Failure> {
        let form: Vec<_> = fields.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let mut response = self
            .0
            .post(API)
            .send_form(form)
            .map_err(|error| {self.1=transport_problem(&error);Failure::Network})?;
        response
            .body_mut()
            .with_config()
            .limit(8 * 1024 * 1024)
            .read_json()
            .map_err(|_| Failure::Invalid)
    }
}

fn success(value: Value) -> Result<Value, Failure> {
    if value.get("Success") == Some(&Value::Bool(true)) {
        Ok(value)
    } else if matches!(value.get("Code").and_then(Value::as_str),Some("expired_token"|"invalid_credentials"|"access_denied")) {
        Err(Failure::Authentication)
    } else {
        Err(Failure::Rejected)
    }
}

fn call(
    http: &mut impl Transport,
    auth: &Auth,
    request: &str,
    extra: &[(&str, String)],
) -> Result<Value, Failure> {
    let mut fields = vec![
        ("r", request.into()),
        ("u", auth.username.clone()),
        ("t", auth.token.clone()),
    ];
    fields.extend_from_slice(extra);
    success(send_sensitive(http, fields)?)
}

fn login(
    http: &mut impl Transport,
    config: &Config,
    remembered: Option<&Auth>,
) -> Result<Auth, Failure> {
    let mut fields = vec![("r", "login2".into()), ("u", config.username.clone())];
    if !config.token.is_empty() {
        fields.push(("t", config.token.clone()));
    } else if let Some(auth) = remembered {
        fields.push(("t", auth.token.clone()));
    } else {
        fields.push(("p", config.password.clone()));
    }
    let mut response = match send_sensitive(http, fields).and_then(success) {
        Err(Failure::Rejected|Failure::Authentication) if config.token.is_empty() && !config.password.is_empty() => {
            // A remembered token can expire. An explicitly supplied password can renew it.
            success(send_sensitive(http, vec![
                ("r", "login2".into()),
                ("u", config.username.clone()),
                ("p", config.password.clone()),
            ])?)?
        }
        result => result?,
    };
    let username = response["User"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Failure::Invalid)?;
    let username = username.to_owned();
    let Value::String(token) = response["Token"].take() else { return Err(Failure::Invalid); };
    if token.is_empty() { return Err(Failure::Invalid); }
    Ok(Auth { username, token })
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Matches rc_api_init_award_achievement_request, including the original unlock age.
pub(crate) fn award(
    http: &mut impl Transport,
    auth: &Auth,
    unlock: &Unlock,
    now: u64,
) -> Result<(), Failure> {
    let age = now
        .saturating_sub(unlock.earned_at)
        .min(u64::from(u32::MAX));
    let mut signature = format!("{}{}0", unlock.id, auth.username);
    if age > 0 {
        signature.push_str(&format!("{}{age}", unlock.id));
    }
    let mut extra = vec![
        ("a", unlock.id.to_string()),
        ("h", "0".into()),
        ("m", unlock.hash.clone()),
        ("v", format!("{:x}", md5::compute(signature))),
    ];
    if age > 0 {
        extra.push(("o", age.to_string()));
    }
    let mut fields = vec![
        ("r", "awardachievement".into()),
        ("u", auth.username.clone()),
        ("t", auth.token.clone()),
    ];
    fields.extend(extra);
    let response = send_sensitive(http, fields)?;
    // A retry after a lost response may say the server already has this unlock.
    if response["Success"] == true
        || response["Error"]
            .as_str()
            .is_some_and(|s| s.starts_with("User already has"))
    {
        Ok(())
    } else {success(response).map(|_|())}
}

fn unlocked(response: &Value) -> Result<BTreeSet<u32>, Failure> {
    let mut ids = BTreeSet::new();
    for field in ["Unlocks", "HardcoreUnlocks"] {
        // Like rc_api_process_start_session_response, accept omitted empty lists.
        if response[field].is_null() {
            continue;
        }
        let entries = response[field].as_array().ok_or(Failure::Invalid)?;
        for entry in entries {
            let id = entry["ID"]
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or(Failure::Invalid)?;
            ids.insert(id);
        }
    }
    Ok(ids)
}

struct Active {
    generation: u64,
    hash: String,
    ready: bool,
    online: bool,
    last_ping: Instant,
    retry_at: Instant,
    retry_delay: u64,
    status: &'static str,
}

fn prefetch(
    http: &mut impl Transport,
    auth: &Auth,
    dir: &std::path::Path,
    hash: &str,
) -> Result<Game, Failure> {
    let resolved = call(http, auth, "gameid", &[("m", hash.into())])?;
    let id = resolved["GameID"].as_u64().ok_or(Failure::Invalid)?;
    let (game, ids) = if id == 0 {
        (
            Game {
                presence: String::new(),
                id: 0,
                console: 5,
                title: "No achievements for this ROM".into(),
                achievements: vec![],
            },
            BTreeSet::new(),
        )
    } else {
        let patch = call(http, auth, "patch", &[("g", id.to_string())])?;
        let game: Game =
            serde_json::from_value(patch["PatchData"].clone()).map_err(|_| Failure::Invalid)?;
        if !crate::tracked_console(game.console) || u64::from(game.id) != id {
            return Err(Failure::Invalid);
        }
        let unlocks = call(
            http,
            auth,
            "unlocks",
            &[("g", id.to_string()), ("h", "0".into())],
        )?;
        let ids: BTreeSet<u32> =
            serde_json::from_value(unlocks["UserUnlocks"].clone()).map_err(|_| Failure::Invalid)?;
        (game, ids)
    };
    storage::write(&dir.join(format!("{hash}.json")), &(&game, ids))
        .map_err(|_| Failure::Invalid)?;
    Ok(game)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    root: PathBuf,
    mut config: Config,
    store: Arc<Mutex<Store>>,
    loads: mpsc::Receiver<Option<Load>>,
    prepared: mpsc::Sender<Prepared>,
    notices: mpsc::Sender<Notice>,
    status: Arc<Status>,
    http: impl Transport,
) {
    let mut http = WhileActive { inner: http, status: status.clone() };
    let dir = store.lock().unwrap().dir.clone();
    let mut auth: Option<Auth> = None;
    let mut verified = false;
    let mut active: Option<Active> = None;
    let mut next_attempt = Instant::now();
    let mut delay = 30;
    let mut warned = false;
    let mut last_award = 0;
    let (mut library_total,mut library) = crate::library::scan(&root, &dir, now());
    let mut library_failed = 0usize;
    let mut problem = "";
    let mut rescanned = Instant::now();
    let mut badges = crate::badges::Queue::new(&dir);
    loop {
        if status.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        match loads.recv_timeout(Duration::from_millis(200)) {
            Ok(mut load) => {
                while let Ok(newer) = loads.try_recv() {
                    load = newer;
                }
                active = load.and_then(|load| {
                    // GBA identification is the MD5 of the complete uncompressed ROM.
                    // File I/O and hashing happen here, never on the emulator/UI threads.
                    let hash = crate::library::hash(&dir, &load.path).ok()?;
                    let mut game = Active {
                        generation: load.generation,
                        hash,
                        ready: false,
                        online: false,
                        last_ping: Instant::now(),
                        retry_at: Instant::now(),
                        retry_delay: 30,
                        status: "Checking…",
                    };
                    if auth.is_some() {
                        let cache = dir.join(format!("{}.json", game.hash));
                        if let Ok((data, ids)) = storage::read::<(Game, BTreeSet<u32>)>(&cache) {
                            if crate::tracked_console(data.console) {
                                badges.enqueue(&data);
                                let _ = prepared.send(Prepared {
                                    generation: game.generation,
                                    hash: game.hash.clone(),
                                    game: data,
                                    unlocked: ids,
                                    cached: true,
                                });
                                game.ready = true;
                                game.status = "Cached";
                            }
                        }
                    }
                    Some(game)
                });
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        badges.maintain();
        // Publish independent game and artwork phases before attempting HTTP.
        status.pending.store(
            !verified || !library.is_empty()
                || library_failed > 0
                || badges.pending()
                || store.lock().unwrap().unlocks.values().any(|u| !u.synced),
            std::sync::atomic::Ordering::Release,
        );
        publish_detail(&status, verified, library_total, library.len(), library_failed,
            &badges, &store, next_attempt, problem, active.as_ref());
        if status
            .reconnect
            .swap(false, std::sync::atomic::Ordering::AcqRel)
        {
            next_attempt = Instant::now();
            badges.network_available();
            if let Some(game) = active.as_mut() {
                game.retry_at = Instant::now();
                game.retry_delay = 30;
            }
        }
        if Instant::now() < next_attempt {
            continue;
        }
        if should_rescan(rescanned.elapsed(), library.len()) {
            let (total,pending)=crate::library::scan(&root,&dir,now());
            library_total=total;
            if !pending.is_empty() {library=pending;library_failed=0;}
            badges.scan();
            rescanned = Instant::now();
        }
        if !verified
            || !library.is_empty()
            || (badges.pending() && !badges.waiting)
            || store.lock().unwrap().unlocks.values().any(|u| !u.synced)
        {
            status.set(SyncStatus::Syncing);
        }
        publish_detail(&status, verified, library_total, library.len(), library_failed,
            &badges, &store, next_attempt, problem, active.as_ref());
        let result = (|| -> Result<(), Failure> {
            if !verified {
                let logged_in = login(&mut http, &config, auth.as_ref()).map_err(|e| {
                    if e == Failure::Rejected {
                        Failure::Authentication
                    } else {
                        e
                    }
                })?;
                {
                    let mut pending = status.auth.lock().unwrap();
                    if status.stopped.load(std::sync::atomic::Ordering::Acquire) {
                        return Err(Failure::Network);
                    }
                    *pending = Some(logged_in.clone());
                }
                {
                    use zeroize::Zeroize;
                    config.password.zeroize();
                }
                auth = Some(logged_in);
                verified = true;
            }
            let auth = auth.as_ref().ok_or(Failure::Invalid)?;
            // Rotate through pending awards so one rejected/deleted achievement does not
            // prevent other games from syncing or the library from being prepared.
            let mut deferred_error = None;
            let pending = {
                let store = store.lock().unwrap();
                store
                    .unlocks
                    .range((
                        std::ops::Bound::Excluded(last_award),
                        std::ops::Bound::Unbounded,
                    ))
                    .map(|(_, u)| u)
                    .find(|u| !u.synced)
                    .or_else(|| store.unlocks.values().find(|u| !u.synced))
                    .cloned()
            };
            if let Some(unlock) = pending {
                last_award = unlock.id;
                match award(&mut http, auth, &unlock, now()) {
                    Ok(()) => {
                        let mut store = store.lock().unwrap();
                        store.ack(unlock.id).map_err(|_| Failure::Invalid)?;
                        if store.unlocks.values().all(|u| u.synced) {
                            let _ = notices.send(Notice::status(0, "Unlocks uploaded"));
                        }
                    }
                    Err(Failure::Network) => return Err(Failure::Network),
                    Err(error) => deferred_error = Some(error),
                }
            }
            if let Some(game) = active.as_mut().filter(|game| Instant::now() >= game.retry_at) {
                let game_result = (|| -> Result<(), Failure> {
                    if !game.online {
                        let resolved = call(&mut http, auth, "gameid", &[("m", game.hash.clone())])?;
                        let id = resolved["GameID"].as_u64().ok_or(Failure::Invalid)?;
                        if id == 0 {
                            game.online = true;
                            game.ready = false;
                            game.status = "No achievements";
                        } else {
                            let patch = call(&mut http, auth, "patch", &[("g", id.to_string())])?;
                            let data: Game = serde_json::from_value(patch["PatchData"].clone())
                                .map_err(|_| Failure::Invalid)?;
                            if !crate::tracked_console(data.console) || u64::from(data.id) != id {
                                return Err(Failure::Invalid);
                            }
                            if data.title.starts_with("Unsupported Game Version") {
                                game.online = true;
                                game.ready = false;
                                game.status = "Version unsupported";
                                return Ok(());
                            }
                            let session = call(
                                &mut http,
                                auth,
                                "startsession",
                                &[
                                    ("g", id.to_string()),
                                    ("h", "0".into()),
                                    ("m", game.hash.clone()),
                                    ("l", VERSION.into()),
                                ],
                            )?;
                            let ids = unlocked(&session)?;
                            storage::write(&dir.join(format!("{}.json", game.hash)), &(&data, &ids))
                                .map_err(|_| Failure::Invalid)?;
                            badges.enqueue(&data);
                            ping(&mut http, auth, &status, game, &data)?;
                            let _ = prepared.send(Prepared {
                                generation: game.generation,
                                hash: game.hash.clone(),
                                game: data,
                                unlocked: ids,
                                cached: false,
                            });
                            game.ready = true;
                            game.online = true;
                            game.status = "Ready";
                            game.last_ping = Instant::now();
                        }
                    } else if game.ready && game.last_ping.elapsed() >= Duration::from_secs(120) {
                        let cached: (Game, BTreeSet<u32>) =
                            storage::read(&dir.join(format!("{}.json", game.hash)))
                                .map_err(|_| Failure::Invalid)?;
                        ping(&mut http, auth, &status, game, &cached.0)?;
                        game.status = "Ready";
                        game.last_ping = Instant::now();
                    }
                    Ok(())
                })();
                match game_result {
                    Ok(()) => game.retry_delay = 30,
                    Err(Failure::Authentication) => return Err(Failure::Authentication),
                    Err(_) => {
                        game.status = if game.ready { "Cached · retrying" } else { "Retrying" };
                        game.retry_at = Instant::now() + Duration::from_secs(game.retry_delay);
                        game.retry_delay = (game.retry_delay * 2).min(300);
                    }
                }
            }
            // One ROM per pass, with a one-second gap. Prefetch uses `unlocks`, not
            // `startsession`, so preparing a library doesn't claim the user played it.
            if let Some(path) = library.front() {
                if let Ok(hash) = crate::library::hash(&dir, path) {
                    match prefetch(&mut http, auth, &dir, &hash) {
                        Ok(game) => {
                            crate::library::checked(&dir, path, now());
                            badges.enqueue(&game);
                        }
                        Err(Failure::Network) => return Err(Failure::Network),
                        Err(error) => {
                            library_failed += 1;
                            deferred_error = Some(error);
                        }
                    }
                } else {
                    library_failed += 1;
                }
                // Removed/unreadable ROMs are reconsidered on the next library scan.
                library.pop_front();
                if library.is_empty() && library_failed == 0 && !badges.pending() {
                    let _ = notices.send(Notice::status(0, "Offline achievement cache ready"));
                }
            }
            badges.step(&mut http);
            deferred_error.map_or(Ok(()), Err)
        })();
        if status.stopped.load(std::sync::atomic::Ordering::Acquire) { return; }
        status.pending.store(
            !verified || !library.is_empty()
                || library_failed > 0
                || badges.pending()
                || store.lock().unwrap().unlocks.values().any(|u| !u.synced),
            std::sync::atomic::Ordering::Release,
        );
        publish_detail(&status, verified, library_total, library.len(), library_failed,
            &badges, &store, next_attempt, problem, active.as_ref());
        match result {
            Ok(()) => {
                problem = if library_failed > 0 { "Some games could not be prepared" }
                    else if badges.waiting { badges.problem } else { "" };
                status.set(if library_failed > 0 || badges.waiting {
                    SyncStatus::Attention
                } else if !library.is_empty()
                    || badges.pending()
                    || store.lock().unwrap().unlocks.values().any(|u| !u.synced)
                {
                    SyncStatus::Syncing
                } else {
                    SyncStatus::Ready
                });
                delay = 30;
                warned = false;
                next_attempt = Instant::now() + Duration::from_secs(3);
            }
            Err(error) => {
                problem = match error { Failure::Network => http.network_problem(),
                    Failure::Authentication => "Check username or password",
                    Failure::Rejected => "Server rejected a request",
                    Failure::Invalid => "Game data or storage error" };
                status.set(if error == Failure::Network {
                    SyncStatus::Offline
                } else {
                    SyncStatus::Attention
                });
                // Reauthenticate after errors. No rejected or malformed response consumes an award.
                if error==Failure::Authentication {verified=false;status.pending.store(true,std::sync::atomic::Ordering::Release);}
                let quiet_offline = error == Failure::Network
                    && (active.as_ref().is_some_and(|g| g.ready)
                        || (active.is_none() && auth.is_some()));
                if !warned && !quiet_offline {
                    let generation = active.as_ref().map_or(0, |g| g.generation);
                    let title = match error {
                        Failure::Network if active.as_ref().is_some_and(|g| g.ready) => {
                            "Achievements offline - sync later"
                        }
                        Failure::Network => "Achievements need internet once",
                        Failure::Rejected => "Achievement sync needs attention",
                        Failure::Authentication => "Achievements: check account config",
                        Failure::Invalid => "Achievements: data or storage error",
                    };
                    let _ = notices.send(Notice::status(generation, title));
                    warned = true;
                }
                next_attempt = Instant::now() + Duration::from_secs(delay);
                delay = (delay * 2).min(300);
            }
        }
        publish_detail(&status, verified, library_total, library.len(), library_failed,
            &badges, &store, next_attempt, problem, active.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn authentication_expiry_is_distinct_from_retryable_request_failure() {
        for code in ["expired_token","invalid_credentials","access_denied"] {assert_eq!(success(serde_json::json!({"Success":false,"Code":code})),Err(Failure::Authentication));}
        assert_eq!(success(serde_json::json!({"Success":false,"Code":"not_found"})),Err(Failure::Rejected));
    }
    #[test]
    #[ignore = "manual trusted-branch live-account smoke test"]
    fn live_ra_read_only_smoke() {
        use zeroize::Zeroize;
        let username=std::env::var("RA_TEST_USERNAME").expect("RA_TEST_USERNAME is required");
        let password=std::env::var("RA_TEST_PASSWORD").expect("RA_TEST_PASSWORD is required");
        assert!(!username.is_empty() && !password.is_empty(),"RA test secrets are missing");
        let mut config=Config{enabled:true,username,password,token:String::new()};
        let mut http=Http::new();
        let result=login(&mut http,&config,None);config.password.zeroize();
        let Ok(mut auth)=result else {panic!("RA login failed (response and credentials withheld)");};
        println!("RA login passed");
        // Lookup only: no startsession, ping, or awardachievement requests.
        let result=call(&mut http,&auth,"gameid",&[("m","00000000000000000000000000000000".into())]);
        auth.token.zeroize();assert!(result.is_ok(),"RA game lookup failed (response withheld)");
        println!("RA read-only game lookup passed");
        let result=http.badge("25000");assert!(result.as_ref().ok().is_some_and(|b|crate::badges::decode(b).is_some()),"RA artwork download/decode failed");
        println!("RA artwork download/decode passed");
    }
    #[test]
    fn empty_session_unlock_lists_may_be_omitted() {
        assert!(unlocked(&json!({"Success":true})).unwrap().is_empty());
        assert_eq!(
            unlocked(&json!({"Unlocks":[{"ID":7,"When":123}]})).unwrap(),
            [7].into()
        );
        assert!(unlocked(&json!({"Unlocks":"invalid"})).is_err());
    }
    struct Fake {
        reply: Value,
        fields: Vec<(String, String)>,
    }
    impl Transport for Fake {
        fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, Failure> {
            self.fields = fields
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();
            Ok(self.reply.clone())
        }
    }
    #[test]
    fn delayed_awards_use_original_time_and_softcore_signature() {
        let mut http = Fake {
            reply: json!({"Success":true}),
            fields: vec![],
        };
        let auth = Auth {
            username: "Player".into(),
            token: "secret".into(),
        };
        let unlock = Unlock {
            id: 123,
            hash: "abcdef".into(),
            earned_at: 1000,
            synced: false,
        };
        award(&mut http, &auth, &unlock, 1060).unwrap();
        let fields: std::collections::BTreeMap<_, _> = http.fields.into_iter().collect();
        assert_eq!(fields["o"], "60");
        assert_eq!(fields["h"], "0");
        assert_eq!(
            fields["v"],
            format!("{:x}", md5::compute("123Player012360"))
        );
    }
    #[test]
    fn an_http_success_is_not_an_award_acknowledgment() {
        let auth = Auth {
            username: "Player".into(),
            token: "secret".into(),
        };
        let unlock = Unlock {
            id: 1,
            hash: "a".into(),
            earned_at: 100,
            synced: false,
        };
        for reply in [json!({"Success":false,"Error":"Expired token"}), json!({})] {
            let mut http = Fake {
                reply,
                fields: vec![],
            };
            assert_eq!(award(&mut http, &auth, &unlock, 50), Err(Failure::Rejected));
            assert!(!http.fields.iter().any(|(k, _)| k == "o"));
        }
    }
}

fn ping(
    http: &mut impl Transport,
    auth: &Auth,
    status: &Status,
    active: &Active,
    game: &Game,
) -> Result<(), Failure> {
    let presence = status
        .presence
        .lock()
        .unwrap()
        .as_ref()
        .filter(|(generation, text)| *generation == active.generation && !text.is_empty())
        .map(|(_, text)| text.clone())
        .unwrap_or_else(|| format!("Playing {}", game.title));
    call(
        http,
        auth,
        "ping",
        &[
            ("g", game.id.to_string()),
            ("h", "0".into()),
            ("x", active.hash.clone()),
            ("m", presence),
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod presence_tests {
    use super::*;
    #[test]
    fn heartbeat_uses_current_generation_and_separate_hash_field() {
        struct Capture(Vec<(String, String)>);
        impl Transport for Capture {
            fn call(&mut self, fields: &[(&str, String)]) -> Result<Value, Failure> {
                self.0 = fields
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                Ok(serde_json::json!({"Success":true}))
            }
        }
        let mut http = Capture(vec![]);
        let auth = Auth {
            username: "test".into(),
            token: "fixture".into(),
        };
        let status = Status::default();
        let active = Active {
            generation: 2,
            hash: "rom-hash".into(),
            ready: true,
            online: true,
            last_ping: Instant::now(),
            retry_at: Instant::now(),
            retry_delay: 30,
            status: "Ready",
        };
        let game = Game {
            id: 1,
            console: 5,
            title: "Test".into(),
            presence: String::new(),
            achievements: vec![],
        };
        for (generation, text, expected) in [
            (1, "Old game", "Playing Test"),
            (2, "Level 3", "Level 3"),
            (2, "", "Playing Test"),
        ] {
            *status.presence.lock().unwrap() = Some((generation, text.into()));
            ping(&mut http, &auth, &status, &active, &game).unwrap();
            assert!(http.0.contains(&("m".into(), expected.into())));
            assert!(http.0.contains(&("x".into(), "rom-hash".into())));
        }
    }
}

// Never discard the remaining work and reset its denominator during an active pass.
fn should_rescan(elapsed: Duration, remaining: usize) -> bool {
    remaining == 0 && elapsed >= Duration::from_secs(300)
}

#[allow(clippy::too_many_arguments)]
fn publish_detail(status: &Status, authenticated: bool, total: usize, remaining: usize,
    failed: usize, badges: &crate::badges::Queue, store: &Arc<Mutex<Store>>,
    next_attempt: Instant, problem: &'static str, active: Option<&Active>) {
    let (badges_done, badges_total) = badges.counts();
    let detail = crate::SyncDetail {
        status: status.get(), authenticated,catalog_ready:true,artwork_ready:!badges.discovery_pending(), games_done: total.saturating_sub(remaining),
        games_total: total, games_failed: failed, badges_done, badges_total,
        pending_awards: store.lock().unwrap().unlocks.values().filter(|u| !u.synced).count(),
        retry_seconds: if matches!(status.get(), SyncStatus::Offline | SyncStatus::Attention) {
            if badges.waiting && problem==badges.problem {badges.retry_seconds()}else{next_attempt.saturating_duration_since(Instant::now()).as_secs()}
        } else { 0 }, problem,
        game_status: active.map_or("", |game| game.status),
        game_retry_seconds: active.map_or(0, |game| {
            game.retry_at.saturating_duration_since(Instant::now()).as_secs()
        }),
    };
    *status.detail.lock().unwrap() = detail;
}

#[cfg(test)]
mod progress_tests {
    use super::*;
    #[test]
    fn a_large_library_keeps_its_active_pass_after_five_minutes() {
        assert!(!should_rescan(Duration::from_secs(900), 850));
        assert!(!should_rescan(Duration::from_secs(299), 0));
        assert!(should_rescan(Duration::from_secs(300), 0));
    }
    #[test]
    fn game_counts_do_not_depend_on_newly_discovered_artwork() {
        let mut d = crate::SyncDetail { games_done: 60, games_total: 900,
            badges_done: 2, badges_total: 3, ..Default::default() };
        assert_eq!(d.progress(), Some(("Game data", 60, 900)));
        d.badges_total = 9000;
        assert_eq!(d.progress(), Some(("Game data", 60, 900)));
        d.games_done = 900;
        assert_eq!(d.progress(), Some(("Artwork", 2, 9000)));
        d.badges_done = 9000;
        assert_eq!(d.progress(), None);
    }
}
