use nci::agents::Agent;

use super::na;

#[test]
fn empty() {
    na(Agent::Yarn, Vec::new(), "yarn".to_string());
}

#[test]
fn foo() {
    na(Agent::Yarn, vec!["foo".to_string()], "yarn foo".to_string());
}

#[test]
fn run_test() {
    na(
        Agent::Yarn,
        vec!["run".to_string(), "test".to_string()],
        "yarn run test".to_string(),
    );
}
