use nci::{agents::Agent, parse::parse_ni, runner::RunnerContext};

#[test]
fn empty() {
    let (agent, args) = parse_ni(
        Agent::Npm,
        vec!["--frozen-if-present".into()],
        Some(RunnerContext {
            programmatic: false,
            has_lock: true,
            cwd: std::env::current_dir().unwrap(),
        }),
    );

    assert_eq!(agent, "npm");
    assert_eq!(args, ["ci"])
}
