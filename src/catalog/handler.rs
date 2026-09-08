//! Orchestrate catalog-mode installs. Called from `bin/ni.rs` before the
//! normal `parse_ni` flow. When this returns `Some(cmd)`, the caller skips
//! `parse_ni` and uses the catalog-resolved command instead.

use std::path::PathBuf;

use console::style;
use tokio::runtime::Runtime;

use crate::{
    agents::Agent, config::get_catalog, fetch::fetch_latest_version, parse::CommandTuple,
    runner::RunnerContext,
};

use super::{
    catalog_ref,
    package_json::{find_closest_package_json, update_package_json_catalog_refs, DepType, Entry},
    prompt::{prompt_select_catalog_with_previous, CatalogSelection},
    provider_for, CatalogConfig,
};

/// Try to handle `ni <packages…>` as a catalog install. Returns `Some` when
/// catalog mode took ownership; `None` lets the caller fall back to a normal
/// `parse_ni` add.
pub fn handle_catalog_install(
    agent: &Agent,
    args: &[String],
    ctx: Option<&RunnerContext>,
) -> Option<CommandTuple> {
    if !get_catalog() {
        return None;
    }
    run_catalog_install(
        agent,
        args,
        ctx,
        prompt_select_catalog_with_previous,
        fetch_latest,
    )
}

fn run_catalog_install(
    agent: &Agent,
    args: &[String],
    ctx: Option<&RunnerContext>,
    mut select: impl FnMut(
        &CatalogConfig,
        &str,
        bool,
        Option<&Option<String>>,
        bool,
    ) -> CatalogSelection,
    mut latest: impl FnMut(&str) -> Result<String, String>,
) -> Option<CommandTuple> {
    let provider = provider_for(agent)?;

    let has_workspace_flag = args.iter().any(|a| a == "-w" || a == "--workspace");
    let clean: Vec<String> = args
        .iter()
        .filter(|a| *a != "-w" && *a != "--workspace")
        .cloned()
        .collect();
    let (packages, flags) = split_packages_and_flags(&clean);
    if packages.is_empty() {
        return None;
    }

    let cwd = ctx
        .map(|c| c.cwd.clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let mut config = provider(&cwd)?;
    let dep_type = dep_type_from_flags(&flags);
    let programmatic = ctx.map(|c| c.programmatic).unwrap_or(false);

    let mut catalog_entries: Vec<(String, String)> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    let mut previous: Option<Option<String>> = None;
    let mut apply_to_rest: Option<Option<String>> = None;
    for (index, pkg) in packages.iter().enumerate() {
        if let Some(existing) = config.find_package(pkg) {
            let name = existing.name.clone();
            if !programmatic {
                println!(
                    "{} {} {}",
                    style("\u{2713}").green(),
                    style(pkg).cyan(),
                    style(format!("→ found in {} catalog", name)).dim()
                );
            }
            catalog_entries.push((pkg.clone(), catalog_ref(&name)));
            continue;
        }
        let target_name = if let Some(target) = &apply_to_rest {
            target.clone()
        } else {
            let has_remaining = packages[index + 1..]
                .iter()
                .any(|p| config.find_package(p).is_none());
            let selection = select(&config, pkg, programmatic, previous.as_ref(), has_remaining);
            if !programmatic {
                previous = Some(selection.catalog_name.clone());
                if selection.apply_to_rest {
                    apply_to_rest = Some(selection.catalog_name.clone());
                }
            }
            selection.catalog_name
        };
        let Some(target_name) = target_name else {
            skipped.push(pkg.clone());
            continue;
        };
        // Fetch latest from npm and persist in the provider's catalog file.
        let version = match latest(pkg) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "{} couldn't resolve latest version for {}: {}",
                    style("\u{2717}").red(),
                    pkg,
                    e
                );
                skipped.push(pkg.clone());
                continue;
            }
        };
        if let Err(e) = config.add_package(&target_name, pkg, &version) {
            eprintln!(
                "{} failed to update {}: {}",
                style("\u{2717}").red(),
                config.file_path.display(),
                e
            );
            skipped.push(pkg.clone());
            continue;
        }
        if !programmatic {
            println!(
                "{} {} {}",
                style("+").green(),
                style(pkg).cyan(),
                style(format!("→ {} catalog ({})", target_name, version)).dim()
            );
        }
        catalog_entries.push((pkg.clone(), catalog_ref(&target_name)));
    }

    if catalog_entries.is_empty() {
        return None;
    }

    let pkg_json_path = if has_workspace_flag {
        config
            .file_path
            .parent()
            .map(|p| p.join("package.json"))
            .unwrap_or_else(|| PathBuf::from("package.json"))
    } else {
        find_closest_package_json(&cwd)?
    };

    let entries: Vec<Entry<'_>> = catalog_entries
        .iter()
        .map(|(name, r)| Entry {
            name,
            catalog_ref: r,
        })
        .collect();
    if let Err(e) = update_package_json_catalog_refs(&pkg_json_path, &entries, dep_type) {
        eprintln!(
            "{} failed to update {}: {}",
            style("\u{2717}").red(),
            pkg_json_path.display(),
            e
        );
        return None;
    }

    // If any package was skipped, add it normally; otherwise just install.
    use crate::agents::AgentCommand;
    if !skipped.is_empty() {
        let mut add_args = skipped;
        add_args.extend(flags.into_iter().map(|f| {
            if *agent == Agent::Bun && f == "-D" {
                "-d".into()
            } else {
                f
            }
        }));
        return resolve(agent, AgentCommand::Add, add_args);
    }
    resolve(agent, AgentCommand::Install, Vec::new())
}

