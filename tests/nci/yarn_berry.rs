use nci::agents::Agent;

use super::nci;

#[test]
fn with_lock() {
    nci(Agent::YarnBerry, true, "yarn", &["install", "--immutable"]);
}

#[test]
fn without_lock() {
    nci(Agent::YarnBerry, false, "yarn", &["install"]);
}
