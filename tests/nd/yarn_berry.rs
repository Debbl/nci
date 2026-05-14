use nci::agents::Agent;

use super::nd;

#[test]
fn empty() {
    nd(Agent::YarnBerry, Vec::new(), "yarn dedupe".to_string());
}

#[test]
fn passthrough() {
    nd(
        Agent::YarnBerry,
        vec!["--mode".to_string(), "update-lockfile".to_string()],
        "yarn dedupe --mode update-lockfile".to_string(),
    );
}
