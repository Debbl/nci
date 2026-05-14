//! Smoke tests for the binary entry points: spawn real binaries with the new
//! global flags (`--agent`, `?`, `--help`) and assert on stdout.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

fn run_na_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_na"))
        .args(args)
        .current_dir(dir)
        .env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("NI_DEFAULT_AGENT")
        .output()
        .expect("spawn na")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn help_does_not_crash() {
    let dir = TempDir::new().unwrap();
    let out = run_na_in(dir.path(), &["--help"]);
    assert!(out.status.success(), "help should exit zero");
    let s = stdout(&out);
    assert!(s.contains("ni    -"), "help missing ni line:\n{s}");
    assert!(s.contains("ni ?"), "help missing dry-run line:\n{s}");
}

#[test]
fn agent_flag_no_lock() {
    let dir = TempDir::new().unwrap();
    let out = run_na_in(dir.path(), &["--agent"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "unknown");
}

#[test]
fn agent_flag_with_npm_lock() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let out = run_na_in(dir.path(), &["--agent"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "npm");
}

#[test]
fn agent_flag_with_pnpm_lock() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();
    let out = run_na_in(dir.path(), &["--agent"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "pnpm");
}

#[test]
fn agent_flag_with_deno_json() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("deno.json"), "{}").unwrap();
    let out = run_na_in(dir.path(), &["--agent"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "deno");
}

/// Dry-run via `?` should print the resolved command and not execute. Skipped
/// when Volta is present on PATH, since the prefix would change the output.
#[test]
fn debug_dry_run() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed, output would be prefixed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let out = run_na_in(dir.path(), &["?", "foo"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm foo");
}

#[test]
fn change_directory_consumes_two_args() {
    // -C <path> should be consumed; the rest of the args go to the runner.
    let outer = TempDir::new().unwrap();
    let inner = outer.path().join("project");
    fs::create_dir(&inner).unwrap();
    fs::write(inner.join("pnpm-lock.yaml"), "").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_na"))
        .args(["-C", "project", "--agent"])
        .current_dir(outer.path())
        .env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("NI_DEFAULT_AGENT")
        .output()
        .expect("spawn na");
    assert!(out.status.success());
    assert_eq!(stdout(&out), "pnpm");
}
