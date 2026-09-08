use console::style;
use indexmap::IndexMap;
use inquire::Confirm;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path, process};

use crate::{
    agents::Agent,
    runner::{execa_command, DetectOptions},
    utils::{terminal_link, which_cmd},
};

#[derive(Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct Package {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub version: Option<String>,
    pub packageManager: Option<String>,
    pub scripts: Option<IndexMap<String, String>>,
    #[serde(rename = "scripts-info")]
    pub scripts_info: Option<IndexMap<String, String>>,
    pub dependencies: Option<IndexMap<String, String>>,
    pub devDependencies: Option<IndexMap<String, String>>,
}

lazy_static! {
    pub static ref AGENT_MAP: IndexMap<&'static str, Agent> = {
        let mut m = IndexMap::new();
        m.insert("aube", Agent::Aube);
        m.insert("nub", Agent::Nub);
        m.insert("pnpm-rush", Agent::PnpmRush);
        m.insert("bun", Agent::Bun);
        m.insert("deno", Agent::Deno);
        m.insert("pnpm", Agent::Pnpm);
        m.insert("pnpm@6", Agent::Pnpm6);
        m.insert("yarn", Agent::Yarn);
        m.insert("yarn@berry", Agent::YarnBerry);
        m.insert("npm", Agent::Npm);
        m
    };
    pub static ref AGENT_INSTALL: IndexMap<Agent, &'static str> = {
        let mut m = IndexMap::new();
        m.insert(Agent::Aube, "https://aube.en.dev/installation");
        m.insert(Agent::Nub, "https://nubjs.com/docs/install");
        m.insert(
            Agent::PnpmRush,
            "https://rushjs.io/pages/intro/get_started/",
        );
        m.insert(Agent::Bun, "https://bun.sh");
        m.insert(
            Agent::Deno,
            "https://docs.deno.com/runtime/getting_started/installation/",
        );
        m.insert(Agent::Pnpm, "https://pnpm.io/installation");
        m.insert(Agent::Pnpm6, "https://pnpm.io/6.x/installation");
        m.insert(Agent::Yarn, "https://classic.yarnpkg.com/en/docs/install");
        m.insert(
            Agent::YarnBerry,
            "https://yarnpkg.com/getting-started/install",
        );
        m.insert(
            Agent::Npm,
            "https://docs.npmjs.com/cli/v8/configuring-npm/install",
        );
        m
    };
    pub static ref LOCKS_MAP: IndexMap<&'static str, Agent> = {
        let mut m = IndexMap::new();
        m.insert("aube-lock.yaml", Agent::Aube);
        m.insert("aube-workspace.yaml", Agent::Aube);
        m.insert("bun.lock", Agent::Bun);
        m.insert("bun.lockb", Agent::Bun);
        m.insert("deno.lock", Agent::Deno);
        m.insert("nub.lock", Agent::Nub);
        m.insert("pnpm-lock.yaml", Agent::Pnpm);
        m.insert("pnpm-workspace.yaml", Agent::Pnpm);
        m.insert("yarn.lock", Agent::Yarn);
        m.insert("package-lock.json", Agent::Npm);
        m.insert("npm-shrinkwrap.json", Agent::Npm);
        m
    };
}

pub fn detect(options: DetectOptions) -> Option<Agent> {
    // ni gives Deno configuration in the target directory precedence over
    // Node package managers, even if package.json declares another agent.
    if options.cwd.join("deno.json").is_file() || options.cwd.join("deno.jsonc").is_file() {
        return Some(Agent::Deno);
    }
    let (agent, version) = detect_in_ancestors(&options)?;
    if !which_cmd(agent.exec()) && !options.programmatic {
        if !options.auto_install {
            println!(
                "{}",
                style(format!(
                    "[ni] Detected {} but it doesn't seem to be installed.",
                    &agent.as_str()
                ))
                .yellow()
            );

            if env::var("CI").is_ok() {
                process::exit(1)
            }
            let install_url = AGENT_INSTALL.get(&agent).copied().unwrap_or("");
            let install_link = terminal_link(
                &style(agent.exec()).blue().underlined().to_string(),
                install_url,
            );
            let install_confirm_text =
                format!("Would you like to globally install {}?", install_link);
            let confirmation = Confirm::new(&install_confirm_text)
                .with_default(false)
                .prompt()
                .unwrap_or(false);

            if !confirmation {
                process::exit(1)
            }
        }

        let install_package = match agent {
            Agent::Aube => "@endevco/aube",
            Agent::PnpmRush => "@microsoft/rush",
            _ => agent.exec(),
        };
        let mut args: Vec<String> = vec!["i".into(), "-g".into()];
        if let Some(v) = version.clone() {
            args.push(format!("{}@{}", install_package, v));
        } else {
            args.push(install_package.to_string());
        }
        execa_command("npm", Some(args)).unwrap()
    }

    Some(agent)
}

/// Search one directory at a time so the nearest project's metadata wins.
fn detect_in_ancestors(options: &DetectOptions) -> Option<(Agent, Option<String>)> {
    for directory in options.cwd.ancestors() {
        if directory.join("rush.json").is_file() {
            return Some((Agent::PnpmRush, None));
        }
        let lock_agent = LOCKS_MAP
            .iter()
            .find(|(lock, _)| directory.join(lock).is_file())
            .map(|(_, agent)| agent.clone());
        if let Some(result) =
            read_package_manager(&directory.join("package.json"), options.programmatic)
        {
            return Some(result);
        }
        if let Some(agent) = lock_agent {
            return Some((agent, None));
        }
    }
    None
}

fn read_package_manager(path: &Path, programmatic: bool) -> Option<(Agent, Option<String>)> {
    let pkg: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
    let (name, raw_version) = if let Some(pm) = pkg.get("packageManager").and_then(|v| v.as_str()) {
        let pm = pm.strip_prefix('^').unwrap_or(pm);
        let (name, version) = pm.split_once('@').map_or((pm, None), |(n, v)| (n, Some(v)));
        (name, version)
    } else {
        let pm = pkg.get("devEngines")?.get("packageManager")?;
        (
            pm.get("name")?.as_str()?,
            pm.get("version").and_then(|v| v.as_str()),
        )
    };
    // Match the first numeric version in ranges such as ^10.0.0 or >=7.
    let version = raw_version.and_then(|v| {
        let start = v.find(|c: char| c.is_ascii_digit())?;
        Some(
            v[start..]
                .split(|c: char| !c.is_ascii_digit() && c != '.')
                .next()?
                .trim_end_matches('.')
                .to_string(),
        )
    });
    let major = version
        .as_deref()
        .and_then(|v| v.split('.').next()?.parse::<u64>().ok());
    match (name, major) {
        ("yarn", Some(v)) if v > 1 => Some((Agent::YarnBerry, Some("berry".into()))),
        ("pnpm", Some(v)) if v < 7 => Some((Agent::Pnpm6, version)),
        _ => match AGENT_MAP.get(name) {
            Some(agent) => Some((agent.clone(), version)),
            None => {
                if !programmatic {
                    eprintln!("[ni] Unknown packageManager: {}", name);
                }
                None
            }
        },
    }
}

pub fn find_up(filename: &str, cwd: &Path) -> Option<String> {
    let mut cwd = cwd.to_path_buf();
    loop {
        let file_path = cwd.join(filename);
        if file_path.is_file() {
            return Some(file_path.to_string_lossy().into());
        }
        if !cwd.pop() {
            break;
        }
    }
    None
}
