use nci::agents::Agent;

use super::na;

#[test]
fn empty() {
    na(Agent::Npm, Vec::new(), "npm".to_string());
}

#[test]
fn foo() {
    na(Agent::Npm, vec!["foo".to_string()], "npm foo".to_string());
}

#[test]
fn run_test() {
    na(
        Agent::Npm,
        vec!["run".to_string(), "test".to_string()],
        "npm run test".to_string(),
    );
}
