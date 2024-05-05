use nci::agents::Agent;

use super::nr;

#[test]
fn empty() {
    nr(Agent::Yarn, Vec::new(), "yarn run start".to_string())
}

#[test]
fn if_present() {
    nr(
        Agent::Yarn,
        vec!["test".to_string(), "--if-present".to_string()],
        "yarn run --if-present test".to_string(),
    )
}

#[test]
fn script() {
    nr(
        Agent::Yarn,
        vec!["dev".to_string()],
        "yarn run dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::Yarn,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "yarn run build --watch -o".to_string(),
    )
}

#[test]
fn colon() {
    nr(
        Agent::Yarn,
        vec!["build:dev".to_string()],
        "yarn run build:dev".to_string(),
    )
}
