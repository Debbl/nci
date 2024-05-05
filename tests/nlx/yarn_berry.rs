use nci::agents::Agent;

use super::nlx;

#[test]
fn single_uninstall() {
    nlx(
        Agent::YarnBerry,
        vec!["esbuild".to_string()],
        "yarn dlx esbuild".to_string(),
    );
}

#[test]
fn multiple() {
    nlx(
        Agent::YarnBerry,
        vec!["esbuild".to_string(), "--version".to_string()],
        "yarn dlx esbuild --version".to_string(),
    );
}
