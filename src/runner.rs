use console::style;
use inquire::Select;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::{env, io};

use crate::agents::Agent;
use crate::config::{get_default_agent, get_global_agent, DefaultAgent};
use crate::detect::{detect, AGENT_MAP};
use crate::utils::{exclude, get_volta_prefix};

const DEBUG_SIGN: &str = "?";
const PROGRAMMATIC_SIGN: &str = "--programmatic";

#[derive(Clone)]
pub struct DetectOptions {
    pub cwd: PathBuf,
    pub auto_install: bool,
    pub programmatic: bool,
}
impl Default for DetectOptions {
    fn default() -> Self {
        DetectOptions {
            cwd: env::current_dir().unwrap(),
            auto_install: false,
            programmatic: false,
        }
    }
}
impl DetectOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_auto_install(mut self, auto_install: bool) -> Self {
        self.auto_install = auto_install;
        self
    }
}

pub struct RunnerContext {
    pub programmatic: bool,
    pub has_lock: bool,
    pub cwd: PathBuf,
}

pub type Runner =
    fn(agent: Agent, args: Vec<String>, ctx: Option<RunnerContext>) -> (String, Vec<String>);

pub fn run_cli(func: Runner, options: Option<DetectOptions>) {
    let args = env::args().collect::<Vec<String>>()[1..]
        .to_vec()
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect::<Vec<String>>();

    let mut options = options.unwrap_or_default();

    run(func, args, &mut options)
}

pub fn run(func: Runner, args: Vec<String>, options: &mut DetectOptions) {
    let version = env!("CARGO_PKG_VERSION");

    let mut args = args;

    // `-C path` switches the working directory used for detection and
    // execution, then is consumed.
    if args.len() >= 2 && args[0] == "-C" {
        let path = Path::new(args[1].as_str());
        options.cwd = if path.is_absolute() {
            path.to_path_buf()
        } else {
            options.cwd.join(path)
        };
        args.drain(0..2);
    }

    // `?` (DEBUG_SIGN) — dry-run: print the resolved command instead of
    // executing it. `--programmatic` flips the programmatic option from the
    // command line.
    let debug = args.iter().any(|a| a == DEBUG_SIGN);
    if debug {
        args = exclude(&args, &[DEBUG_SIGN.to_string()]);
    }
    let programmatic_from_args = args.iter().any(|a| a == PROGRAMMATIC_SIGN);
    if programmatic_from_args {
        args = exclude(&args, &[PROGRAMMATIC_SIGN.to_string()]);
        options.programmatic = true;
    }

    if args.len() == 1 && (args[0].to_lowercase() == "-v" || args[0] == "--version") {
        print_versions(version, options);
        return;
    }

    // `--agent` prints the detected agent name. Useful for shell scripts —
    // force programmatic on detect() so it never prompts to auto-install a
    // missing agent, but keep the user's programmatic setting for the
    // fallback so "no lock + prompt config" stays as "unknown".
    if args.len() == 1 && args[0] == "--agent" {
        let mut detect_opts = options.clone();
        detect_opts.programmatic = true;
        let agent = match detect(detect_opts) {
            Some(a) => a.as_str().to_string(),
            None => match get_default_agent(options.programmatic) {
                DefaultAgent::Agent(a) => a.as_str().to_string(),
                DefaultAgent::Prompt => "unknown".to_string(),
            },
        };
        println!("{}", agent);
        return;
    }

    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        let dash = style("-").dim();
        println!(
            "{} {}\n",
            style("nci").green().bold(),
            style(format!("use the right package manager v{}", version)).dim()
        );
        println!("ni    {}  install", dash);
        println!("nr    {}  run", dash);
        println!("nlx   {}  execute", dash);
        println!("nup   {}  upgrade", dash);
        println!("nun   {}  uninstall", dash);
        println!("nci   {}  clean install", dash);
        println!("nd    {}  dedupe dependencies", dash);
        println!("na    {}  agent alias", dash);
        println!("nu    {}  alias for nup (legacy)", dash);
        println!("ni -v       {}  show versions", dash);
        println!("ni --agent  {}  print detected agent (for scripting)", dash);
        println!("ni ?        {}  dry run (print the resolved command)", dash);
        println!(
            "{}",
            style("\ncheck https://github.com/Debbl/nci for more documentation.").blue()
        );
        return;
    }

    // Apply -C: enter options.cwd so subsequent detection and the eventual
    // spawned process all see the same directory. Closures may chdir again
    // for finer-grained changes (e.g. `nr -p` selecting a workspace package).
    if let Err(e) = env::set_current_dir(&options.cwd) {
        eprintln!(
            "[ni] couldn't enter {}: {}",
            options.cwd.display(),
            e
        );
        process::exit(1);
    }

    let command = get_cli_command(func, args, options.clone());

    if let Some((mut agent, mut args)) = command {
        // `useSfw=true` wraps the resolved command in `sfw <cmd> <args>`.
        if crate::config::get_use_sfw() {
            if crate::utils::which_cmd("sfw") {
                args.insert(0, agent);
                agent = "sfw".to_string();
            } else if options.programmatic {
                eprintln!("[ni] sfw is enabled but not installed.");
                process::exit(1);
            } else {
                eprintln!("[ni] sfw is enabled but not installed.");
                eprintln!("[ni] Install it with: npm install -g sfw");
                process::exit(1);
            }
        }

        if let Some((volta, volta_args)) = get_volta_prefix() {
            args.insert(0, agent);
            agent = volta;
            args = volta_args.into_iter().chain(args).collect();
        }

        if debug {
            // Dry-run: emit the resolved command on stdout and stop.
            let mut tokens = vec![agent];
            tokens.extend(args);
            println!("{}", tokens.join(" "));
            return;
        }

        if !options.programmatic {
            println!(
                "{} {}",
                style("Running:").dim(),
                style(format!("{} {}", agent, args.join(" "))).green()
            );
        }

        execa_command(&agent, Some(args)).unwrap();
    }
}

