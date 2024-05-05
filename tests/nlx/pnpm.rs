use nci::agents::Agent;

use super::nlx;

#[test]
fn single_uninstall() {
    nlx(
        Agent::Pnpm,
        vec!["esbuild".to_string()],
        "pnpm dlx esbuild".to_string(),
    );
}

#[test]
fn multiple() {
    nlx(
        Agent::Pnpm,
        vec!["esbuild".to_string(), "--version".to_string()],
        "pnpm dlx esbuild --version".to_string(),
    );
}
