use std::{path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Re-run when HEAD moves so CARGO_VERSION_INFO embeds the current SHA.
    // .git/HEAD catches branch switches and detached-HEAD checkouts.
    // .git/logs/HEAD (the reflog) gets an append entry on every HEAD-moving
    // operation (commit, reset, rebase, amend, checkout), including commits
    // that move the current branch tip without editing .git/HEAD's content.
    // Without these, a plain `cargo install --path . --force` after a new
    // commit reinstalls a binary whose --version reports the parent SHA,
    // causing silent drift between the installed binary and the source tree.
    if Path::new(".git/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/HEAD");
    }
    if Path::new(".git/logs/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/logs/HEAD");
    }

    if !commit_info() {
        println!(
            "cargo:rustc-env=CARGO_VERSION_INFO={}",
            env!("CARGO_PKG_VERSION"),
        );
    }
}

fn commit_info() -> bool {
    if !Path::new(".git").exists() {
        return false;
    }
    let output = match Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--date=short")
        .arg("--format=%H %h %cd")
        .arg("--abbrev")
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return false,
    };
    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut parts = stdout.split_whitespace();
    let mut next = || parts.next().unwrap();
    println!("cargo:rustc-env=CARGO_GIT_HASH={}", next());
    println!(
        "cargo:rustc-env=CARGO_VERSION_INFO={} ({} {})",
        env!("CARGO_PKG_VERSION"),
        next(),
        next()
    );
    true
}
