use nci::agents::Agent;

use super::nlx;

#[test]
fn single_uninstall() {
    nlx(
        Agent::Bun,
        vec!["esbuild".to_string()],
        "bunx esbuild".to_string(),
    );
}

#[test]
fn multiple() {
    nlx(
        Agent::Bun,
        vec!["esbuild".to_string(), "--version".to_string()],
        "bunx esbuild --version".to_string(),
    );
}
