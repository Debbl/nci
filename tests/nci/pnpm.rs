use nci::agents::Agent;

use super::nci;

#[test]
fn with_lock() {
    nci(Agent::Pnpm, true, "pnpm", &["i", "--frozen-lockfile"]);
}

#[test]
fn without_lock() {
    nci(Agent::Pnpm, false, "pnpm", &["i"]);
}
