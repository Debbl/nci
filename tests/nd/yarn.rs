//! Yarn 1 has no `dedupe` subcommand (`YARN_COMMAND.dedupe = cmd![]`). When
//! the user runs `nd` against a yarn 1 lockfile, the runtime should fail with
//! a clear "not supported by agent" diagnostic instead of emitting a broken
//! command.
//!
//! We can't reuse the `nd()` helper from `mod.rs` because `parse_nd` calls
//! `process::exit(1)` on the unsupported path — that would tear down the test
//! runner. So we spawn the real binary instead.

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

#[test]
fn yarn1_dedupe_is_unsupported() {
    let dir = TempDir::new().unwrap();
    // Plain yarn.lock + no `packageManager` field => detected as yarn 1.
    fs::write(dir.path().join("yarn.lock"), "").unwrap();
    fs::write(dir.path().join("package.json"), r#"{"name":"demo"}"#).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nd"));
    clean_env(&mut cmd);
    let out = cmd
        .arg("?")
        .current_dir(dir.path())
        .output()
        .expect("spawn nd");

    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not supported by agent")
            && stderr.contains("yarn"),
        "expected unsupported-agent diagnostic, got: {stderr}"
    );
}
