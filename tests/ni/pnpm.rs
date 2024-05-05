use nci::agents::Agent;

use super::ni;

#[test]
fn empty() {
    ni(Agent::Pnpm, Vec::new(), "pnpm i".to_string());
}

#[test]
fn single_add() {
    ni(
        Agent::Pnpm,
        vec!["axios".to_string()],
        "pnpm add axios".to_string(),
    );
}

#[test]
fn multiple() {
    ni(
        Agent::Pnpm,
        vec!["eslint".to_string(), "@types/node".to_string()],
        "pnpm add eslint @types/node".to_string(),
    );
}

#[test]
fn add_dev() {
    ni(
        Agent::Pnpm,
        vec!["vite".to_string(), "-D".to_string()],
        "pnpm add vite -D".to_string(),
    );
}

#[test]
fn global() {
    ni(
        Agent::Pnpm,
        vec!["eslint".to_string(), "-g".to_string()],
        "pnpm add -g eslint".to_string(),
    );
}

#[test]
fn frozen() {
    ni(
        Agent::Pnpm,
        vec!["--frozen".to_string()],
        "pnpm i --frozen-lockfile".to_string(),
    );
}

#[test]
fn forward1() {
    ni(
        Agent::Pnpm,
        vec!["--anything".to_string()],
        "pnpm i --anything".to_string(),
    );
}
#[test]
fn forward2() {
    ni(Agent::Pnpm, vec!["-a".to_string()], "pnpm i -a".to_string());
}
