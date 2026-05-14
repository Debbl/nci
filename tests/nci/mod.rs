use nci::{agents::Agent, parse::parse_ni, runner::RunnerContext};

pub mod bun;
pub mod deno;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

/// Helper: call `parse_ni` the same way `bin/nci.rs` does — with
/// `--frozen-if-present` and a context carrying `has_lock`.
pub fn nci(agent: Agent, has_lock: bool, expected_agent: &str, expected_args: &[&str]) {
    let ctx = RunnerContext {
        programmatic: false,
        has_lock,
        cwd: std::env::current_dir().unwrap(),
    };
    let (got_agent, got_args) = parse_ni(agent, vec!["--frozen-if-present".into()], Some(ctx));
    assert_eq!(got_agent, expected_agent);
    assert_eq!(got_args, expected_args);
}