/// Print `nci`'s own version plus the runtime versions of node, the detected
/// agent, and the global agent. Mirrors `ni -v` in upstream.
fn print_versions(self_version: &str, options: &DetectOptions) {
    fn version_of(cmd: &str) -> String {
        let out = Command::new(cmd).arg("-v").output();
        match out {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.starts_with('v') {
                    s
                } else {
                    format!("v{}", s)
                }
            }
            _ => "unknown".to_string(),
        }
    }
    fn pad(s: &str) -> String {
        format!("{:<10}", s)
    }

    println!(
        "{} {}",
        pad("nci"),
        style(format!("v{}", self_version)).cyan()
    );
    println!("{} {}", pad("node"), style(version_of("node")).green());

    match detect(options.clone()) {
        Some(a) => println!(
            "{} {}",
            pad(a.as_str()),
            style(version_of(a.exec())).blue()
        ),
        None => println!("{} no lock file", pad("agent")),
    }

    let global = get_global_agent();
    let label = format!("{} -g", global.as_str());
    println!(
        "{} {}",
        pad(&label),
        style(version_of(global.exec())).blue()
    );
}

fn get_cli_command(
    func: Runner,
    args: Vec<String>,
    options: DetectOptions,
) -> Option<(String, Vec<String>)> {
    let global = "-g".to_string();
    if args.contains(&global) {
        return Some(func(get_global_agent(), args, None));
    }
    let detected = detect(options.clone());
    // `hasLock` upstream is `Boolean(agent)` — true iff detect resolved an
    // agent (via lockfile or `packageManager` field).
    let has_lock = detected.is_some();
    let mut _agent = match detected {
        Some(v) => DefaultAgent::Agent(v),
        None => get_default_agent(options.programmatic),
    };

    if _agent == DefaultAgent::Prompt {
        let items: Vec<&&str> = AGENT_MAP.keys().filter(|x| !x.contains("@")).collect();
        let filter = |input: &str, opt: &&&str, _opt_str: &str, _idx: usize| -> bool {
            crate::fuzzy::matches(input, opt)
        };
        let selection = Select::new("Choose the agent", items)
            .with_filter(&filter)
            .prompt();
        if let Ok(selection) = selection {
            let value = AGENT_MAP.get(selection);
            if let Some(value) = value {
                _agent = DefaultAgent::Agent(value.clone());
            } else {
                return None;
            }
        } else {
            process::exit(1)
        }
    }
    let runner_ctx = RunnerContext {
        programmatic: options.programmatic,
        has_lock,
        cwd: options.cwd,
    };
    match _agent {
        DefaultAgent::Agent(agent) => Some(func(agent, args, Some(runner_ctx))),
        DefaultAgent::Prompt => Some(func(Agent::Npm, args, Some(runner_ctx))),
    }
}

pub fn execa_command(agent: &str, args: Option<Vec<String>>) -> Result<(), io::Error> {
    let status = Command::new(agent)
        .args(args.unwrap_or_default())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("failed to spawn `{}`: {}", agent, e),
            )
        })?
        .wait()?;

    // Propagate the agent's exit code so CI / shell scripts can react to
    // failure. Without this, `nci` would always exit 0 even when npm fails.
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
