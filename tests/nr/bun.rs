use nci::agents::Agent;

use super::nr;

#[test]
fn empty() {
    nr(Agent::Bun, Vec::new(), "bun run start".to_string())
}

#[test]
fn script() {
    nr(
        Agent::Bun,
        vec!["dev".to_string()],
        "bun run dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::Bun,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "bun run build --watch -o".to_string(),
    )
}

#[test]
fn colon() {
    nr(
        Agent::Bun,
        vec!["build:dev".to_string()],
        "bun run build:dev".to_string(),
    )
}
