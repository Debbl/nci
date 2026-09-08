//! Catalog support for pnpm, Yarn Berry, and Bun workspaces.
//!
//! When a pnpm workspace uses [catalogs] in `pnpm-workspace.yaml`, `ni` should
//! prefer writing `catalog:` references into `package.json` instead of pinning
//! versions. This module mirrors the upstream `src/catalog/` subtree:
//!
//! - [`mod@types`] / this file: data model and YAML-side reads.
//! - [`yaml`]: line-based YAML edits that insert new entries while leaving
//!   the rest of the file alone (comments, blank lines, key order).
//! - [`package_json`]: write `catalog:` refs into the consuming package.json.
//! - [`prompt`]: pick a catalog when multiple are configured.
//! - [`handler`]: glue that the `ni` binary calls before falling through to
//!   the normal install flow.
//!
//! [catalogs]: https://pnpm.io/catalogs

mod bun;
pub mod handler;
pub mod package_json;
pub mod prompt;
pub mod yaml;

pub use bun::detect_bun_catalogs;

use std::{
    fs,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use serde::Deserialize;

use crate::agents::Agent;

#[derive(Debug, Clone)]
pub struct CatalogInfo {
    /// `"default"` for the top-level `catalog:` block, otherwise the name
    /// under `catalogs.<name>`.
    pub name: String,
    pub packages: IndexMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CatalogConfig {
    pub file_path: PathBuf,
    pub catalogs: Vec<CatalogInfo>,
    pub has_default_catalog: bool,
    pub has_named_catalogs: bool,
}

#[derive(Deserialize)]
struct WorkspaceYaml {
    #[serde(default)]
    catalog: Option<IndexMap<String, String>>,
    #[serde(default)]
    catalogs: Option<IndexMap<String, IndexMap<String, String>>>,
}

/// Walk up from `cwd` looking for `pnpm-workspace.yaml`. Returns the absolute
/// path of the first match, or `None`.
pub fn find_pnpm_workspace_yaml(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd.to_path_buf();
    loop {
        let candidate = dir.join("pnpm-workspace.yaml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Return the catalog provider for agents with workspace catalog support.
pub fn provider_for(agent: &Agent) -> Option<fn(&Path) -> Option<CatalogConfig>> {
    match agent {
        Agent::Pnpm | Agent::Pnpm6 => Some(detect_pnpm_catalogs),
        Agent::YarnBerry => Some(detect_yarn_catalogs),
        Agent::Bun => Some(detect_bun_catalogs),
        _ => None,
    }
}

/// Read `pnpm-workspace.yaml` and surface its catalog configuration. Returns
/// `None` if the file is missing, can't be parsed, or has no catalogs.
pub fn detect_pnpm_catalogs(cwd: &Path) -> Option<CatalogConfig> {
    let file_path = find_pnpm_workspace_yaml(cwd)?;
    detect_yaml_catalogs(file_path)
}

pub fn detect_yarn_catalogs(cwd: &Path) -> Option<CatalogConfig> {
    let file_path = crate::detect::find_up(".yarnrc.yml", cwd)?;
    detect_yaml_catalogs(PathBuf::from(file_path))
}

fn detect_yaml_catalogs(file_path: PathBuf) -> Option<CatalogConfig> {
    let content = fs::read_to_string(&file_path).ok()?;
    let ws: WorkspaceYaml = serde_yaml_ng::from_str(&content).ok()?;
    config_from_fields(file_path, ws)
}

fn config_from_fields(file_path: PathBuf, ws: WorkspaceYaml) -> Option<CatalogConfig> {
    let has_default_catalog = ws.catalog.as_ref().is_some_and(|m| !m.is_empty());
    let has_named_catalogs = ws.catalogs.as_ref().is_some_and(|m| !m.is_empty());
    if !has_default_catalog && !has_named_catalogs {
        return None;
    }

    let mut catalogs = Vec::new();
    if let Some(c) = ws.catalog {
        if !c.is_empty() {
            catalogs.push(CatalogInfo {
                name: "default".to_string(),
                packages: c,
            });
        }
    }
    if let Some(named) = ws.catalogs {
        for (name, packages) in named {
            catalogs.push(CatalogInfo { name, packages });
        }
    }

    Some(CatalogConfig {
        file_path,
        catalogs,
        has_default_catalog,
        has_named_catalogs,
    })
}

impl CatalogConfig {
    /// Persist in the provider's original format, then update the in-memory catalog.
    pub fn add_package(&mut self, name: &str, pkg: &str, version: &str) -> std::io::Result<()> {
        let original = fs::read_to_string(&self.file_path)?;
        if self
            .file_path
            .file_name()
            .is_some_and(|n| n == "package.json")
        {
            bun::add_package(&self.file_path, &original, name, pkg, version)?;
        } else {
            let updated = yaml::insert_catalog_entry(&original, name, pkg, version);
            fs::write(&self.file_path, updated)?;
        }
        if let Some(info) = self.catalogs.iter_mut().find(|c| c.name == name) {
            info.packages.insert(pkg.to_string(), version.to_string());
        } else {
            let mut packages = IndexMap::new();
            packages.insert(pkg.to_string(), version.to_string());
            self.catalogs.push(CatalogInfo {
                name: name.to_string(),
                packages,
            });
        }
        if name == "default" {
            self.has_default_catalog = true;
        } else {
            self.has_named_catalogs = true;
        }
        Ok(())
    }

    /// Find which catalog (if any) declares `pkg`.
    pub fn find_package(&self, pkg: &str) -> Option<&CatalogInfo> {
        self.catalogs.iter().find(|c| c.packages.contains_key(pkg))
    }
}

/// `catalog:` for the default catalog, otherwise `catalog:<name>`. This is
/// what gets written into `package.json`.
pub fn catalog_ref(name: &str) -> String {
    if name == "default" {
        "catalog:".to_string()
    } else {
        format!("catalog:{}", name)
    }
}