fn split_packages_and_flags(args: &[String]) -> (Vec<String>, Vec<String>) {
    let mut pkgs = Vec::new();
    let mut flags = Vec::new();
    for a in args {
        if a.starts_with('-') {
            flags.push(a.clone());
        } else {
            pkgs.push(a.clone());
        }
    }
    (pkgs, flags)
}

fn dep_type_from_flags(flags: &[String]) -> DepType {
    if flags.iter().any(|f| f == "-D" || f == "-d") {
        DepType::DevDependencies
    } else if flags.iter().any(|f| f == "--save-peer") {
        DepType::PeerDependencies
    } else {
        DepType::Dependencies
    }
}

/// Resolve a normal `<agent> <command>` for use after the catalog path has
/// already mutated package.json. We don't go through `parse_ni` here because
/// the args list has already been cleaned up (no `-g`, no catalog packages).
fn resolve(
    agent: &Agent,
    command: crate::agents::AgentCommand,
    args: Vec<String>,
) -> Option<CommandTuple> {
    let value = agent.commands().get(command);
    crate::parse::construct(value, &args)
}

fn fetch_latest(pkg: &str) -> Result<String, String> {
    Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(fetch_latest_version(pkg))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn batch_catalog_selection_reuses_previous_and_applies_to_remaining() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "catalogs:\n  prod:\n    react: ^18\n  dev: {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let ctx = RunnerContext {
            cwd: dir.path().into(),
            programmatic: false,
            has_lock: true,
        };
        let args = ["react", "lodash", "axios", "dayjs"].map(String::from);
        let mut calls = 0;
        let command = run_catalog_install(
            &Agent::Pnpm,
            &args,
            Some(&ctx),
            |_, pkg, _, previous, remaining| {
                calls += 1;
                if calls == 1 {
                    assert_eq!(pkg, "lodash");
                    assert_eq!(previous, None); // Existing react must not set previous.
                } else {
                    assert_eq!(pkg, "axios");
                    assert_eq!(previous, Some(&Some("dev".into())));
                }
                assert!(remaining);
                CatalogSelection {
                    catalog_name: Some("dev".into()),
                    apply_to_rest: calls == 2,
                }
            },
            |_| Ok("^1.0.0".into()),
        )
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(command, ("pnpm".into(), vec!["i".into()]));
        let pkg: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("package.json")).unwrap())
                .unwrap();
        assert_eq!(pkg["dependencies"]["react"], "catalog:prod");
        for name in ["lodash", "axios", "dayjs"] {
            assert_eq!(pkg["dependencies"][name], "catalog:dev");
            let config = super::super::detect_pnpm_catalogs(dir.path()).unwrap();
            assert_eq!(config.find_package(name).unwrap().packages[name], "^1.0.0");
        }
    }

    #[test]
    fn batch_can_reuse_skip_without_affecting_existing_catalog_packages() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "catalogs:\n  prod:\n    react: ^18\n",
        )
        .unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let ctx = RunnerContext {
            cwd: dir.path().into(),
            programmatic: false,
            has_lock: true,
        };
        let args = ["one", "two", "three", "react"].map(String::from);
        let mut calls = 0;
        let command = run_catalog_install(
            &Agent::Pnpm,
            &args,
            Some(&ctx),
            |_, _, _, previous, remaining| {
                calls += 1;
                if calls == 2 {
                    assert_eq!(previous, Some(&None));
                }
                assert!(remaining);
                CatalogSelection {
                    catalog_name: None,
                    apply_to_rest: calls == 2,
                }
            },
            |_| panic!("skipped packages must not fetch versions"),
        )
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(
            command,
            (
                "pnpm".into(),
                ["add", "one", "two", "three"].map(String::from).to_vec()
            )
        );
    }

    #[test]
    fn last_unknown_package_has_no_apply_to_remaining_shortcut() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "catalogs:\n  prod:\n    react: ^18\n",
        )
        .unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let ctx = RunnerContext {
            cwd: dir.path().into(),
            programmatic: false,
            has_lock: true,
        };
        run_catalog_install(
            &Agent::Pnpm,
            &["new".into(), "react".into()],
            Some(&ctx),
            |_, _, _, _, remaining| {
                assert!(!remaining);
                CatalogSelection::default()
            },
            |_| panic!("skipped package"),
        )
        .unwrap();
    }
}
