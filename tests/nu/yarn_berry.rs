use nci::agents::Agent;

use super::nu;

#[test]
fn empty() {
    nu(Agent::YarnBerry, Vec::new(), "yarn up".to_string());
}

#[test]
fn interactive() {
    nu(
        Agent::YarnBerry,
        vec!["-i".to_string()],
        "yarn up -i".to_string(),
    );
}
