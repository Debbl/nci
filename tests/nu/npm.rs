use nci::agents::Agent;

use super::nu;

#[test]
fn empty() {
    nu(Agent::Npm, Vec::new(), "npm update".to_string());
}
