use nci::agents::Agent;

use super::na;

#[test]
fn empty() {
    na(Agent::Bun, Vec::new(), "bun".to_string());
}

#[test]
fn foo() {
    na(Agent::Bun, vec!["foo".to_string()], "bun foo".to_string());
}

#[test]
fn run_test() {
    na(
        Agent::Bun,
        vec!["run".to_string(), "test".to_string()],
        "bun run test".to_string(),
    );
}
