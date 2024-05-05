use nci::agents::Agent;

use super::nun;

#[test]
fn single_remove() {
    nun(
        Agent::YarnBerry,
        vec!["axios".to_string()],
        "yarn remove axios".to_string(),
    );
}

#[test]
fn multiple() {
    nun(
        Agent::YarnBerry,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "yarn remove eslint @types/node".to_string(),
    );
}

#[test]
fn multiple_dev() {
    nun(
        Agent::YarnBerry,
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
        Agent::YarnBerry,
        vec!["eslint".to_string(), "ni".to_string(), "-g".to_string()],
        "npm uninstall -g eslint ni".to_string(),
    );
}
