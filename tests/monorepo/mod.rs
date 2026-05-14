use std::fs;

use nci::monorepo::{find_packages, load_packages};
use tempfile::TempDir;

fn touch(dir: &std::path::Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

/// Build a workspace with the given package.json paths. Each is given the
/// supplied body (or `{}` if empty).
fn workspace(specs: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (rel, body) in specs {
        let body = if body.is_empty() { "{}" } else { *body };
        touch(dir.path(), rel, body);
    }
    dir
}

#[test]
fn empty_without_root_package_returns_empty() {
    // No root package.json → no monorepo to walk.
    let dir = workspace(&[("packages/foo/package.json", r#"{"name":"foo"}"#)]);
    let pkgs = find_packages(dir.path());
    assert!(pkgs.is_empty());
}

#[test]
fn finds_all_workspace_packages() {
    let dir = workspace(&[
        ("package.json", r#"{"name":"root"}"#),
        ("packages/api/package.json", r#"{"name":"api"}"#),
        ("packages/web/package.json", r#"{"name":"web"}"#),
    ]);
    let mut pkgs = find_packages(dir.path());
    pkgs.sort();
    assert_eq!(pkgs.len(), 3);
    let joined: Vec<String> = pkgs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(joined.iter().any(|p| p == "package.json"));
    assert!(joined.iter().any(|p| p.contains("api")));
    assert!(joined.iter().any(|p| p.contains("web")));
}

#[test]
fn ignores_node_modules_and_dist() {
    let dir = workspace(&[
        ("package.json", r#"{"name":"root"}"#),
        ("packages/api/package.json", r#"{"name":"api"}"#),
        // These should all be skipped.
        ("node_modules/react/package.json", r#"{"name":"react"}"#),
        ("packages/api/node_modules/foo/package.json", r#"{"name":"foo"}"#),
        ("dist/some/package.json", r#"{"name":"dist-pkg"}"#),
        ("public/package.json", r#"{"name":"public-pkg"}"#),
        ("fixtures/case-a/package.json", r#"{"name":"fix"}"#),
        (".git/package.json", r#"{"name":"git-pkg"}"#),
    ]);
    let pkgs = find_packages(dir.path());
    let names: Vec<String> = pkgs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert_eq!(pkgs.len(), 2, "got: {:?}", names);
    assert!(names.iter().all(|n| !n.contains("node_modules")));
    assert!(names.iter().all(|n| !n.contains("dist")));
    assert!(names.iter().all(|n| !n.contains("public")));
    assert!(names.iter().all(|n| !n.contains("fixture")));
    assert!(names.iter().all(|n| !n.contains(".git")));
}

#[test]
fn load_packages_reads_name_and_scripts() {
    let dir = workspace(&[
        ("package.json", r#"{"name":"root"}"#),
        (
            "packages/api/package.json",
            r#"{"name":"@scope/api","scripts":{"dev":"node api.js","build":"tsc"}}"#,
        ),
    ]);
    let pkgs = load_packages(dir.path(), None);
    assert_eq!(pkgs.len(), 2);
    let api = pkgs
        .iter()
        .find(|p| p.name == "@scope/api")
        .expect("api package");
    assert!(api.scripts.contains_key("dev"));
    assert!(api.scripts.contains_key("build"));
    // cwd should be the package directory, not the lockfile path.
    assert!(api.cwd.ends_with("packages/api"));
}

#[test]
fn load_packages_filter_keeps_only_those_with_script() {
    let dir = workspace(&[
        ("package.json", r#"{"name":"root"}"#),
        (
            "packages/api/package.json",
            r#"{"name":"api","scripts":{"dev":"node api.js"}}"#,
        ),
        (
            "packages/web/package.json",
            r#"{"name":"web","scripts":{"build":"vite"}}"#,
        ),
    ]);
    let pkgs = load_packages(dir.path(), Some("dev"));
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "api");
}
