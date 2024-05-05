use nci::agents::Agent;

use super::ni;

#[test]
fn empty() {
    ni(Agent::Bun, Vec::new(), "bun install".to_string());
}

#[test]
fn single_add() {
    ni(
        Agent::Bun,
        vec!["axios".to_string()],
        "bun add axios".to_string(),
    );
}

#[test]
fn add_dev() {
    ni(
        Agent::Bun,
        vec!["vite".to_string(), "-D".to_string()],
        "bun add vite -d".to_string(),
    );
}

#[test]
fn multiple() {
    ni(
        Agent::Bun,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "bun add eslint @types/node".to_string(),
    );
}

#[test]
fn global() {
    ni(
        Agent::Bun,
        vec!["eslint".to_string(), "-g".to_string()],
        "bun add -g eslint".to_string(),
    );
}

#[test]
fn frozen() {
    ni(
        Agent::Bun,
        vec!["--frozen".to_string()],
        "bun install --no-save".to_string(),
    );
}
