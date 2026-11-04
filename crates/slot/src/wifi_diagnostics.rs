//! Explicit, offline Wi-Fi reports. Never export raw command output or credentials.
use crate::wifi::{Busy, WifiView};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub type Export = Receiver<Result<String, &'static str>>;
const LIMIT: u64 = 64 * 1024;

pub fn start(root: PathBuf, view: WifiView, sharing:crate::sharing::View, ra:slot_achievements::SyncDetail) -> Export {
    let (tx, rx) = mpsc::channel();
    let _ = std::thread::Builder::new()
        .name("wifi-report".into())
        .spawn(move || {
            let mut report = collect(&root, &view);
            report.push_str(&format!("sharing_active={}\nsharing_busy={}\nsharing_error={}\nra_authenticated={}\nra_status={:?}\nra_retry_seconds={}\n",
                sharing.active,sharing.busy,sharing.error.is_some(),ra.authenticated,ra.status,ra.retry_seconds));
            // Only fixed categories leave the process, never server responses or account data.
            let reason=match ra.problem {
                "DNS lookup failed"=>"dns", "Server request timed out"=>"timeout",
                "TLS failed · check date/time"=>"tls", "Server rate limit · retrying"=>"rate-limit",
                "Check username or password"=>"authentication", ""=>"none", _=>"other",
            };
            report.push_str(&format!("ra_problem={reason}\n"));
            let _ =
                tx.send(save(&root, &report).map_err(|_| "Could not save report · check SD card"));
        });
    rx
}

