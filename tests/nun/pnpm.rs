use nci::agents::Agent;

use super::nun;

#[test]
fn single_remove() {
    nun(
        Agent::Pnpm,
        vec!["axios".to_string()],
        "pnpm remove axios".to_string(),
    );
}

#[test]
fn multiple() {
    nun(
        Agent::Pnpm,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "pnpm remove eslint @types/node".to_string(),
    );
}

#[test]
fn multiple_dev() {
    nun(
        Agent::Pnpm,
        vec![
            "eslint".to_string(),
            "@types/node".to_string(),
            "-D".to_string(),
        ],
        "pnpm remove eslint @types/node -D".to_string(),
    );
}

#[test]
fn global() {
    nun(
        Agent::Pnpm,
        vec!["eslint".to_string(), "ni".to_string(), "-g".to_string()],
        "pnpm remove --global eslint ni".to_string(),
    );
}
