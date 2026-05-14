use nci::agents::Agent;

use super::nci;

#[test]
fn with_lock() {
    nci(Agent::Deno, true, "deno", &["install", "--frozen"]);
}

#[test]
fn without_lock() {
    nci(Agent::Deno, false, "deno", &["install"]);
}
