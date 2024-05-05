use nci::agents::Agent;

use super::nu;

#[test]
fn empty() {
    nu(Agent::Yarn, Vec::new(), "yarn upgrade".to_string());
}

#[test]
fn interactive() {
    nu(
        Agent::Yarn,
        vec!["-i".to_string()],
        "yarn upgrade-interactive".to_string(),
    );
}

#[test]
fn interactive_latest() {
    nu(
        Agent::Yarn,
        vec!["-i".to_string(), "--latest".to_string()],
        "yarn upgrade-interactive --latest".to_string(),
    );
}
