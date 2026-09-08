//! Interactive catalog selection. Returns the chosen catalog name or `None`
//! when the user skipped / cancelled.

use console::style;
use inquire::{Select, Text};

use super::CatalogConfig;
use crate::fuzzy;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatalogSelection {
    pub catalog_name: Option<String>,
    pub apply_to_rest: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Choice {
    Catalog(String),
    Same(Option<String>),
    Rest(Option<String>),
    Create,
    Skip,
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Catalog(name) => write!(f, "{}", name),
            Self::Same(name) => write!(
                f,
                "{} (same as previous)",
                name.as_deref().unwrap_or("skip")
            ),
            Self::Rest(name) => write!(
                f,
                "{} (apply to all remaining)",
                name.as_deref().unwrap_or("skip")
            ),
            Self::Create => write!(f, "[create new catalog]"),
            Self::Skip => write!(f, "[skip — install without catalog]"),
        }
    }
}

fn choices(
    config: &CatalogConfig,
    previous: Option<&Option<String>>,
    has_remaining: bool,
) -> Vec<Choice> {
    let mut choices = Vec::new();
    if let Some(previous) = previous {
        choices.push(Choice::Same(previous.clone()));
        if has_remaining {
            choices.push(Choice::Rest(previous.clone()));
        }
    }
    choices.extend(
        config
            .catalogs
            .iter()
            .map(|c| Choice::Catalog(c.name.clone())),
    );
    choices.extend([Choice::Create, Choice::Skip]);
    choices
}

pub fn prompt_select_catalog(
    config: &CatalogConfig,
    pkg: &str,
    programmatic: bool,
) -> Option<String> {
    prompt_select_catalog_with_previous(config, pkg, programmatic, None, false).catalog_name
}

pub fn prompt_select_catalog_with_previous(
    config: &CatalogConfig,
    pkg: &str,
    programmatic: bool,
    previous: Option<&Option<String>>,
    has_remaining: bool,
) -> CatalogSelection {
    if config.has_default_catalog && !config.has_named_catalogs {
        return CatalogSelection {
            catalog_name: Some("default".into()),
            apply_to_rest: false,
        };
    }
    if programmatic {
        return CatalogSelection::default();
    }
    let message = format!("select catalog for {}", style(pkg).yellow());
    let filter =
        |input: &str, _opt: &Choice, label: &str, _idx: usize| fuzzy::matches(input, label);
    let chosen = Select::new(&message, choices(config, previous, has_remaining))
        .with_filter(&filter)
        .prompt();
    match chosen {
        Ok(Choice::Catalog(name)) => CatalogSelection {
            catalog_name: Some(name),
            apply_to_rest: false,
        },
        Ok(Choice::Same(name)) => CatalogSelection {
            catalog_name: name,
            apply_to_rest: false,
        },
        Ok(Choice::Rest(name)) => CatalogSelection {
            catalog_name: name,
            apply_to_rest: true,
        },
        Ok(Choice::Create) => CatalogSelection {
            catalog_name: prompt_new_catalog_name(),
            apply_to_rest: false,
        },
        _ => CatalogSelection::default(),
    }
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

    #[test]
    fn shortcuts_precede_catalogs_and_preserve_skip_selection() {
        let config = cfg(false, &["prod", "dev"]);
        assert_eq!(
            choices(&config, None, true)[0],
            Choice::Catalog("prod".into())
        );
        let previous = Some("dev".into());
        let list = choices(&config, Some(&previous), true);
        assert_eq!(list[0], Choice::Same(previous.clone()));
        assert_eq!(list[1], Choice::Rest(previous));
        let list = choices(&config, Some(&None), false);
        assert_eq!(list[0], Choice::Same(None));
        assert!(!list.iter().any(|c| matches!(c, Choice::Rest(_))));
    }
}
