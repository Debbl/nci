use nci::agents::Agent;

use super::nd;

#[test]
fn empty() {
    nd(Agent::Pnpm, Vec::new(), "pnpm dedupe".to_string());
}

#[test]
fn check_flag() {
    nd(
        Agent::Pnpm,
        vec!["-c".to_string()],
        "pnpm dedupe --check".to_string(),
    );
}
