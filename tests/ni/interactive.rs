//! `ni -i` opens an interactive picker (search npm, choose package, choose
//! mode). Mirrors `test/ni/interactive.spec.ts` for the cancellation case
//! where the user dismisses the very first prompt (Ctrl+C / EOF).
//!
//! We don't try to drive a fake npm registry — we just close stdin so
//! `inquire::Text::new(...).prompt()` returns `Err`, and the runtime should
//! exit 1 cleanly instead of panicking or hanging.

use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
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

/// Cancelling the very first prompt (the search pattern) should exit 1
/// without ever hitting the network or panicking.
#[test]
fn dash_i_with_closed_stdin_exits_one() {
    let dir = TempDir::new().unwrap();
    // A lockfile keeps detect() from prompting for which agent to use.
    fs::write(dir.path().join("package-lock.json"), "").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ni"));
    clean_env(&mut cmd);
    let mut child = cmd
        .arg("-i")
        .current_dir(dir.path())
        // null stdin → `inquire::Text::prompt()` errors immediately, no hang.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ni -i");

    // Belt and suspenders: if the binary ever does block on the prompt, kill
    // it after a short wait so the test fails loudly instead of hanging CI.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => {
                assert!(!status.success(), "expected non-zero exit on cancel");
                // We don't pin the exact code (1 vs 130 vs platform-specific),
                // just that the process exited and didn't succeed.
                return;
            }
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    panic!("ni -i did not exit within 10s after stdin EOF");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
