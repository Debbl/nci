use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli(binary: &str, cwd: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .current_dir(cwd)
        .args(args)
        .env("NI_CONFIG_FILE", cwd.join("missing.nirc"))
        .env("NI_DEFAULT_AGENT", "npm")
        .env("NI_USE_SFW", "false")
        .env("NI_CATALOG", "true")
        .env_remove("NI_RUN_AGENT")
        .env("TMPDIR", cwd)
        .env("PATH", "")
        .output()
        .unwrap()
}

fn assert_output(output: Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
}

#[test]
fn package_manager_version_ranges_do_not_panic() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"packageManager":"pnpm@^10.0.0"}"#,
    )
    .unwrap();
    assert_output(
        cli(env!("CARGO_BIN_EXE_ni"), dir.path(), &["--agent"]),
        "pnpm",
    );
}

#[test]
fn nr_programmatic_defaults_to_start() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    assert_output(
        cli(
            env!("CARGO_BIN_EXE_nr"),
            dir.path(),
            &["--programmatic", "?"],
        ),
        "npm run start",
    );
}

#[test]
fn nearest_directory_wins_over_parent_lock_type() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("bun.lock"), "").unwrap();
    let child = dir.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("package-lock.json"), "").unwrap();
    assert_output(cli(env!("CARGO_BIN_EXE_ni"), &child, &["--agent"]), "npm");
}

#[test]
fn detects_new_markers_and_package_metadata() {
    for (file, contents, agent) in [
        ("deno.lock", "", "deno"),
        ("pnpm-workspace.yaml", "", "pnpm"),
        ("aube-lock.yaml", "", "aube"),
        ("aube-workspace.yaml", "", "aube"),
        ("nub.lock", "", "nub"),
        ("rush.json", "{}", "pnpm-rush"),
        (
            "package.json",
            r#"{"devEngines":{"packageManager":{"name":"pnpm","version":">=6.0.0"}}}"#,
            "pnpm@6",
        ),
        (
            "package.json",
            r#"{"packageManager":"yarn@~4.0.0"}"#,
            "yarn@berry",
        ),
        ("package.json", r#"{"packageManager":"pnpm"}"#, "pnpm"),
        ("package.json", r#"{"packageManager":"aube@1.0.0"}"#, "aube"),
        ("package.json", r#"{"packageManager":"nub@1.0.0"}"#, "nub"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(file), contents).unwrap();
        assert_output(
            cli(env!("CARGO_BIN_EXE_ni"), dir.path(), &["--agent"]),
            agent,
        );
    }
}

#[test]
fn malformed_metadata_falls_back_without_panicking() {
    for contents in [
        "{",
        r#"{"packageManager":"unknown@1"}"#,
        r#"{"scripts":5,"packageManager":"npm@10"}"#,
    ] {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), contents).unwrap();
        fs::write(dir.path().join("package-lock.json"), "").unwrap();
        assert_output(
            cli(env!("CARGO_BIN_EXE_ni"), dir.path(), &["--agent"]),
            "npm",
        );
    }
}

#[test]
fn local_package_manager_beats_parent_lock_and_deno_config_beats_both() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("bun.lock"), "").unwrap();
    let child = dir.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(
        child.join("package.json"),
        r#"{"packageManager":"pnpm@10"}"#,
    )
    .unwrap();
    assert_output(cli(env!("CARGO_BIN_EXE_ni"), &child, &["--agent"]), "pnpm");
    fs::write(child.join("deno.json"), "{}").unwrap();
    assert_output(cli(env!("CARGO_BIN_EXE_ni"), &child, &["--agent"]), "deno");
}

