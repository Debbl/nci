use nci::agents::Agent;

use super::nci;

#[test]
fn with_lock() {
    nci(
        Agent::Yarn,
        true,
        "yarn",
        &["install", "--frozen-lockfile"],
    );
}

#[test]
fn without_lock() {
    nci(Agent::Yarn, false, "yarn", &["install"]);
}
