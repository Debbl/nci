use std::{fs, process::Command};

use nci::completion::completion_suggestions;
use tempfile::TempDir;

fn write_pkg(scripts: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    let mut entries = String::new();
    for (i, (k, v)) in scripts.iter().enumerate() {
        if i > 0 {
            entries.push(',');
        }
        entries.push_str(&format!(r#""{}":"{}""#, k, v));
    }
    let body = format!(r#"{{"scripts":{{{}}}}}"#, entries);
    fs::write(dir.path().join("package.json"), body).unwrap();
    dir
}

#[test]
fn suggests_all_when_empty_prefix() {
    let dir = write_pkg(&[("dev", "vite"), ("build", "vite build"), ("test", "vitest")]);
    let mut suggestions = completion_suggestions(dir.path(), "");
    suggestions.sort();
    assert_eq!(suggestions, vec!["build", "dev", "test"]);
}

#[test]
fn filters_by_prefix_substring() {
    let dir = write_pkg(&[("dev", "vite"), ("build", "vite build"), ("build:dev", "vite build --dev")]);
    let mut suggestions = completion_suggestions(dir.path(), "build");
    suggestions.sort();
    assert_eq!(suggestions, vec!["build", "build:dev"]);
}

#[test]
fn smart_case_matches() {
    let dir = write_pkg(&[("Dev", "vite"), ("build", "vite build")]);
    // Lowercase needle → case-insensitive (fzf smart-case): matches "Dev".
    assert_eq!(completion_suggestions(dir.path(), "dev"), vec!["Dev"]);
    // Uppercase needle → case-sensitive: "DEV" doesn't match "Dev".
    assert!(completion_suggestions(dir.path(), "DEV").is_empty());
}

#[test]
fn ranks_better_matches_first() {
    // With fuzzy ranking, an exact-substring match outranks one with leading
    // junk, even though both contain the needle.
    let dir = write_pkg(&[
        ("build:dev", "vite build --mode dev"),
        ("dev", "vite"),
        ("test", "vitest"),
    ]);
    let suggestions = completion_suggestions(dir.path(), "dev");
    assert_eq!(suggestions.first().unwrap(), "dev");
}

#[test]
fn skips_question_prefixed_description_keys() {
    // The `?key` convention is a description for `key` (see npm-scripts-info);
    // it shouldn't show up as a runnable script.
    let dir = write_pkg(&[("dev", "vite"), ("?dev", "run the dev server")]);
    let suggestions = completion_suggestions(dir.path(), "");
    assert_eq!(suggestions, vec!["dev"]);
}

#[test]
fn no_scripts_returns_empty() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package.json"), "{}").unwrap();
    let suggestions = completion_suggestions(dir.path(), "");
    assert!(suggestions.is_empty());
}

// --- CLI smoke tests: the static script printers ---

fn run_nr_clean(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nr"))
        .args(args)
        .env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("COMP_LINE")
        .env_remove("COMP_CWORD")
        .output()
        .expect("spawn nr")
}

#[test]
fn bash_script_contains_complete_directive() {
    let out = run_nr_clean(&["--completion-bash"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("complete -F _nr_completion nr"));
}

#[test]
fn zsh_script_contains_compdef() {
    let out = run_nr_clean(&["--completion-zsh"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("#compdef nr"));
}

#[test]
fn fish_script_registers_complete() {
    let out = run_nr_clean(&["--completion-fish"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("complete -c nr -f"));
}

#[test]
fn runtime_completion_zsh_mode() {
    let dir = write_pkg(&[("dev", "vite"), ("build", "vite build")]);
    let out = Command::new(env!("CARGO_BIN_EXE_nr"))
        .args(["--completion", "dev"])
        .current_dir(dir.path())
        .env_remove("CI")
        .env_remove("NI_CONFIG_FILE")
        .env_remove("COMP_LINE")
        .env_remove("COMP_CWORD")
        .output()
        .expect("spawn nr");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(s, "dev");
}