#[test]
fn nlx_local_uses_each_agents_local_command() {
    for (agent, expected) in [
        ("npm@10", "npx vite --help"),
        ("pnpm@10", "pnpm exec vite --help"),
        ("pnpm@6", "pnpm exec vite --help"),
        ("yarn@1", "yarn exec vite -- --help"),
        ("yarn@4", "yarn exec vite --help"),
        ("bun@1", "bun x vite --help"),
        ("deno@2", "deno task --eval vite --help"),
        ("aube@1", "aube exec vite --help"),
        ("nub@1", "nub exec vite --help"),
        ("pnpm-rush@1", "rush-pnpm exec vite --help"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            format!(r#"{{"packageManager":"{agent}"}}"#),
        )
        .unwrap();
        assert_output(
            cli(
                env!("CARGO_BIN_EXE_nlx"),
                dir.path(),
                &["--programmatic", "?", "--local", "vite", "--help"],
            ),
            expected,
        );
    }
}

#[test]
fn new_agents_cover_all_command_shapes() {
    use nci::{
        agents::{Agent, AgentCommand},
        parse::construct,
    };
    for (agent, command, expected) in [
        (Agent::Aube, AgentCommand::Install, "aube install pkg"),
        (
            Agent::Aube,
            AgentCommand::Frozen,
            "aube install --frozen-lockfile pkg",
        ),
        (Agent::Aube, AgentCommand::Global, "aube add -g pkg"),
        (Agent::Aube, AgentCommand::Add, "aube add pkg"),
        (Agent::Aube, AgentCommand::Upgrade, "aube update pkg"),
        (
            Agent::Aube,
            AgentCommand::UpgradeInteractive,
            "aube update -i pkg",
        ),
        (Agent::Aube, AgentCommand::Dedupe, "aube dedupe pkg"),
        (Agent::Aube, AgentCommand::Execute, "aube dlx pkg"),
        (Agent::Aube, AgentCommand::Run, "aube run pkg"),
        (Agent::Aube, AgentCommand::Uninstall, "aube remove pkg"),
        (
            Agent::Aube,
            AgentCommand::GlobalUninstall,
            "aube remove -g pkg",
        ),
        (Agent::Nub, AgentCommand::Execute, "nubx pkg"),
        (
            Agent::Nub,
            AgentCommand::Frozen,
            "nub install --frozen-lockfile pkg",
        ),
        (
            Agent::Nub,
            AgentCommand::GlobalUninstall,
            "nub remove -g pkg",
        ),
        (Agent::PnpmRush, AgentCommand::Run, "rush-pnpm run pkg"),
        (Agent::PnpmRush, AgentCommand::Install, "rush-pnpm i pkg"),
        (
            Agent::PnpmRush,
            AgentCommand::Frozen,
            "rush-pnpm i --frozen-lockfile pkg",
        ),
        (
            Agent::PnpmRush,
            AgentCommand::GlobalUninstall,
            "rush-pnpm remove --global pkg",
        ),
    ] {
        let (cmd, args) = construct(agent.commands().get(command), &["pkg".into()]).unwrap();
        assert_eq!(format!("{cmd} {}", args.join(" ")), expected);
    }
    let (cmd, args) = nci::parse::parse_nd(Agent::Aube, vec!["-c".into()], None);
    assert_eq!(format!("{cmd} {}", args.join(" ")), "aube dedupe --check");
}

#[test]
fn nr_preserves_forwarded_port_and_handles_leading_workspace_flags() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    assert_output(
        cli(
            env!("CARGO_BIN_EXE_nr"),
            dir.path(),
            &["--programmatic", "?", "dev", "-p", "3000"],
        ),
        "npm run dev -- -p 3000",
    );
    assert_output(
        cli(
            env!("CARGO_BIN_EXE_nr"),
            dir.path(),
            &["--programmatic", "?", "-w", "app", "dev", "--watch"],
        ),
        "npm run -w=app dev -- --watch",
    );
}

#[test]
fn dry_run_quotes_spaced_arguments() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("package-lock.json"), "").unwrap();
    assert_output(
        cli(
            env!("CARGO_BIN_EXE_na"),
            dir.path(),
            &["--programmatic", "?", "run", "test", "a b"],
        ),
        "npm run test \"a b\"",
    );
}

#[cfg(unix)]
#[test]
fn auto_install_env_installs_scoped_aube_package_without_prompting() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"packageManager":"aube@1.2.3"}"#,
    )
    .unwrap();
    let bin = dir.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let npm = bin.join("npm");
    // Stub the installer; never access the registry or install a real package.
    fs::write(&npm, "#!/bin/sh\nprintf '%s\\n' \"$@\" > install-args\n").unwrap();
    fs::set_permissions(&npm, fs::Permissions::from_mode(0o755)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ni"))
        .current_dir(dir.path())
        .arg("?")
        .env("PATH", &bin)
        .env("NI_AUTO_INSTALL", "true")
        .env("NI_CONFIG_FILE", dir.path().join("missing.nirc"))
        .env("NI_USE_SFW", "false")
        .env("NI_CATALOG", "false")
        .env("CI", "true")
        .output()
        .unwrap();
    assert_output(output, "aube install");
    assert_eq!(
        fs::read_to_string(dir.path().join("install-args")).unwrap(),
        "i\n-g\n@endevco/aube@1.2.3\n"
    );
}

