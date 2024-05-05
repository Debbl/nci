use nci::agents::Agent;

use super::na;

#[test]
fn empty() {
    na(Agent::Pnpm, Vec::new(), "pnpm".to_string());
}

#[test]
fn foo() {
    na(Agent::Pnpm, vec!["foo".to_string()], "pnpm foo".to_string());
}

#[test]
fn run_test() {
    na(
        Agent::Pnpm,
        vec!["run".to_string(), "test".to_string()],
        "pnpm run test".to_string(),
    );
}
