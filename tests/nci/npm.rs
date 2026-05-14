use nci::agents::Agent;

use super::nci;

#[test]
fn with_lock() {
    nci(Agent::Npm, true, "npm", &["ci"]);
}

#[test]
fn without_lock() {
    nci(Agent::Npm, false, "npm", &["i"]);
}
