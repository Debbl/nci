use nci::agents::Agent;

use super::nr;

#[test]
fn empty() {
    nr(Agent::Npm, Vec::new(), "npm run start".to_string())
}

#[test]
fn if_present() {
    nr(
        Agent::Npm,
        vec!["test".to_string(), "--if-present".to_string()],
        "npm run --if-present test".to_string(),
    )
}

#[test]
fn script() {
    nr(
        Agent::Npm,
        vec!["dev".to_string()],
        "npm run dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::Npm,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "npm run build -- --watch -o".to_string(),
    )
}

#[test]
fn colon() {
    nr(
        Agent::Npm,
        vec!["build:dev".to_string()],
        "npm run build:dev".to_string(),
    )
}
