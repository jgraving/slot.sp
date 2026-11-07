use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/logs/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/tags");
    if std::path::Path::new("../../.git/packed-refs").exists() {
        println!("cargo:rerun-if-changed=../../.git/packed-refs");
    }

    // Checkout can move a symbolic branch without changing .git/HEAD.
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"]) {
        if let Some(path) = git(&["rev-parse", "--git-path", &reference]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    let source = git(&["rev-parse", "HEAD"])
        .or_else(|| {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let root = root.canonicalize().ok()?;
            git(&[
                "config",
                "--global",
                "--add",
                "safe.directory",
                root.to_str()?,
            ])?;
            git(&["rev-parse", "HEAD"])
        })
        .unwrap_or_else(|| "unknown".into());
    let hash = &source[..source.len().min(8)];
    let date = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    let version=format!("{}-{}",date.replace('-',"."),&hash[..hash.len().min(7)]);
    println!("cargo:rustc-env=SLOT_GIT_HASH={hash}");
    println!("cargo:rustc-env=SLOT_SOURCE_COMMIT={source}");
    println!("cargo:rustc-env=SLOT_VERSION={version}");
    println!("cargo:rustc-env=SLOT_BUILD_DATE={date}");
}
