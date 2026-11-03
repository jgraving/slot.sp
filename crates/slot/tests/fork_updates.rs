use slot::update::*;
use serde_json::{json,Value};
use std::io::{Cursor,Write};
const SHA:&str="1111111111111111111111111111111111111111";
fn release()->Value {json!({"draft":false,"prerelease":false,"tag_name":"build-2026.11.03-1111111","target_commitish":SHA,"assets":[{"name":"slot.sp-System.zip","state":"uploaded","size":123,"digest":format!("sha256:{}","ab".repeat(32)),"browser_download_url":"https://github.com/jgraving/slot.sp/releases/download/build-2026.11.03-1111111/slot.sp-System.zip"}]})}
fn package(source:&str,extra:Option<&str>)->Vec<u8> {
    let mut zip=zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options=zip::write::FileOptions::default().unix_permissions(0o755);
    for name in ["slot","launch_frontend.sh","mgba_libretro.so","gpsp_libretro.so"] {
        zip.start_file(format!("System/{name}"),options).unwrap();zip.write_all(b"fixture").unwrap();
    }
    zip.start_file("System/build-info.json",options).unwrap();
    zip.write_all(json!({"product":"slot.sp","repository":"jgraving/slot.sp","architecture":"aarch64","source_commit":source}).to_string().as_bytes()).unwrap();
    if let Some(path)=extra {zip.start_file(path,options).unwrap();zip.write_all(b"bad").unwrap();}
    zip.finish().unwrap().into_inner()
}
#[test]
fn only_a_published_fork_asset_with_a_digest_is_accepted() {
    let v=release();let r=parse_release(&v.to_string()).unwrap();assert_eq!(r.source,SHA);
    for bad in ["https://github.com/BrandonKowalski/slot/releases/download/x/slot.sp-System.zip","http://github.com/jgraving/slot.sp/releases/download/x/slot.sp-System.zip"] {
        let mut v=release();v["assets"][0]["browser_download_url"]=json!(bad);assert!(parse_release(&v.to_string()).is_err());
    }
    let mut v=release();v["draft"]=json!(true);assert!(parse_release(&v.to_string()).is_err());
    let mut v=release();v["assets"][0]["digest"]=Value::Null;assert!(parse_release(&v.to_string()).is_err());
}
#[test]
fn staging_checks_source_and_cannot_write_config_or_escape_system() {
    let d=tempfile::tempdir().unwrap();std::fs::create_dir_all(d.path().join("System")).unwrap();std::fs::write(d.path().join("System/slot"),b"installed").unwrap();
    let r=parse_release(&release().to_string()).unwrap();
    for zip in [package("2222222222222222222222222222222222222222",None),package(SHA,Some("System/../../Config/settings")),package(SHA,Some("Config/settings")),package(SHA,Some("System/bad\nslot"))] {
        assert!(stage(d.path(),&zip,&r).is_err());assert!(!d.path().join(STAGING).exists());
        assert_eq!(std::fs::read(d.path().join("System/slot")).unwrap(),b"installed");
    }
    stage(d.path(),&package(SHA,None),&r).unwrap();assert!(d.path().join(STAGING).join(MANIFEST).is_file());
    assert!(stage(d.path(),&package(SHA,None),&r).is_err(),"a ready update is never overwritten");
}
#[test]
fn baseos_is_exact_model_stable_version_and_requires_a_verdict() {
    let v=json!({"draft":false,"prerelease":false,"tag_name":"v1.4.0","assets":[{"name":"baseos-rg35xxsp-1.4.0.bosupd","state":"uploaded","size":50,"digest":format!("sha256:{}","ab".repeat(32)),"browser_download_url":"https://github.com/pvaibhav/BaseOS/releases/download/v1.4.0/baseos-rg35xxsp-1.4.0.bosupd"}]}).to_string();
    assert!(parse_baseos(&v,"rg35xxsp","1.3.0").is_some());assert!(parse_baseos(&v,"rg34xxsp","1.3.0").is_none());
    assert!(will_apply("payload: /mnt/baseos-rg35xxsp-1.4.0.bosupd\nWILL APPLY\n","baseos-rg35xxsp-1.4.0.bosupd").is_ok());
    assert!(will_apply("payload: /mnt/baseos-rg35xxsp-1.4.0.bosupd\nrejected: wrong board\n","baseos-rg35xxsp-1.4.0.bosupd").is_err());
    assert!(will_apply("WILL APPLY","baseos-rg35xxsp-1.4.0.bosupd").is_err());
}
#[test]
fn cancellation_never_commits_an_install_after_the_ticket_is_cancelled() {
    let t=Ticket::default();assert!(t.cancel());assert!(!t.commit());assert_eq!(t.settle(UpdateEvent::Staged),UpdateEvent::Cancelled);
    let mut reader=Stoppable::new(Cursor::new(b"data"),t);assert!(std::io::Read::read(&mut reader,&mut [0;4]).is_err());
}
#[test]
fn update_checks_use_fork_api_and_skip_the_running_source() {
    let mut urls=Vec::new();let v=release().to_string().into_bytes();
    let e=check_with(2_000_000_000,&mut |url|{urls.push(url.to_string());Ok(v.clone())},Running{slot:"11111111",baseos:None});
    assert_eq!(e,UpdateEvent::UpToDate);assert_eq!(urls,vec![LATEST]);
}

#[test]
fn newer_or_diverged_test_builds_are_never_offered_an_older_stable_launcher() {
    for status in ["behind","diverged","identical"] {
        let v=release().to_string().into_bytes();
        let result=check_with(2_000_000_000,&mut |url| if url==LATEST {Ok(v.clone())}else {assert!(url.starts_with("https://api.github.com/repos/jgraving/slot.sp/compare/"));Ok(json!({"status":status}).to_string().into_bytes())},Running{slot:"22222222",baseos:None});
        assert_eq!(result,UpdateEvent::UpToDate);
    }
}
