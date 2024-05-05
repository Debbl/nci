use nci::agents::Agent;

use super::ni;

#[test]
fn empty() {
    ni(Agent::Yarn, Vec::new(), "yarn install".to_string());
}

#[test]
fn single_add() {
    ni(
        Agent::Yarn,
        vec!["axios".to_string()],
        "yarn add axios".to_string(),
    );
}

#[test]
fn multiple() {
    ni(
        Agent::Yarn,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "yarn add eslint @types/node".to_string(),
    );
}

#[test]
fn add_dev() {
    ni(
        Agent::Yarn,
        vec!["vite".to_string(), "-D".to_string()],
        "yarn add vite -D".to_string(),
    );
}

#[test]
fn global() {
    ni(
        Agent::Yarn,
        vec!["eslint".to_string(), "-g".to_string()],
        "yarn global add eslint".to_string(),
    );
}

#[test]
fn frozen() {
    ni(
        Agent::Yarn,
        vec!["--frozen".to_string()],
        "yarn install --frozen-lockfile".to_string(),
    );
}
