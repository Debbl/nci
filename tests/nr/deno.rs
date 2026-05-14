use nci::agents::Agent;

use super::nr;

#[test]
fn empty() {
    nr(Agent::Deno, Vec::new(), "deno task start".to_string())
}

#[test]
fn if_present() {
    nr(
        Agent::Deno,
        vec!["test".to_string(), "--if-present".to_string()],
        "deno task --if-present test".to_string(),
    )
}

#[test]
fn script() {
    nr(
        Agent::Deno,
        vec!["dev".to_string()],
        "deno task dev".to_string(),
    )
}

#[test]
fn script_with_arguments() {
    nr(
        Agent::Deno,
        vec!["build".to_string(), "--watch".to_string(), "-o".to_string()],
        "deno task build --watch -o".to_string(),
    )
}
