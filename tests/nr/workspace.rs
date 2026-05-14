use nci::agents::Agent;

use super::nr;

// `-w value` / `--workspace value` get merged into `-w=value` /
// `--workspace=value` so npm doesn't treat the flag as boolean true.

#[test]
fn short_pnpm() {
    nr(
        Agent::Pnpm,
        vec!["dev".to_string(), "-w".to_string(), "api".to_string()],
        "pnpm run dev -w=api".to_string(),
    );
}

#[test]
fn long_pnpm() {
    nr(
        Agent::Pnpm,
        vec![
            "dev".to_string(),
            "--workspace".to_string(),
            "api".to_string(),
        ],
        "pnpm run dev --workspace=api".to_string(),
    );
}

#[test]
fn npm_dashdash() {
    // npm uses dashDashArg, so the merged flag still ends up after `--`.
    nr(
        Agent::Npm,
        vec!["dev".to_string(), "-w".to_string(), "api".to_string()],
        "npm run dev -- -w=api".to_string(),
    );
}

#[test]
fn no_value_left_alone() {
    // Last arg or followed by another flag → don't merge.
    nr(
        Agent::Pnpm,
        vec!["dev".to_string(), "-w".to_string()],
        "pnpm run dev -w".to_string(),
    );
}

#[test]
fn flag_value_left_alone() {
    nr(
        Agent::Pnpm,
        vec![
            "dev".to_string(),
            "-w".to_string(),
            "--prod".to_string(),
        ],
        "pnpm run dev -w --prod".to_string(),
    );
}
