//! Interactive catalog selection. Returns the chosen catalog name or `None`
//! when the user skipped / cancelled.

use console::style;
use inquire::{Select, Text};

use super::CatalogConfig;
use crate::fuzzy;

/// Pick a catalog for `pkg`. Behaviour mirrors upstream:
///
/// - Only the default catalog exists → no prompt, return `Some("default")`.
/// - Multiple catalogs → prompt with the existing names plus "create new"
///   and "skip" choices.
/// - `programmatic` → never prompt, return `None`.
pub fn prompt_select_catalog(
    config: &CatalogConfig,
    pkg: &str,
    programmatic: bool,
) -> Option<String> {
    if config.has_default_catalog && !config.has_named_catalogs {
        return Some("default".to_string());
    }
    if programmatic {
        return None;
    }

    let mut choices: Vec<String> = config.catalogs.iter().map(|c| c.name.clone()).collect();
    choices.push("[create new catalog]".to_string());
    choices.push("[skip — install without catalog]".to_string());

    let message = format!("select catalog for {}", style(pkg).yellow());
    let filter = |input: &str, opt: &String, _opt_str: &str, _idx: usize| -> bool {
        fuzzy::matches(input, opt)
    };
    let chosen = Select::new(&message, choices.clone())
        .with_filter(&filter)
        .prompt()
        .ok()?;

    if chosen.starts_with("[skip") {
        return None;
    }
    if chosen.starts_with("[create new") {
        return prompt_new_catalog_name();
    }
    Some(chosen)
}

fn prompt_new_catalog_name() -> Option<String> {
    let name = Text::new("new catalog name").prompt().ok()?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// We can't easily unit-test the interactive prompts, but the
// "only default catalog skips prompting" branch is pure logic.
#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use std::path::PathBuf;

    use crate::catalog::CatalogInfo;

    fn cfg(has_default: bool, named: &[&str]) -> CatalogConfig {
        let mut catalogs = Vec::new();
        if has_default {
            catalogs.push(CatalogInfo {
                name: "default".to_string(),
                packages: IndexMap::new(),
            });
        }
        for n in named {
            catalogs.push(CatalogInfo {
                name: (*n).to_string(),
                packages: IndexMap::new(),
            });
        }
        CatalogConfig {
            file_path: PathBuf::from("/tmp/pnpm-workspace.yaml"),
            catalogs,
            has_default_catalog: has_default,
            has_named_catalogs: !named.is_empty(),
        }
    }

    #[test]
    fn programmatic_returns_none_unless_default_only() {
        let c = cfg(true, &["prod"]);
        assert_eq!(prompt_select_catalog(&c, "react", true), None);
    }

    #[test]
    fn default_only_skips_prompt() {
        let c = cfg(true, &[]);
        assert_eq!(
            prompt_select_catalog(&c, "react", true),
            Some("default".to_string())
        );
        // Also picks default when not programmatic, since there's no choice
        // to make.
        assert_eq!(
            prompt_select_catalog(&c, "react", false),
            Some("default".to_string())
        );
    }
}
