use super::ni;

#[test]
fn empty() {
    ni(
        nci::agents::Agent::YarnBerry,
        Vec::new(),
        "yarn install".to_string(),
    );
}

#[test]
fn single_add() {
    ni(
        nci::agents::Agent::YarnBerry,
        vec!["axios".to_string()],
        "yarn add axios".to_string(),
    );
}

#[test]
fn multiple() {
    ni(
        nci::agents::Agent::YarnBerry,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "yarn add eslint @types/node".to_string(),
    );
}

#[test]
fn add_dev() {
    ni(
        nci::agents::Agent::YarnBerry,
        vec!["vite".to_string(), "-D".to_string()],
        "yarn add vite -D".to_string(),
    );
}

#[test]
fn global() {
    ni(
        nci::agents::Agent::YarnBerry,
        vec!["eslint".to_string(), "-g".to_string()],
        "npm i -g eslint".to_string(),
    );
}

#[test]
fn frozen() {
    ni(
        nci::agents::Agent::YarnBerry,
        vec!["--frozen".to_string()],
        "yarn install --immutable".to_string(),
    );
}
