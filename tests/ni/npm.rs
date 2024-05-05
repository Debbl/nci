use nci::agents::Agent;

use super::ni;

#[test]
fn empty() {
    ni(Agent::Npm, Vec::new(), "npm i".to_string());
}

#[test]
fn single_add() {
    ni(
        Agent::Npm,
        vec!["axios".to_string()],
        "npm i axios".to_string(),
    );
}

#[test]
fn add_dev() {
    ni(
        Agent::Npm,
        vec!["vite".to_string(), "-D".to_string()],
        "npm i vite -D".to_string(),
    );
}

#[test]
fn multiple() {
    ni(
        Agent::Npm,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "npm i eslint @types/node".to_string(),
    );
}

#[test]
fn global() {
    ni(
        Agent::Npm,
        vec!["eslint".to_string(), "-g".to_string()],
        "npm i -g eslint".to_string(),
    )
}

#[test]
fn frozen() {
    ni(
        Agent::Npm,
        vec!["--frozen".to_string()],
        "npm ci".to_string(),
    )
}
