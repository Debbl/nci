use nci::agents::Agent;

use super::nun;

#[test]
fn single_uninstall() {
    nun(
        Agent::Bun,
        vec!["axios".to_string()],
        "bun remove axios".to_string(),
    );
}

#[test]
fn multiple() {
    nun(
        Agent::Bun,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "bun remove eslint @types/node".to_string(),
    );
}

#[test]
fn global() {
    nun(
        Agent::Bun,
        vec!["eslint".to_string(), "ni".to_string(), "-g".to_string()],
        "bun remove -g eslint ni".to_string(),
    );
}
