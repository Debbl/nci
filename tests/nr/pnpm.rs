use nci::agents::Agent;

use super::nr;

#[test]
fn empty() {
    nr(Agent::Pnpm, Vec::new(), "pnpm run start".to_string())
}

#[test]
fn if_present() {
    nr(
        Agent::Pnpm,
        vec!["test".to_string(), "--if-present".to_string()],
        "pnpm run --if-present test".to_string(),
    )
}

#[test]
fn script() {
    nr(
        Agent::Pnpm,
        vec!["dev".to_string()],
        "pnpm run dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::Pnpm,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "pnpm run build --watch -o".to_string(),
    )
}

#[test]
fn colon() {
    nr(
        Agent::Pnpm,
        vec!["build:dev".to_string()],
        "pnpm run build:dev".to_string(),
    )
}
