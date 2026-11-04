use slot::update::*;
use serde_json::{json,Value};
use std::io::{Cursor,Write};
const SHA:&str="1111111111111111111111111111111111111111";
fn release()->Value {json!({"draft":false,"prerelease":false,"tag_name":"build-2026.11.03-1111111","target_commitish":SHA,"assets":[{"name":"slot.sp-System.zip","state":"uploaded","size":123,"digest":format!("sha256:{}","ab".repeat(32)),"browser_download_url":"https://github.com/jgraving/slot.sp/releases/download/build-2026.11.03-1111111/slot.sp-System.zip"}]})}
fn package(source:&str,extra:Option<&str>)->Vec<u8> {
    package_with_entry(source,extra.map(|path|(path,b"bad".as_slice())))
}
fn package_with_entry(source:&str,extra:Option<(&str,&[u8])>)->Vec<u8> {
    let mut zip=zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options=zip::write::FileOptions::default().unix_permissions(0o755);
    for name in ["slot","launch_frontend.sh","mgba_libretro.so","gpsp_libretro.so"] {
        zip.start_file(format!("System/{name}"),options).unwrap();zip.write_all(b"fixture").unwrap();
    }
    zip.start_file("System/build-info.json",options).unwrap();
    zip.write_all(json!({"product":"slot.sp","repository":"jgraving/slot.sp","architecture":"aarch64","source_commit":source}).to_string().as_bytes()).unwrap();
    if let Some((path,bytes))=extra {zip.start_file(path,options).unwrap();zip.write_all(bytes).unwrap();}
    zip.finish().unwrap().into_inner()
}
// Craft external metadata directly: ZipWriter strips special Unix mode bits.
// The central directory is the input the updater's ZipFile actually observes.
fn entry_metadata(bytes:&mut [u8],name:&str,mode:u32,size:Option<u32>) {
    let mut at=0;
    while at+46<=bytes.len() {
        if &bytes[at..at+4]!=b"PK\x01\x02" {at+=1;continue;}
        let len=u16::from_le_bytes(bytes[at+28..at+30].try_into().unwrap()) as usize;
        if &bytes[at+46..at+46+len]==name.as_bytes() {
            bytes[at+5]=3; // Unix creator, so the high attribute word is the file mode.
            bytes[at+38..at+42].copy_from_slice(&(mode<<16).to_le_bytes());
            if let Some(size)=size {bytes[at+24..at+28].copy_from_slice(&size.to_le_bytes());}
            return;
        }
        at+=46+len;
    }
    panic!("missing fixture entry {name}");
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
fn staging_rejects_symlink_metadata_including_an_enormous_declared_target() {
    let d=tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    let private=d.path().join("Config/settings");
    std::fs::write(&private,b"keep").unwrap();
    let r=parse_release(&release().to_string()).unwrap();
    for size in [None,Some(0xffff_fffe)] {
        let mut bytes=package_with_entry(SHA,Some(("System/escape",b"../../Config/settings")));
        entry_metadata(&mut bytes,"System/escape",0o120777,size);
        let mut archive=zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let entry=archive.by_name("System/escape").unwrap();
        assert_eq!(entry.unix_mode().unwrap()&0o170000,0o120000);
        if let Some(size)=size {assert_eq!(entry.size(),u64::from(size));}
        drop(entry);
        assert!(stage(d.path(),&bytes,&r).is_err());
        assert!(!d.path().join(STAGING).exists());
        assert_eq!(std::fs::read(&private).unwrap(),b"keep");
    }
}
#[test]
fn staging_strips_privileged_mode_bits_but_preserves_executable_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let d=tempfile::tempdir().unwrap();
    let r=parse_release(&release().to_string()).unwrap();
    let mut bytes=package(SHA,None);
    entry_metadata(&mut bytes,"System/slot",0o107755,None);
    let mut archive=zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    assert_eq!(archive.by_name("System/slot").unwrap().unix_mode().unwrap()&0o7777,0o7755);
    stage(d.path(),&bytes,&r).unwrap();
    let installed=d.path().join(STAGING).join("slot");
    assert_eq!(std::fs::metadata(installed).unwrap().permissions().mode()&0o7777,0o755);
}
#[test]
fn staging_rejects_enormous_regular_entries_before_reading_them() {
    let d=tempfile::tempdir().unwrap();
    let r=parse_release(&release().to_string()).unwrap();
    let mut bytes=package(SHA,Some("System/huge"));
    entry_metadata(&mut bytes,"System/huge",0o100644,Some(0xffff_fffe));
    assert!(stage(d.path(),&bytes,&r).is_err());
    assert!(!d.path().join(STAGING).exists());
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

#[test]
fn inconsistent_expanded_size_cannot_bypass_staging_budget() {
    let d=tempfile::tempdir().unwrap();let r=parse_release(&release().to_string()).unwrap();
    let mut bytes=package(SHA,Some("System/extra"));
    entry_metadata(&mut bytes,"System/extra",0o100644,Some(1));
    assert!(stage(d.path(),&bytes,&r).is_err());
    assert!(!d.path().join(STAGING).exists());
}
#[test]
fn failed_download_checks_never_touch_installed_or_user_files() {
    let d=tempfile::tempdir().unwrap();
    for path in ["System/slot","Config/settings","Games/game.gba","Saves/game.sav"] {
        let p=d.path().join(path);std::fs::create_dir_all(p.parent().unwrap()).unwrap();std::fs::write(p,b"keep").unwrap();
    }
    let bytes=package(SHA,None);let mut r=parse_release(&release().to_string()).unwrap();r.size=bytes.len() as u64;
    // Correct size, incorrect digest; then correct digest with truncation.
    assert_eq!(install_with(d.path(),&mut |_|Ok(bytes.clone()),&r),UpdateEvent::Failed(UpdateFail::Checksum));
    r.sha256=ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().try_into().unwrap();
    assert_eq!(install_with(d.path(),&mut |_|Ok(bytes[..bytes.len()-1].to_vec()),&r),UpdateEvent::Failed(UpdateFail::Download));
    assert!(!d.path().join(STAGING).exists());
    for path in ["System/slot","Config/settings","Games/game.gba","Saves/game.sav"] {assert_eq!(std::fs::read(d.path().join(path)).unwrap(),b"keep");}
}

fn baseos_fixture()->Release {
    Release{kind:Kind::BaseOs,running:"1.3.0".into(),version:"1.4.0".into(),url:"https://github.com/pvaibhav/BaseOS/releases/download/v1.4.0/baseos-rg35xxsp-1.4.0.bosupd".into(),sha256:ring::digest::digest(&ring::digest::SHA256,b"payload").as_ref().try_into().unwrap(),size:7,source:String::new()}
}
#[test]
fn baseos_staging_preserves_existing_payloads_and_refuses_redirected_downloads() {
    use std::os::unix::fs::symlink;
    let r=baseos_fixture();let name=bosupd_name(&r);
    for collision in ["ready","part-link","ready-link","part-directory"] {
        let d=tempfile::tempdir().unwrap();let keep=d.path().join("keep");std::fs::write(&keep,b"keep").unwrap();
        let done=d.path().join(&name);let part=d.path().join(format!("{name}.part"));
        match collision {
            "ready"=>std::fs::write(&done,b"existing").unwrap(),
            "part-link"=>symlink(&keep,&part).unwrap(),
            "ready-link"=>symlink(&keep,&done).unwrap(),
            _=>std::fs::create_dir(&part).unwrap(),
        }
        let result=install_baseos_with(d.path(),100,&mut |_,_|panic!("must reject before downloading"),&mut ||panic!("must not inspect rejected path"),&r);
        assert!(matches!(result,UpdateEvent::Failed(_)));
        assert_eq!(std::fs::read(&keep).unwrap(),b"keep");
        if collision=="ready" {assert_eq!(std::fs::read(done).unwrap(),b"existing");}
    }
}
#[test]
fn baseos_failures_never_leave_a_bootable_payload_and_can_retry() {
    let r=baseos_fixture();let name=bosupd_name(&r);
    for failure in ["space","download","cancel","length","digest","verdict","missing-status"] {
        let d=tempfile::tempdir().unwrap();let keep=d.path().join("Config/settings");std::fs::create_dir_all(keep.parent().unwrap()).unwrap();std::fs::write(&keep,b"keep").unwrap();
        let mut status_called=false;
        let result=install_baseos_with(d.path(),if failure=="space" {0}else{100},&mut |_,out| {
            assert_ne!(failure,"space");
            out.write_all(if failure=="length" {b"short"}else{b"payload"}).unwrap();
            if failure=="download" {return Err(UpdateFail::Download);}
            if failure=="cancel" {
                let t=Ticket::default();t.cancel();let mut read=Stoppable::new(Cursor::new(b"payload"),t);
                return copy_counted(&mut read,out,r.size,&mut |_|{});
            }
            Ok(if failure=="digest" {[0;32]}else{r.sha256})
        },&mut || {status_called=true;if failure=="missing-status" {None}else{Some(format!("payload: /mnt/sdcard/{name}\nrejected: wrong board\n"))}},&r);
        assert!(matches!(result,UpdateEvent::Failed(_)),"{failure}");
        assert_eq!(status_called,matches!(failure,"verdict"|"missing-status"));
        assert!(!d.path().join(&name).exists());assert!(!d.path().join(format!("{name}.part")).exists());
        assert_eq!(std::fs::read(&keep).unwrap(),b"keep");
        // Restarting a partial download starts from byte zero and publishes only
        // after complete size/digest validation and the matching BaseOS verdict.
        std::fs::write(d.path().join(format!("{name}.part")),b"stale incomplete").unwrap();
        assert_eq!(install_baseos_with(d.path(),100,&mut |_,out|{out.write_all(b"payload").unwrap();Ok(r.sha256)},&mut ||Some(format!("payload: /mnt/sdcard/{name}\nWILL APPLY on the next boot\n")),&r),UpdateEvent::Staged);
        assert_eq!(std::fs::read(d.path().join(&name)).unwrap(),b"payload");
        assert!(!d.path().join(format!("{name}.part")).exists());
    }
}
#[test]
fn baseos_verdict_cannot_be_borrowed_from_another_payload() {
    let name=bosupd_name(&baseos_fixture());
    for status in [format!("payload: /card/other.bosupd\nWILL APPLY\npayload: /card/{name}\nrejected: incompatible\n"),format!("payload: /card/{name}\nrejected: incompatible\npayload: /card/other.bosupd\nWILL APPLY\n"),"payload: /card/other.bosupd\nWILL APPLY\n".into()] {
        assert!(will_apply(&status,&name).is_err());
    }
}
#[test]
fn staging_root_collisions_cannot_modify_an_unrelated_tree() {
    let d=tempfile::tempdir().unwrap();let outside=tempfile::tempdir().unwrap();
    let keep=outside.path().join("slot");std::fs::write(&keep,b"keep").unwrap();
    std::os::unix::fs::symlink(outside.path(),d.path().join(STAGING)).unwrap();
    let r=parse_release(&release().to_string()).unwrap();
    assert!(stage(d.path(),&package(SHA,None),&r).is_err());
    assert_eq!(std::fs::read(&keep).unwrap(),b"keep");
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(),1);
}

