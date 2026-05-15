//! Tests for the `useSfw` wrapper. With `NI_USE_SFW=true`, the resolved
//! command should be prefixed with `sfw` — or the process should exit
//! non-zero with a helpful message if `sfw` isn't on PATH.
//!
//! Both tests fully control `PATH`: a tempdir with (or without) a fake `sfw`
//! script. That keeps the result independent of whatever the developer has
//! installed locally and also strips `volta` out of the way, which would
//! otherwise wrap the command again and break the assertions.

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, process::Command};
use tempfile::TempDir;

fn clean_env(cmd: &mut Command) -> &mut Command {
    cmd.env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("NI_DEFAULT_AGENT")
        .env_remove("NI_GLOBAL_AGENT")
        .env_remove("NI_RUN_AGENT")
        .env_remove("NI_USE_SFW")
        .env_remove("NI_CATALOG")
        .env_remove("NI_NO_LAST_COMMAND")
        .env_remove("NI_AUTO_INSTALL")
}

fn project_dir_with_npm_lock() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    dir
}

#[cfg(unix)]
fn write_fake_exe(dir: &std::path::Path, name: &str) {
    // The fake binary is never executed — `?` short-circuits before spawn.
    // It only needs to exist on PATH so `which(name)` returns Ok.
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
}

#[cfg(unix)]
#[test]
fn wraps_command_with_sfw_when_available() {
    let path_dir = TempDir::new().unwrap();
    write_fake_exe(path_dir.path(), "sfw");

    let project = project_dir_with_npm_lock();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ni"));
    clean_env(&mut cmd);
    let out = cmd
        .env("NI_USE_SFW", "true")
        // Limit PATH to our tempdir: no `volta`, only our fake `sfw`.
        .env("PATH", path_dir.path())
        .args(["--programmatic", "?"])
        .current_dir(project.path())
        .output()
        .expect("spawn ni");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(stdout, "sfw npm i");
}

#[cfg(unix)]
#[test]
fn errors_when_sfw_enabled_but_not_installed() {
    // PATH with npm but no `sfw` and no `volta`. The runtime should fail fast
    // with a clear message instead of trying to spawn the wrapper.
    let path_dir = TempDir::new().unwrap();
    write_fake_exe(path_dir.path(), "npm");

    let project = project_dir_with_npm_lock();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ni"));
    clean_env(&mut cmd);
    let out = cmd
        .env("NI_USE_SFW", "true")
        .env("PATH", path_dir.path())
        .args(["--programmatic", "?"])
        .current_dir(project.path())
        .output()
        .expect("spawn ni");
    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("sfw is enabled but not installed"),
        "expected sfw-missing diagnostic, got: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn non_programmatic_mode_also_prints_install_hint() {
    let path_dir = TempDir::new().unwrap();
    write_fake_exe(path_dir.path(), "npm");

    let project = project_dir_with_npm_lock();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ni"));
    clean_env(&mut cmd);
    let out = cmd
        .env("NI_USE_SFW", "true")
        .env("PATH", path_dir.path())
        // No `--programmatic`: the second `eprintln!` adds an install hint.
        .args(["?"])
        .current_dir(project.path())
        .output()
        .expect("spawn ni");
    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("npm install -g sfw"),
        "expected install hint in non-programmatic mode, got: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn ni_use_sfw_false_does_not_wrap() {
    // Even when NI_USE_SFW=false is explicitly set, we shouldn't wrap, even if
    // sfw is on PATH.
    let path_dir = TempDir::new().unwrap();
    write_fake_exe(path_dir.path(), "sfw");

    let project = project_dir_with_npm_lock();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ni"));
    clean_env(&mut cmd);
    let out = cmd
        .env("NI_USE_SFW", "false")
        .env("PATH", path_dir.path())
        .args(["--programmatic", "?"])
        .current_dir(project.path())
        .output()
        .expect("spawn ni");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(stdout, "npm i");
}
