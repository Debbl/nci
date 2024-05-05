use nci::agents::Agent;

use super::nun;

#[test]
fn single_uninstall() {
    nun(
        Agent::Npm,
        vec!["axios".to_string()],
        "npm uninstall axios".to_string(),
    );
}

#[test]
fn multiple() {
    nun(
        Agent::Npm,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "npm uninstall eslint @types/node".to_string(),
    );
}

#[test]
fn multiple_dev() {
    nun(
        Agent::Npm,
        vec![
            "eslint".to_string(),
            "@types/node".to_string(),
            "-D".to_string(),
        ],
        "npm uninstall eslint @types/node -D".to_string(),
    );
}

#[test]
fn global() {
    nun(
        Agent::Npm,
        vec!["eslint".to_string(), "ni".to_string(), "-g".to_string()],
        "npm uninstall -g eslint ni".to_string(),
    );
}
