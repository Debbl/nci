use nci::agents::Agent;

use super::nun;

#[test]
fn single() {
    nun(
        Agent::Deno,
        vec!["axios".to_string()],
        "deno remove axios".to_string(),
    );
}

#[test]
fn multiple() {
    nun(
        Agent::Deno,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "deno remove eslint @types/node".to_string(),
    );
}

#[test]
fn global() {
    nun(
        Agent::Deno,
        vec!["eslint".to_string(), "-g".to_string()],
        "deno uninstall -g eslint".to_string(),
    );
}
