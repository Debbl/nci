use nci::agents::Agent;

use super::ni;

// `-P` is the install-production flag. npm uses `--omit=dev`; everyone else
// gets `--production` (deno will reject it, matching upstream behaviour).

#[test]
fn npm() {
    ni(
        Agent::Npm,
        vec!["-P".to_string()],
        "npm i --omit=dev".to_string(),
    );
}

#[test]
fn yarn() {
    ni(
        Agent::Yarn,
        vec!["-P".to_string()],
        "yarn install --production".to_string(),
    );
}

#[test]
fn yarn_berry() {
    ni(
        Agent::YarnBerry,
        vec!["-P".to_string()],
        "yarn install --production".to_string(),
    );
}

#[test]
fn pnpm() {
    ni(
        Agent::Pnpm,
        vec!["-P".to_string()],
        "pnpm i --production".to_string(),
    );
}

#[test]
fn bun() {
    ni(
        Agent::Bun,
        vec!["-P".to_string()],
        "bun install --production".to_string(),
    );
}
