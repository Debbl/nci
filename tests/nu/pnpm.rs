use nci::agents::Agent;

use super::nu;

#[test]
fn empty() {
    nu(Agent::Pnpm, Vec::new(), "pnpm update".to_string());
}

#[test]
fn interactive() {
    nu(
        Agent::Pnpm,
        vec!["-i".to_string()],
        "pnpm update -i".to_string(),
    );
}

#[test]
fn interactive_latest() {
    nu(
        Agent::Pnpm,
        vec!["-i".to_string(), "--latest".to_string()],
        "pnpm update -i --latest".to_string(),
    );
}
