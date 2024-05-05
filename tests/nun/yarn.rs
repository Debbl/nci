use nci::agents::Agent;

use super::nun;

#[test]
fn single_remove() {
    nun(
        Agent::Yarn,
        vec!["axios".to_string()],
        "yarn remove axios".to_string(),
    );
}

#[test]
fn multiple() {
    nun(
        Agent::Yarn,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "yarn remove eslint @types/node".to_string(),
    );
}

#[test]
fn multiple_dev() {
    nun(
        Agent::Yarn,
        vec![
            "eslint".to_string(),
            "@types/node".to_string(),
            "-D".to_string(),
        ],
        "yarn remove eslint @types/node -D".to_string(),
    );
}

#[test]
fn global() {
    nun(
        Agent::Yarn,
        vec!["eslint".to_string(), "ni".to_string(), "-g".to_string()],
        "yarn global remove eslint ni".to_string(),
    );
}
