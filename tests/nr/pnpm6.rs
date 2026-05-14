use nci::agents::Agent;

use super::nr;

// pnpm@6 uses dashDashArg for `run`: extra args go after a `--` separator.

#[test]
fn empty() {
    nr(Agent::Pnpm6, Vec::new(), "pnpm run start".to_string())
}

#[test]
fn script() {
    nr(
        Agent::Pnpm6,
        vec!["dev".to_string()],
        "pnpm run dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::Pnpm6,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "pnpm run build -- --watch -o".to_string(),
    )
}

#[test]
fn if_present() {
    nr(
        Agent::Pnpm6,
        vec!["test".to_string(), "--if-present".to_string()],
        "pnpm run --if-present test".to_string(),
    )
}
