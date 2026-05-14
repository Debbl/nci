//! Smoke tests for the binary entry points: spawn real binaries with the new
//! global flags (`--agent`, `?`, `--help`) and assert on stdout.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
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

fn run_na_in(dir: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_na"));
    clean_env(&mut cmd);
    cmd.args(args)
        .current_dir(dir)
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

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_na"));
    clean_env(&mut cmd);
    let out = cmd
        .args(["-C", "project", "--agent"])
        .current_dir(outer.path())
        .output()
        .expect("spawn na");
    assert!(out.status.success());
    assert_eq!(stdout(&out), "pnpm");
}

#[test]
fn help_lists_nd_and_nup() {
    let dir = TempDir::new().unwrap();
    let out = run_na_in(dir.path(), &["--help"]);
    let s = stdout(&out);
    assert!(s.contains("nd    -"), "help missing nd line:\n{s}");
    assert!(s.contains("nup   -"), "help missing nup line:\n{s}");
}

#[test]
fn nd_binary_runs() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nd"));
    clean_env(&mut cmd);
    let out = cmd
        .args(["?"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nd");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm dedupe");
}

#[test]
fn nup_binary_runs() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nup"));
    clean_env(&mut cmd);
    let out = cmd
        .args(["?"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nup");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm update");
}

#[test]
fn nu_legacy_alias_still_works() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nu"));
    clean_env(&mut cmd);
    let out = cmd
        .args(["?"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nu");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm update");
}

#[test]
fn nci_forwards_user_args() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    // `nci foo` should produce a frozen install with foo appended.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nci"));
    clean_env(&mut cmd);
    let out = cmd
        .args(["?", "foo"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nci");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm ci foo");
}

#[test]
fn nr_run_agent_node_via_env() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nr"));
    clean_env(&mut cmd);
    let out = cmd
        .env("NI_RUN_AGENT", "node")
        .args(["?", "dev"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nr");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "node --run dev");
}

#[test]
fn nr_p_auto_selects_single_package() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    // One workspace package that has a `build` script. With only one match,
    // -p should auto-select it (no prompt) and emit `npm run build`.
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package.json"), r#"{"name":"root"}"#).unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    fs::create_dir_all(dir.path().join("packages/api")).unwrap();
    fs::write(
        dir.path().join("packages/api/package.json"),
        r#"{"name":"api","scripts":{"build":"tsc"}}"#,
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nr"));
    clean_env(&mut cmd);
    let out = cmd
        .args(["-p", "build", "?"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nr");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "npm run build");
}

#[test]
fn nr_run_agent_node_strips_if_present() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nr"));
    clean_env(&mut cmd);
    let out = cmd
        .env("NI_RUN_AGENT", "node")
        .args(["?", "test", "--if-present"])
        .current_dir(dir.path())
        .output()
        .expect("spawn nr");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "node --run test");
}
