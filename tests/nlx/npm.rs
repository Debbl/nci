use nci::agents::Agent;

use super::nlx;

#[test]
fn single_uninstall() {
    nlx(
        Agent::Npm,
        vec!["esbuild".to_string()],
        "npx esbuild".to_string(),
    );
}

#[test]
fn multiple() {
    nlx(
        Agent::Npm,
        vec!["esbuild".to_string(), "--version".to_string()],
        "npx esbuild --version".to_string(),
    );
}
