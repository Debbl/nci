use nci::agents::Agent;

use super::nlx;

#[test]
fn single() {
    nlx(
        Agent::Deno,
        vec!["esbuild".to_string()],
        "deno x esbuild".to_string(),
    );
}

#[test]
fn multiple() {
    nlx(
        Agent::Deno,
        vec!["esbuild".to_string(), "--version".to_string()],
        "deno x esbuild --version".to_string(),
    );
}
