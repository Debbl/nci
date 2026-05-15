//! End-to-end tests for the config pipeline (`src/config.rs`). We spawn the
//! real `ni` binary with `?` (dry-run) plus `--programmatic` and observe the
//! resolved command in stdout — that command is determined by the agent that
//! `assign()` produces, so it doubles as a readout of the layered config.

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

fn run_ni(dir: &std::path::Path, args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ni"));
    clean_env(&mut cmd);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.args(args).current_dir(dir).output().expect("spawn ni")
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn skip_if_volta() -> bool {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed, would prepend the resolved command");
        return true;
    }
    false
}

#[test]
fn defaults_to_npm_in_programmatic_mode_without_lockfile() {
    if skip_if_volta() {
        return;
    }
    let dir = TempDir::new().unwrap();
    let out = run_ni(dir.path(), &["--programmatic", "?"], &[]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    // No lockfile + programmatic + default `prompt` config → falls through to
    // npm via `get_default_agent`.
    assert_eq!(stdout(&out), "npm i");
}

#[test]
fn env_ni_default_agent_overrides_default() {
    if skip_if_volta() {
        return;
    }
    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?"],
        &[("NI_DEFAULT_AGENT", "pnpm")],
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm i");
}

#[test]
fn env_ni_global_agent_picks_global_for_dash_g() {
    if skip_if_volta() {
        return;
    }
    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?", "-g", "tsx"],
        &[("NI_GLOBAL_AGENT", "pnpm")],
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm add -g tsx");
}

#[test]
fn nirc_camel_case_default_agent_is_loaded() {
    if skip_if_volta() {
        return;
    }
    let rc_dir = TempDir::new().unwrap();
    let rc_path = rc_dir.path().join(".nirc");
    fs::write(&rc_path, "defaultAgent=pnpm\n").unwrap();

    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?"],
        &[("NI_CONFIG_FILE", rc_path.to_str().unwrap())],
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm i");
}

#[test]
fn nirc_snake_case_default_agent_is_loaded() {
    if skip_if_volta() {
        return;
    }
    let rc_dir = TempDir::new().unwrap();
    let rc_path = rc_dir.path().join(".nirc");
    // The fallback form — both forms are accepted, see `section_get`.
    fs::write(&rc_path, "default_agent=yarn\n").unwrap();

    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?"],
        &[("NI_CONFIG_FILE", rc_path.to_str().unwrap())],
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "yarn install");
}

#[test]
fn env_overrides_nirc() {
    if skip_if_volta() {
        return;
    }
    let rc_dir = TempDir::new().unwrap();
    let rc_path = rc_dir.path().join(".nirc");
    fs::write(&rc_path, "defaultAgent=yarn\n").unwrap();

    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?"],
        &[
            ("NI_CONFIG_FILE", rc_path.to_str().unwrap()),
            ("NI_DEFAULT_AGENT", "pnpm"),
        ],
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    // Env wins over the file.
    assert_eq!(stdout(&out), "pnpm i");
}

#[test]
fn nirc_global_agent_camel_case() {
    if skip_if_volta() {
        return;
    }
    let rc_dir = TempDir::new().unwrap();
    let rc_path = rc_dir.path().join(".nirc");
    fs::write(&rc_path, "globalAgent=pnpm\n").unwrap();

    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?", "-g", "tsx"],
        &[("NI_CONFIG_FILE", rc_path.to_str().unwrap())],
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm add -g tsx");
}

#[test]
fn missing_nirc_file_is_silently_ignored() {
    if skip_if_volta() {
        return;
    }
    let dir = TempDir::new().unwrap();
    let out = run_ni(
        dir.path(),
        &["--programmatic", "?"],
        &[("NI_CONFIG_FILE", "/this/path/definitely/does/not/exist/.nirc")],
    );
    // The runtime should fall back to defaults, not error out.
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm i");
}
