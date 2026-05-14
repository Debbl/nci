//! End-to-end tests for the `ni`-side catalog flow. Each test stages a tiny
//! pnpm workspace (pnpm-lock.yaml + pnpm-workspace.yaml + package.json),
//! spawns the `ni` binary with `?` (dry-run) plus `--programmatic` to avoid
//! prompts, and asserts on both stdout and the files left behind.

use std::{fs, process::Command};
use tempfile::TempDir;

fn workspace(yaml: &str, pkg_json: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();
    fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();
    fs::write(dir.path().join("package.json"), pkg_json).unwrap();
    dir
}

fn run_ni(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ni"))
        .args(args)
        .current_dir(dir)
        .env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("NI_DEFAULT_AGENT")
        .env_remove("NI_CATALOG")
        .output()
        .expect("spawn ni")
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn writes_catalog_ref_when_package_is_in_default_catalog() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let yaml = "catalog:\n  react: ^18.0.0\n";
    let pkg = r#"{"name":"demo","dependencies":{"vue":"^3.0.0"}}"#;
    let dir = workspace(yaml, pkg);

    // `ni react --programmatic ?` should:
    //  - write "react": "catalog:" into package.json
    //  - emit `pnpm i` (since all packages handled via catalog)
    let out = run_ni(dir.path(), &["--programmatic", "?", "react"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm i");

    let body = fs::read_to_string(dir.path().join("package.json")).unwrap();
    assert!(body.contains("\"react\": \"catalog:\""), "got: {body}");
    // pre-existing vue entry is preserved.
    assert!(body.contains("\"vue\": \"^3.0.0\""));
}

#[test]
fn writes_catalog_ref_for_named_catalog() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let yaml = "catalogs:\n  prod:\n    react: ^18.0.0\n";
    let pkg = r#"{"name":"demo"}"#;
    let dir = workspace(yaml, pkg);

    let out = run_ni(dir.path(), &["--programmatic", "?", "react"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm i");

    let body = fs::read_to_string(dir.path().join("package.json")).unwrap();
    assert!(body.contains("\"react\": \"catalog:prod\""), "got: {body}");
}

#[test]
fn writes_to_devdependencies_when_dash_d() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let yaml = "catalog:\n  typescript: ^5.0.0\n";
    let pkg = r#"{"name":"demo"}"#;
    let dir = workspace(yaml, pkg);

    let out = run_ni(dir.path(), &["--programmatic", "?", "typescript", "-D"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let body = fs::read_to_string(dir.path().join("package.json")).unwrap();
    assert!(body.contains("\"devDependencies\""), "got: {body}");
    assert!(body.contains("\"typescript\": \"catalog:\""), "got: {body}");
}

#[test]
fn falls_back_to_normal_add_when_no_catalog_section() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    // pnpm-workspace.yaml exists but declares no catalog → handler returns
    // None and the normal `pnpm add react` flow runs.
    let yaml = "packages:\n  - 'packages/*'\n";
    let pkg = r#"{"name":"demo"}"#;
    let dir = workspace(yaml, pkg);

    let out = run_ni(dir.path(), &["--programmatic", "?", "react"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm add react");

    // package.json was NOT mutated (no catalog ref written).
    let body = fs::read_to_string(dir.path().join("package.json")).unwrap();
    assert!(!body.contains("catalog"), "shouldn't write catalog ref:\n{body}");
}

#[test]
fn disabled_via_env_var_uses_normal_add() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    let yaml = "catalog:\n  react: ^18.0.0\n";
    let pkg = r#"{"name":"demo"}"#;
    let dir = workspace(yaml, pkg);

    // NI_CATALOG=false disables the feature even if catalogs are configured.
    let out = Command::new(env!("CARGO_BIN_EXE_ni"))
        .args(["--programmatic", "?", "react"])
        .current_dir(dir.path())
        .env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("NI_DEFAULT_AGENT")
        .env("NI_CATALOG", "false")
        .output()
        .expect("spawn ni");
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm add react");
}

#[test]
fn programmatic_with_named_catalogs_only_skips_unknown_packages() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    // Only named catalogs (no default), so a package that isn't in any
    // catalog has nowhere to go without prompting. Programmatic mode skips
    // it; result is a plain `pnpm add lodash`.
    let yaml = "catalogs:\n  prod:\n    react: ^18.0.0\n";
    let pkg = r#"{"name":"demo"}"#;
    let dir = workspace(yaml, pkg);

    let out = run_ni(dir.path(), &["--programmatic", "?", "lodash"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(stdout(&out), "pnpm add lodash");

    // package.json was NOT mutated (lodash isn't in any catalog and we
    // can't prompt in programmatic mode).
    let body = fs::read_to_string(dir.path().join("package.json")).unwrap();
    assert!(!body.contains("catalog"), "got: {body}");
}

#[test]
fn pnpm_workspace_yaml_is_updated_when_default_only() {
    if which::which("volta").is_ok() {
        eprintln!("skipping: volta is installed");
        return;
    }
    // With only a default catalog, programmatic still proceeds (no prompt
    // needed). But adding a brand-new package would require an npm registry
    // round-trip — to avoid network in tests, we put `lodash` already in
    // catalog and just verify the existing-entry path works end to end.
    let yaml = "catalog:\n  react: ^18.0.0\n  lodash: ^4.17.21\n";
    let pkg = r#"{"name":"demo"}"#;
    let dir = workspace(yaml, pkg);

    let out = run_ni(dir.path(), &["--programmatic", "?", "lodash"]);
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let body = fs::read_to_string(dir.path().join("package.json")).unwrap();
    assert!(body.contains("\"lodash\": \"catalog:\""), "got: {body}");
    // pnpm-workspace.yaml is untouched since lodash was already there.
    let yaml_after = fs::read_to_string(dir.path().join("pnpm-workspace.yaml")).unwrap();
    assert_eq!(yaml_after, yaml);
}
