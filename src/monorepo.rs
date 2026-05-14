use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::utils::get_package_json;

/// Directory names that are never workspace packages and shouldn't be walked
/// into. Mirrors upstream's tinyglobby ignore list.
const IGNORE_DIRS: &[&str] = &["node_modules", "dist", "public", "fixture", "fixtures"];

/// Locate every `package.json` reachable from `cwd`, skipping common
/// build/vendor directories. Returns paths relative to `cwd`.
pub fn find_packages(cwd: &Path) -> Vec<PathBuf> {
    if !cwd.join("package.json").exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let walker = WalkDir::new(cwd).into_iter().filter_entry(|e| {
        // Don't descend into ignored directory names. Hidden directories
        // (`.git`, `.cache`, …) are skipped too — matches `dot: false`.
        if e.depth() == 0 {
            return true;
        }
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            if name.starts_with('.') {
                return false;
            }
            return !IGNORE_DIRS.iter().any(|n| n == &name.as_ref());
        }
        true
    });
    for entry in walker.flatten() {
        if entry.file_type().is_file() && entry.file_name() == "package.json" {
            if let Ok(rel) = entry.path().strip_prefix(cwd) {
                out.push(rel.to_path_buf());
            }
        }
    }
    out
}

/// One package surfaced by the monorepo picker. `cwd` is absolute so the
/// caller can hand it back to `RunnerContext`.
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub cwd: PathBuf,
    pub description: String,
    pub scripts: indexmap::IndexMap<String, String>,
}

impl std::fmt::Display for PackageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dim = console::style(self.cwd.display()).dim();
        write!(f, "{:<30} {}", self.name, dim)
    }
}

/// Load metadata for every package found under `cwd`. If `filter_cmd` is
/// supplied, drop packages that don't declare that script.
pub fn load_packages(cwd: &Path, filter_cmd: Option<&str>) -> Vec<PackageEntry> {
    let pkg_paths = find_packages(cwd);
    let mut entries: Vec<PackageEntry> = pkg_paths
        .into_iter()
        .map(|rel| {
            let abs = cwd.join(&rel);
            let pkg_dir = abs.parent().unwrap_or(cwd).to_path_buf();
            let pkg = get_package_json(&abs.to_string_lossy());
            let name = pkg.name.unwrap_or_else(|| rel.to_string_lossy().to_string());
            let scripts = pkg.scripts.unwrap_or_default();
            PackageEntry {
                name,
                cwd: pkg_dir,
                description: String::new(),
                scripts,
            }
        })
        .collect();

    if let Some(cmd) = filter_cmd {
        entries.retain(|p| p.scripts.contains_key(cmd));
    }
    entries
}
