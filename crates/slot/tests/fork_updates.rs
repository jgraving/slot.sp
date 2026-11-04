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
