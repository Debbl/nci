use nci::agents::Agent;

use super::na;

#[test]
fn empty() {
    na(Agent::Deno, Vec::new(), "deno".to_string());
}

#[test]
fn foo() {
    na(Agent::Deno, vec!["foo".to_string()], "deno foo".to_string());
}

#[test]
fn run_test() {
    na(
        Agent::Deno,
        vec!["run".to_string(), "test".to_string()],
        "deno run test".to_string(),
    );
}