#[test]
fn baseos_failure_does_not_hide_a_launcher_update() {
    let body=release().to_string().into_bytes();
    let result=check_with(2_000_000_000,&mut |url| {
        if url==LATEST {Ok(body.clone())}
        else if url.contains("/compare/") {Ok(json!({"status":"ahead"}).to_string().into_bytes())}
        else {Err(UpdateFail::Offline)}
    },Running{slot:"22222222",baseos:Some(("rgsp","1.3.0"))});
    let UpdateEvent::Available(offers)=result else {panic!("{result:?}");};
    assert_eq!(offers.len(),1);assert_eq!(offers[0].kind,Kind::Slot);
}
#[test]
fn newer_rgsp_baseos_is_offered_even_when_launcher_is_current() {
    let v=json!({"draft":false,"prerelease":false,"tag_name":"v1.4.0","assets":[{"name":"baseos-rgsp-1.4.0.bosupd","state":"uploaded","size":54917120,"digest":format!("sha256:{}","ab".repeat(32)),"browser_download_url":"https://github.com/pvaibhav/BaseOS/releases/download/v1.4.0/baseos-rgsp-1.4.0.bosupd"}]}).to_string();
    let result=check_with(2_000_000_000,&mut |url|Ok(if url==LATEST {release().to_string()}else{v.clone()}.into_bytes()),Running{slot:"11111111",baseos:Some(("rgsp","1.3.0"))});
    let UpdateEvent::Available(offers)=result else {panic!("{result:?}");};
    assert_eq!(offers.len(),1);assert_eq!(offers[0].kind,Kind::BaseOs);assert_eq!(offers[0].version,"1.4.0");
}
