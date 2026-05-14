use nci::agents::Agent;

use super::ni;

#[test]
fn empty() {
    ni(Agent::Deno, Vec::new(), "deno install".to_string());
}

#[test]
fn single_add() {
    ni(
        Agent::Deno,
        vec!["axios".to_string()],
        "deno add axios".to_string(),
    );
}

#[test]
fn multiple() {
    ni(
        Agent::Deno,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "deno add eslint @types/node".to_string(),
    );
}

#[test]
fn global() {
    ni(
        Agent::Deno,
        vec!["eslint".to_string(), "-g".to_string()],
        "deno install -g eslint".to_string(),
    );
}

#[test]
fn frozen() {
    ni(
        Agent::Deno,
        vec!["--frozen".to_string()],
        "deno install --frozen".to_string(),
    );
}
