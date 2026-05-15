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

/// Unit-level tests for the catalog provider in `src/catalog/mod.rs`. These
/// poke the library functions directly, mirroring ni's `test/ni/catalog.spec.ts`.
mod provider {
    use std::fs;

    use nci::catalog::{catalog_ref, detect_pnpm_catalogs};
    use tempfile::TempDir;

    fn named_workspace() -> TempDir {
        let yaml = "packages:\n  - packages/*\n\ncatalogs:\n  prod:\n    react: ^18.3.0\n    express: ^4.21.0\n  dev:\n    typescript: ^5.6.0\n    vitest: ^2.1.0\n";
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();
        dir
    }

    fn default_only_workspace() -> TempDir {
        let yaml = "packages:\n  - packages/*\n\ncatalog:\n  react: ^18.3.0\n  express: ^4.21.0\n";
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();
        dir
    }

    #[test]
    fn catalog_ref_returns_bare_for_default() {
        assert_eq!(catalog_ref("default"), "catalog:");
    }

    #[test]
    fn catalog_ref_includes_name_for_non_default() {
        assert_eq!(catalog_ref("dev"), "catalog:dev");
        assert_eq!(catalog_ref("prod"), "catalog:prod");
    }

    #[test]
    fn detect_named_catalogs() {
        let dir = named_workspace();
        let config = detect_pnpm_catalogs(dir.path()).expect("config");
        assert!(!config.has_default_catalog);
        assert!(config.has_named_catalogs);
        assert_eq!(config.catalogs.len(), 2);
        let names: Vec<_> = config.catalogs.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["prod", "dev"]);
    }

    #[test]
    fn detect_default_catalog_only() {
        let dir = default_only_workspace();
        let config = detect_pnpm_catalogs(dir.path()).expect("config");
        assert!(config.has_default_catalog);
        assert!(!config.has_named_catalogs);
        assert_eq!(config.catalogs.len(), 1);
        assert_eq!(config.catalogs[0].name, "default");
    }

    #[test]
    fn detect_walks_up_from_subdirectory() {
        let dir = named_workspace();
        let sub = dir.path().join("packages").join("app");
        fs::create_dir_all(&sub).unwrap();
        let config = detect_pnpm_catalogs(&sub).expect("config");
        assert_eq!(config.catalogs.len(), 2);
    }

    #[test]
    fn find_package_in_named_catalog() {
        let dir = named_workspace();
        let config = detect_pnpm_catalogs(dir.path()).unwrap();
        let info = config.find_package("react").expect("react in catalog");
        assert_eq!(info.name, "prod");

        let info = config.find_package("typescript").expect("typescript");
        assert_eq!(info.name, "dev");
    }

    #[test]
    fn find_package_returns_none_for_unknown() {
        let dir = named_workspace();
        let config = detect_pnpm_catalogs(dir.path()).unwrap();
        assert!(config.find_package("unknown-pkg").is_none());
    }

    #[test]
    fn find_package_in_default_catalog() {
        let dir = default_only_workspace();
        let config = detect_pnpm_catalogs(dir.path()).unwrap();
        let info = config.find_package("react").expect("react");
        assert_eq!(info.name, "default");
    }

    #[test]
    fn detect_returns_none_when_yaml_has_no_catalogs() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "packages:\n  - packages/*\n",
        )
        .unwrap();
        assert!(detect_pnpm_catalogs(dir.path()).is_none());
    }
}
