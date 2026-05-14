use crate::{
    agents::Agent,
    detect::{detect, AGENT_MAP},
    runner::DetectOptions,
};
use dirs::home_dir;
use ini::Ini;
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub enum DefaultAgent {
    Prompt,
    Agent(Agent),
}

/// `runAgent=node` makes nr run via `node --run <script>` (requires Node 22+).
#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub enum RunAgent {
    Node,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub default_agent: DefaultAgent,
    pub global_agent: Agent,
    pub run_agent: Option<RunAgent>,
    pub use_sfw: bool,
    pub catalog: bool,
    pub no_last_command: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_agent: DefaultAgent::Prompt,
            global_agent: Agent::Npm,
            run_agent: None,
            use_sfw: false,
            catalog: true,
            no_last_command: false,
        }
    }
}

/// Try both camelCase and snake_case forms of a key.
fn section_get<'a>(
    section: &'a ini::Properties,
    camel: &str,
    snake: &str,
) -> Option<&'a str> {
    section.get(camel).or_else(|| section.get(snake))
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

impl Config {
    /// Layer the ini file and environment variables on top of the defaults.
    pub fn assign(&self) -> Self {
        let home = home_dir().unwrap_or(PathBuf::from("~/"));
        let rc_path = env::var("NI_CONFIG_FILE")
            .unwrap_or_else(|_| home.join(".nirc").to_string_lossy().to_string());

        let mut config = self.clone();

        if Path::new(&rc_path).exists() {
            if let Ok(conf) = Ini::load_from_file(&rc_path) {
                if let Some(section) = conf.section(None::<String>) {
                    if let Some(v) = section_get(section, "defaultAgent", "default_agent") {
                        if v == "prompt" {
                            config.default_agent = DefaultAgent::Prompt;
                        } else if let Some(agent) = AGENT_MAP.get(v) {
                            config.default_agent = DefaultAgent::Agent(agent.clone());
                        }
                    }
                    if let Some(v) = section_get(section, "globalAgent", "global_agent") {
                        if let Some(agent) = AGENT_MAP.get(v) {
                            config.global_agent = agent.clone();
                        }
                    }
                    if let Some(v) = section_get(section, "runAgent", "run_agent") {
                        if v == "node" {
                            config.run_agent = Some(RunAgent::Node);
                        }
                    }
                    if let Some(v) = section_get(section, "useSfw", "use_sfw") {
                        if let Some(b) = parse_bool(v) {
                            config.use_sfw = b;
                        }
                    }
                    if let Some(v) = section.get("catalog") {
                        if let Some(b) = parse_bool(v) {
                            config.catalog = b;
                        }
                    }
                    if let Some(v) = section_get(section, "noLastCommand", "no_last_command") {
                        if let Some(b) = parse_bool(v) {
                            config.no_last_command = b;
                        }
                    }
                }
            }
        }

        // Environment variables override the file.
        if let Ok(v) = env::var("NI_DEFAULT_AGENT") {
            if v == "prompt" {
                config.default_agent = DefaultAgent::Prompt;
            } else if let Some(agent) = AGENT_MAP.get(v.as_str()) {
                config.default_agent = DefaultAgent::Agent(agent.clone());
            }
        }
        if let Ok(v) = env::var("NI_GLOBAL_AGENT") {
            if let Some(agent) = AGENT_MAP.get(v.as_str()) {
                config.global_agent = agent.clone();
            }
        }
        if let Ok(v) = env::var("NI_RUN_AGENT") {
            if v == "node" {
                config.run_agent = Some(RunAgent::Node);
            }
        }
        if let Ok(v) = env::var("NI_USE_SFW") {
            if let Some(b) = parse_bool(&v) {
                config.use_sfw = b;
            }
        }
        if let Ok(v) = env::var("NI_CATALOG") {
            if let Some(b) = parse_bool(&v) {
                config.catalog = b;
            }
        }
        if let Ok(v) = env::var("NI_NO_LAST_COMMAND") {
            if let Some(b) = parse_bool(&v) {
                config.no_last_command = b;
            }
        }

        config
    }
}

pub fn get_config() -> Config {
    let mut config = Config::default().assign();
    let options = DetectOptions {
        programmatic: true,
        ..DetectOptions::default()
    };
    if let Some(agent) = detect(options) {
        config.default_agent = DefaultAgent::Agent(agent);
    }
    config
}

pub fn get_default_agent(programmatic: bool) -> DefaultAgent {
    let Config { default_agent, .. } = get_config();
    let ci = env::var("CI");

    if default_agent == DefaultAgent::Prompt && (programmatic || ci.is_ok()) {
        return DefaultAgent::Agent(Agent::Npm);
    }
    default_agent
}

pub fn get_global_agent() -> Agent {
    get_config().global_agent
}

pub fn get_run_agent() -> Option<RunAgent> {
    get_config().run_agent
}

pub fn get_use_sfw() -> bool {
    get_config().use_sfw
}

pub fn get_catalog() -> bool {
    get_config().catalog
}

pub fn get_no_last_command() -> bool {
    get_config().no_last_command
}
