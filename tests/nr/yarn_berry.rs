use nci::agents::Agent;

use super::nr;

#[test]
fn empty() {
    nr(Agent::YarnBerry, Vec::new(), "yarn run start".to_string())
}

#[test]
fn if_present() {
    nr(
        Agent::YarnBerry,
        vec!["test".to_string(), "--if-present".to_string()],
        "yarn run --if-present test".to_string(),
    )
}

#[test]
fn script() {
    nr(
        Agent::YarnBerry,
        vec!["dev".to_string()],
        "yarn run dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::YarnBerry,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "yarn run build --watch -o".to_string(),
    )
}

#[test]
fn colon() {
    nr(
        Agent::YarnBerry,
        vec!["build:dev".to_string()],
        "yarn run build:dev".to_string(),
    )
}