#[test]
fn yarn_and_bun_catalogs_write_refs_to_child_or_workspace_root() {
    for (agent, config_file, config) in [
        (
            "yarn@4",
            ".yarnrc.yml",
            "nodeLinker: node-modules\ncatalogs:\n  prod:\n    react: ^18.0.0\n",
        ),
        (
            "bun@1",
            "package.json",
            r#"{"name":"root","packageManager":"bun@1","workspaces":{"packages":["packages/*"],"catalogs":{"prod":{"react":"^18.0.0"}}}}"#,
        ),
        (
            "bun@1",
            "package.json",
            r#"{"name":"root","packageManager":"bun@1","workspaces":["packages/*"],"catalogs":{"prod":{"react":"^18.0.0"}}}"#,
        ),
    ] {
        for root in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            fs::write(
                dir.path().join("package.json"),
                format!(r#"{{"name":"root","packageManager":"{agent}"}}"#),
            )
            .unwrap();
            fs::write(dir.path().join(config_file), config).unwrap();
            let child = dir.path().join("packages/app");
            fs::create_dir_all(&child).unwrap();
            fs::write(child.join("package.json"), r#"{"name":"app"}"#).unwrap();
            let mut args = vec!["--programmatic", "?", "react", "-D"];
            if root {
                args.push("-w");
            }
            assert_output(
                cli(env!("CARGO_BIN_EXE_ni"), &child, &args),
                if agent.starts_with("bun") {
                    "bun install"
                } else {
                    "yarn install"
                },
            );
            let target = if root { dir.path() } else { &child };
            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(target.join("package.json")).unwrap())
                    .unwrap();
            assert_eq!(json["devDependencies"]["react"], "catalog:prod");
        }
    }
}

#[test]
fn bun_catalog_mixed_install_rewrites_dev_flag() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"packageManager":"bun@1","workspaces":[],"catalogs":{"prod":{"react":"^18"}}}"#,
    )
    .unwrap();
    assert_output(
        cli(
            env!("CARGO_BIN_EXE_ni"),
            dir.path(),
            &["--programmatic", "?", "react", "unknown-package", "-D"],
        ),
        "bun add unknown-package -d",
    );
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("package.json")).unwrap())
            .unwrap();
    assert_eq!(json["devDependencies"]["react"], "catalog:prod");
    assert!(json["devDependencies"].get("unknown-package").is_none());
}

#[test]
fn catalog_providers_persist_new_packages_and_preserve_location() {
    use nci::{agents::Agent, catalog::provider_for};
    for (agent, file, content, nested) in [
        (Agent::YarnBerry, ".yarnrc.yml", "# preserve this comment\nnodeLinker: node-modules\ncatalogs:\n  prod:\n    react: ^18\n", false),
        (Agent::Bun, "package.json", "{\n\t\"workspaces\": {\"packages\": [], \"catalogs\": {\"prod\": {\"react\": \"^18\"}}},\n\t\"scripts\": {\"build\": \"tsc\"}\n}\n", true),
        (Agent::Bun, "package.json", "{\n    \"workspaces\": [],\n    \"catalog\": {\"react\": \"^18\"}\n}\n", false),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(file);
        fs::write(&path, content).unwrap();
        let provider = provider_for(&agent).unwrap();
        let mut config = provider(dir.path()).unwrap();
        config.add_package("prod", "@types/node", "^22.0.0").unwrap();
        config.add_package("dev", "typescript", "^5.0.0").unwrap();
        config.add_package("default", "vue", "^3.0.0").unwrap();
        let config = provider(dir.path()).unwrap();
        assert_eq!(config.find_package("@types/node").unwrap().name, "prod");
        assert_eq!(config.find_package("typescript").unwrap().name, "dev");
        assert_eq!(config.find_package("vue").unwrap().name, "default");
        let result = fs::read_to_string(path).unwrap();
        if agent == Agent::YarnBerry {
            assert!(result.contains("# preserve this comment"));
            assert!(result.contains("nodeLinker: node-modules"));
        } else {
            let json: serde_json::Value = serde_json::from_str(&result).unwrap();
            let target = if nested { &json["workspaces"] } else { &json };
            assert_eq!(target["catalogs"]["prod"]["@types/node"], "^22.0.0");
            assert_eq!(target["catalog"]["vue"], "^3.0.0");
            if nested {
                assert!(json.get("catalogs").is_none());
                assert_eq!(json["scripts"]["build"], "tsc");
                assert!(result.contains("\n\t\"workspaces\""));
            } else {
                assert!(result.contains("\n    \"workspaces\""));
            }
        }
    }
}

#[test]
fn bun_catalog_detection_respects_workspace_boundary_and_nested_precedence() {
    use nci::catalog::detect_bun_catalogs;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("package.json");
    fs::write(&file, r#"{"catalog":{"react":"^18"}}"#).unwrap();
    assert!(detect_bun_catalogs(dir.path()).is_none());
    fs::write(
        &file,
        r#"{"workspaces":{"catalogs":{"dev":{}}},"catalog":{"react":"^18"}}"#,
    )
    .unwrap();
    let config = detect_bun_catalogs(dir.path()).unwrap();
    assert_eq!(config.catalogs[0].name, "dev");
    assert!(config.find_package("react").is_none());
    let child = dir.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("package.json"), r#"{"workspaces":[]}"#).unwrap();
    assert!(detect_bun_catalogs(&child).is_none());
}
