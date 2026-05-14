use nci::agents::Agent;

use super::nd;

#[test]
fn empty() {
    nd(Agent::Npm, Vec::new(), "npm dedupe".to_string());
}

#[test]
fn check_flag_becomes_dry_run() {
    // npm dedupe doesn't have --check; -c is rewritten to --dry-run.
    nd(
        Agent::Npm,
        vec!["-c".to_string()],
        "npm dedupe --dry-run".to_string(),
    );
}
