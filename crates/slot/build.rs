use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").env("TZ", "UTC").args(args).output().ok()?;
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

    let hash = git(&["rev-parse", "--short=8", "HEAD"])
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
            git(&["rev-parse", "--short=8", "HEAD"])
        })
        .unwrap_or_else(|| "unknown".into());
    // Match the release workflow's UTC source date, independent of when the build runs.
    let date = git(&["show", "-s", "--date=format-local:%Y-%m-%d", "--format=%cd", "HEAD"])
        .unwrap_or_else(|| "unknown".into());
    let version = git(&["describe", "--exact-match", "--tags", "--match", "v[0-9]*"])
        .map(|tag| tag.trim_start_matches('v').to_string())
        .unwrap_or_else(|| {
            if date == "unknown" || hash == "unknown" {
                env!("CARGO_PKG_VERSION").into()
            } else {
                format!("{}-{}", date.replace('-', "."), &hash[..7])
            }
        });

    println!("cargo:rustc-env=SLOT_GIT_HASH={hash}");
    println!("cargo:rustc-env=SLOT_VERSION={version}");
    println!("cargo:rustc-env=SLOT_BUILD_DATE={date}");
}
