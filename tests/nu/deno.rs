use nci::agents::Agent;

use super::nu;

#[test]
fn empty() {
    nu(
        Agent::Deno,
        Vec::new(),
        "deno outdated --update".to_string(),
    );
}

#[test]
fn interactive() {
    nu(
        Agent::Deno,
        vec!["-i".to_string()],
        "deno outdated --update".to_string(),
    );
}
