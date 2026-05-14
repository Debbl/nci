//! Orchestrate catalog-mode installs. Called from `bin/ni.rs` before the
//! normal `parse_ni` flow. When this returns `Some(cmd)`, the caller skips
//! `parse_ni` and uses the catalog-resolved command instead.

use std::{fs, path::PathBuf};

use console::style;
use tokio::runtime::Runtime;

use crate::{
    agents::Agent,
    config::get_catalog,
    fetch::fetch_latest_version,
    parse::CommandTuple,
    runner::RunnerContext,
};

use super::{
    catalog_ref,
    package_json::{find_closest_package_json, update_package_json_catalog_refs, DepType, Entry},
    prompt::prompt_select_catalog,
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

    for pkg in &packages {
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
        let Some(target_name) = prompt_select_catalog(&config, pkg, programmatic) else {
            skipped.push(pkg.clone());
            continue;
        };
        // Fetch latest from npm, write into pnpm-workspace.yaml.
        let version = match fetch_latest(pkg) {
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
        if let Err(e) = add_to_workspace(&mut config, &target_name, pkg, &version) {
            eprintln!(
                "{} failed to update pnpm-workspace.yaml: {}",
                style("\u{2717}").red(),
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
        add_args.extend(flags);
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

fn add_to_workspace(
    config: &mut CatalogConfig,
    catalog_name: &str,
    pkg: &str,
    version: &str,
) -> std::io::Result<()> {
    let original = fs::read_to_string(&config.file_path)?;
    let updated = super::yaml::insert_catalog_entry(&original, catalog_name, pkg, version);
    fs::write(&config.file_path, updated)?;

    // Update in-memory mirror.
    if let Some(info) = config
        .catalogs
        .iter_mut()
        .find(|c| c.name == catalog_name)
    {
        info.packages
            .insert(pkg.to_string(), version.to_string());
    } else {
        let mut packages = indexmap::IndexMap::new();
        packages.insert(pkg.to_string(), version.to_string());
        config.catalogs.push(super::CatalogInfo {
            name: catalog_name.to_string(),
            packages,
        });
        if catalog_name == "default" {
            config.has_default_catalog = true;
        } else {
            config.has_named_catalogs = true;
        }
    }
    Ok(())
}
