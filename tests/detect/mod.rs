use std::{fs, path::PathBuf};

use nci::{agents::Agent, detect::detect, runner::DetectOptions};
use tempfile::TempDir;

fn detect_in(files: &[&str]) -> (TempDir, Option<Agent>) {
    let dir = tempfile::tempdir().expect("create tempdir");
    for f in files {
        let path: PathBuf = dir.path().join(f);
        fs::write(&path, "").expect("write fixture");
    }
    let options = DetectOptions {
        cwd: dir.path().to_path_buf(),
        auto_install: false,
        programmatic: true,
    };
    let agent = detect(options);
    (dir, agent)
}

#[test]
fn empty_dir() {
    let (_d, agent) = detect_in(&[]);
    assert_eq!(agent, None);
}

#[test]
fn package_lock() {
    let (_d, agent) = detect_in(&["package-lock.json"]);
    assert_eq!(agent, Some(Agent::Npm));
}

#[test]
fn npm_shrinkwrap() {
    let (_d, agent) = detect_in(&["npm-shrinkwrap.json"]);
    assert_eq!(agent, Some(Agent::Npm));
}

#[test]
fn pnpm_lock() {
    let (_d, agent) = detect_in(&["pnpm-lock.yaml"]);
    assert_eq!(agent, Some(Agent::Pnpm));
}

#[test]
fn yarn_lock() {
    let (_d, agent) = detect_in(&["yarn.lock"]);
    assert_eq!(agent, Some(Agent::Yarn));
}

#[test]
fn bun_lockb() {
    let (_d, agent) = detect_in(&["bun.lockb"]);
    assert_eq!(agent, Some(Agent::Bun));
}

#[test]
fn bun_lock_text() {
    let (_d, agent) = detect_in(&["bun.lock"]);
    assert_eq!(agent, Some(Agent::Bun));
}

#[test]
fn deno_json() {
    let (_d, agent) = detect_in(&["deno.json"]);
    assert_eq!(agent, Some(Agent::Deno));
}

#[test]
fn deno_jsonc() {
    let (_d, agent) = detect_in(&["deno.jsonc"]);
    assert_eq!(agent, Some(Agent::Deno));
}