struct Probe {
    result: &'static str,
    text: String,
}
// Read concurrently so a full pipe cannot freeze the worker. Each probe has a
// deadline and an output cap; stderr is discarded and never enters a report.
fn probe(program: &str, args: &[&str], timeout: Duration) -> Probe {
    let empty = |result| Probe {
        result,
        text: String::new(),
    };
    let Ok(mut child) = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return empty("unavailable");
    };
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return empty("failed");
    };
    let (tx, rx) = mpsc::channel();
    if std::thread::Builder::new()
        .name("wifi-probe".into())
        .spawn(move || {
            let mut bytes = Vec::new();
            let result = stdout.take(LIMIT + 1).read_to_end(&mut bytes);
            let _ = tx.send((result.is_ok(), bytes));
        })
        .is_err()
    {
        let _ = child.kill();
        let _ = child.wait();
        return empty("failed");
    }
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let Ok((read_ok, bytes)) =
                    rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                else {
                    return empty("timeout");
                };
                // Reaching the cap closes stdout and can make the producer exit
                // with SIGPIPE. Classify the cap before its resulting exit code.
                if bytes.len() as u64 > LIMIT {
                    return empty("too-large");
                }
                if !read_ok || !status.success() {
                    return empty("failed");
                }
                return Probe {
                    result: "ok",
                    text: String::from_utf8_lossy(&bytes).into_owned(),
                };
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return empty("failed");
            }
            _ => {}
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return empty("timeout");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        line.split_once('=')
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| v)
    })
}
fn state(text: &str) -> &'static str {
    match value(text, "wpa_state") {
        Some("COMPLETED") => "COMPLETED",
        Some("DISCONNECTED") => "DISCONNECTED",
        Some("SCANNING") => "SCANNING",
        Some("ASSOCIATING") => "ASSOCIATING",
        Some("ASSOCIATED") => "ASSOCIATED",
        Some("AUTHENTICATING") => "AUTHENTICATING",
        Some("4WAY_HANDSHAKE") => "4WAY_HANDSHAKE",
        Some("GROUP_HANDSHAKE") => "GROUP_HANDSHAKE",
        Some("INACTIVE") => "INACTIVE",
        Some("INTERFACE_DISABLED") => "INTERFACE_DISABLED",
        _ => "unknown",
    }
}
fn addresses(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "inet" {
                return None;
            }
            let address = parts.next()?.split('/').next()?;
            crate::wifi::usable_ipv4(address).then_some(address)
        })
        .collect()
}
fn append_probe(report: &mut String, name: &str, probe: &Probe) {
    report.push_str(&format!("{name}_probe={}\n", probe.result));
}
fn facts(view: &WifiView, status: &Probe, addr: &Probe, route: &Probe, helper: &Probe) -> String {
    let mut report = String::from("slot.sp Wi-Fi diagnostics v1\nPrivacy: names, addresses, passwords, tokens and raw logs omitted.\nRead-only local checks; no upload or Internet test.\n");
    report.push_str(&format!("launcher_radio_on={}\nlauncher_associated={}\nlauncher_ipv4={}\nlauncher_error_present={}\nlauncher_phase={}\n",
        view.on, view.connected.is_some(), view.ip.as_deref().is_some_and(crate::wifi::usable_ipv4), view.error.is_some(),
        match view.busy { Busy::Idle=>"idle",Busy::Starting=>"starting",Busy::Scanning=>"scanning",Busy::Joining(_)=>"joining",Busy::Addressing=>"getting-address" }));
    append_probe(&mut report, "supplicant", status);
    report.push_str(&format!("supplicant_state={}\n", state(&status.text)));
    let ip = value(&status.text, "ip_address").filter(|ip| crate::wifi::usable_ipv4(ip));
    report.push_str(&format!("supplicant_ipv4={}\n", ip.is_some()));
    append_probe(&mut report, "interface", addr);
    let ips = addresses(&addr.text);
    report.push_str(&format!(
        "interface_ipv4={}\nsupplicant_address_on_interface={}\n",
        !ips.is_empty(),
        ip.is_some_and(|ip| ips.contains(&ip))
    ));
    append_probe(&mut report, "route", route);
    let default_route = route.text.lines().any(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        fields.first() == Some(&"default") && fields.windows(2).any(|w| w == ["dev", "wlan0"])
    });
    report.push_str(&format!("wlan0_default_route={}\n", default_route));
    append_probe(&mut report, "helper", helper);
    // An allowlist of both keys AND values, never a redaction regex over raw text.
    for key in [
        "interface_present",
        "owner_present",
        "link_session",
        "watcher",
        "dhcp_client",
        "wpa_cli_available",
        "udhcpc_available",
        "ip_available",
        "callback_executable",
        "resolver_configured",
    ] {
        let safe = match value(&helper.text, key) {
            Some("yes") => "yes",
            Some("no") => "no",
            Some("alive") => "alive",
            Some("stale") => "stale",
            Some("missing") => "missing",
            _ => "unknown",
        };
        report.push_str(&format!("{key}={safe}\n"));
    }
    report
}
fn collect(root: &Path, view: &WifiView) -> String {
    let timeout = Duration::from_secs(3);
    let status = probe(
        "wpa_cli",
        &["-p", "/var/run/wpa_supplicant", "-i", "wlan0", "status"],
        timeout,
    );
    let addr = probe("ip", &["-4", "addr", "show", "dev", "wlan0"], timeout);
    let route = probe("ip", &["-4", "route", "show", "dev", "wlan0"], timeout);
    let helper_path = root.join("System/slotwifi.sh");
    let helper = probe(
        "/bin/sh",
        &[helper_path.to_str().unwrap_or(""), "diagnostics"],
        timeout,
    );
    let mut report = facts(view, &status, &addr, &route, &helper);
    let build = crate::build_info::Build::current();
    report.push_str(&format!(
        "build_commit={}\nbuild_date={}\n",
        build.hash, build.date
    ));
    for key in ["BASEOS_TARGET","BASEOS_VERSION"] {
        let value=crate::baseos::var(key).filter(|v|v.len()<=40 && v.bytes().all(|b|b.is_ascii_alphanumeric()||b"._-".contains(&b))).unwrap_or_else(||"unknown".into());
        report.push_str(&format!("{key}={value}\n"));
    }
    report.push_str(&format!("clock_plausible={}\n",!crate::update::clock_is_wrong(crate::update::now_unix())));
    for file in ["smbd","smbpasswd","testparm","samba-dcerpcd","rpcd_classic","rpcd_winreg"] {
        report.push_str(&format!("samba_{file}_present={}\n",root.join("System/samba/bin").join(file).is_file()));
    }
    let listening=view.ip.as_deref().and_then(|s|s.parse::<std::net::Ipv4Addr>().ok())
        .filter(|ip|crate::wifi::usable_ipv4(&ip.to_string()))
        .is_some_and(|ip|std::net::TcpStream::connect_timeout(&(ip,445).into(),Duration::from_millis(200)).is_ok());
    report.push_str(&format!("samba_local_listener={listening}\n"));
    report
}
fn save(root: &Path, report: &str) -> std::io::Result<String> {
    let dir = root.join("Diagnostics");
    match fs::create_dir(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    if !fs::symlink_metadata(&dir)?.file_type().is_dir() {
        return Err(std::io::Error::other("unsafe diagnostics directory"));
    }
    // A new file each time; never truncate an existing report or follow its symlink.
    for index in 1..=9999 {
        let name = format!("wifi-{index:04}.txt");
        let path = dir.join(&name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(mut file) => {
                if let Err(error) = file
                    .write_all(report.as_bytes())
                    .and_then(|_| file.sync_all())
                {
                    let _ = fs::remove_file(path);
                    return Err(error);
                }
                return Ok(format!("Diagnostics/{name}"));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other("too many reports"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ok(text: &str) -> Probe {
        Probe {
            result: "ok",
            text: text.into(),
        }
    }
    #[test]
    fn export_omits_identifiers_and_secrets_even_in_unexpected_fields() {
        let view = WifiView {
            on: true,
            connected: Some("PRIVATE_SSID".into()),
            ip: Some("192.0.2.123".into()),
            error: Some("SECRET_PASSWORD".into()),
            busy: Busy::Joining("PRIVATE_SSID".into()),
            ..Default::default()
        };
        let report=facts(&view,&ok("wpa_state=COMPLETED\nssid=PRIVATE_SSID\nip_address=192.0.2.123\nbssid=aa:bb:cc:dd:ee:ff\npsk=SECRET_PASSWORD\ntoken=SECRET_TOKEN\n"),
            &ok(" inet 192.0.2.123/24 brd 192.0.2.255 scope global wlan0"),&ok("default via 192.0.2.1 dev wlan0"),&ok("watcher=SECRET_TOKEN\nowner_present=yes\n"));
        for secret in [
            "PRIVATE_SSID",
            "192.0.2.",
            "aa:bb",
            "SECRET_PASSWORD",
            "SECRET_TOKEN",
        ] {
            assert!(!report.contains(secret), "{secret}");
        }
        assert!(report.contains("interface_ipv4=true\n"));
        assert!(report.contains("supplicant_address_on_interface=true\n"));
        assert!(report.contains("wlan0_default_route=true\n"));
        assert!(report.contains("watcher=unknown\n"));
        assert_eq!(state("wpa_state=SECRET_TOKEN"), "unknown");
    }
    #[test]
    fn differentiates_missing_dhcp_from_missing_supplicant_address() {
        let view = WifiView::default();
        let none = ok("");
        let completed = ok("wpa_state=COMPLETED\n");
        let missing = facts(&view, &completed, &none, &none, &none);
        assert!(missing.contains("interface_ipv4=false"));
        let mismatch = facts(
            &view,
            &completed,
            &ok("inet 10.0.0.2/24 scope global wlan0"),
            &none,
            &none,
        );
        assert!(mismatch.contains("interface_ipv4=true"));
        assert!(mismatch.contains("supplicant_ipv4=false"));
        assert!(addresses("inet 0.0.0.0/32\ninet 169.254.1.2/16\ninet 127.0.0.1/8").is_empty());
    }
    #[test]
    fn saves_distinct_reports_and_refuses_invalid_destination() {
        let root = tempfile::tempdir().unwrap();
        let a = save(root.path(), "first").unwrap();
        let b = save(root.path(), "second").unwrap();
        assert_ne!(a, b);
        assert_eq!(fs::read_to_string(root.path().join(a)).unwrap(), "first");
        let bad = tempfile::tempdir().unwrap();
        fs::write(bad.path().join("Diagnostics"), "keep").unwrap();
        assert!(save(bad.path(), "x").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn refuses_redirected_directory_and_skips_symlink_file() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), root.path().join("Diagnostics")).unwrap();
        assert!(save(root.path(), "x").is_err());
        fs::remove_file(root.path().join("Diagnostics")).unwrap();
        fs::create_dir(root.path().join("Diagnostics")).unwrap();
        let target = outside.path().join("keep");
        fs::write(&target, "keep").unwrap();
        symlink(&target, root.path().join("Diagnostics/wifi-0001.txt")).unwrap();
        assert_eq!(
            save(root.path(), "report").unwrap(),
            "Diagnostics/wifi-0002.txt"
        );
        assert_eq!(fs::read_to_string(target).unwrap(), "keep");
    }
    #[test]
    fn probes_bound_hangs_output_and_failures() {
        assert_eq!(
            probe("/bin/sh", &["-c", "exit 1"], Duration::from_secs(1)).result,
            "failed"
        );
        assert_eq!(
            probe(
                "/bin/sh",
                &["-c", "while :; do :; done"],
                Duration::from_millis(80)
            )
            .result,
            "timeout"
        );
        let result = probe(
            "/bin/sh",
            &["-c", "yes x | head -c 70000"],
            Duration::from_secs(1),
        );
        assert_eq!(result.result, "too-large");
        let failed_after_output = probe(
            "/bin/sh",
            &["-c", "head -c 70000 /dev/zero; exit 7"],
            Duration::from_secs(1),
        );
        assert_eq!(failed_after_output.result, "too-large");
        assert!(failed_after_output.text.is_empty());
    }
}
