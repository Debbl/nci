use nci::agents::Agent;

use super::nu;

#[test]
fn empty() {
    nu(Agent::Bun, Vec::new(), "bun update".to_string());
}

#[test]
fn interactive() {
    nu(Agent::Bun, vec!["-i".to_string()], "bun update".to_string());
}

#[test]
fn interactive_latest() {
    nu(
        Agent::Bun,
        vec!["-i".to_string(), "--latest".to_string()],
        "bun update --latest".to_string(),
    );
}
