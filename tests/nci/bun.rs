use nci::agents::Agent;

use super::nci;

#[test]
fn with_lock() {
    nci(Agent::Bun, true, "bun", &["install", "--frozen-lockfile"]);
}

#[test]
fn without_lock() {
    nci(Agent::Bun, false, "bun", &["install"]);
}
