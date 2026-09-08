use std::process;

use crate::{
    agents::{Agent, AgentCommand, AgentCommandValue, Fragment},
    runner::RunnerContext,
    utils::{exclude, merge_workspace_flag},
};

const GLOBAL: &str = "-g";
const FROZEN: &str = "--frozen";
const IF_PRESENT: &str = "--if-present";
const FROZEN_IF_PRESENT: &str = "--frozen-if-present";

pub type CommandTuple = (String, Vec<String>);

/// Splice user args into a command template. Returns `None` for unsupported
/// commands (`AgentCommandValue::None`).
pub fn construct(value: AgentCommandValue, args: &[String]) -> Option<CommandTuple> {
    match value {
        AgentCommandValue::None => None,
        AgentCommandValue::Plain(frags) => {
            let mut out: Vec<String> = Vec::new();
            for f in frags {
                match f {
                    Fragment::Lit(s) => out.push((*s).to_string()),
                    Fragment::Args => out.extend(args.iter().cloned()),
                }
            }
            if out.is_empty() {
                return None;
            }
            let cmd = out.remove(0);
            Some((cmd, out))
        }
        AgentCommandValue::DashDash(agent, sub) => {
            let mut rest: Vec<String> = vec![sub.to_string()];
            let value_flags: &[&str] = match (agent, sub) {
                ("npm", "run") => &["-w", "--workspace"],
                ("pnpm", "run") => &["-F", "--filter"],
                _ => &[],
            };
            let script_index = args.iter().enumerate().position(|(i, arg)| {
                !arg.starts_with('-') && (i == 0 || !value_flags.contains(&args[i - 1].as_str()))
            });
            if let Some(index) = script_index {
                rest.extend_from_slice(&args[..=index]);
                if index + 1 < args.len() {
                    rest.push("--".into());
                    rest.extend_from_slice(&args[index + 1..]);
                }
            } else {
                rest.extend_from_slice(args);
            }
            Some((agent.to_string(), rest))
        }
    }
}

fn get_command(agent: &Agent, command: AgentCommand, args: Vec<String>) -> CommandTuple {
    let value = agent.commands().get(command);
    match construct(value, &args) {
        Some(t) => t,
        None => {
            eprintln!(
                "\u{2717} Command \"{:?}\" is not supported by agent \"{}\"",
                command,
                agent.as_str()
            );
            process::exit(1);
        }
    }
}

pub fn parse_ni(agent: Agent, args: Vec<String>, ctx: Option<RunnerContext>) -> CommandTuple {
    let mut args = args;
    if agent == Agent::Bun {
        args = args
            .iter()
            .map(|i| if i == "-D" { "-d".into() } else { i.clone() })
            .collect();
    }
    // npm uses `--omit=dev`; other agents map `-P` to `--production`.
    if agent == Agent::Npm {
        args = args
            .iter()
            .map(|i| {
                if i == "-P" {
                    "--omit=dev".into()
                } else {
                    i.clone()
                }
            })
            .collect();
    }
    if args.iter().any(|i| i == "-P") {
        args = args
            .iter()
            .map(|i| {
                if i == "-P" {
                    "--production".into()
                } else {
                    i.clone()
                }
            })
            .collect();
    }
    if args.contains(&GLOBAL.into()) {
        return get_command(
            &agent,
            AgentCommand::Global,
            exclude(&args, &[GLOBAL.to_string()]),
        );
    }
    if args.contains(&FROZEN_IF_PRESENT.into()) {
        let cleaned = exclude(&args, &[FROZEN_IF_PRESENT.to_string()]);
        let has_lock = ctx.as_ref().map(|c| c.has_lock).unwrap_or(false);
        return get_command(
            &agent,
            if has_lock {
                AgentCommand::Frozen
            } else {
                AgentCommand::Install
            },
            cleaned,
        );
    }
    if args.contains(&FROZEN.into()) {
        return get_command(
            &agent,
            AgentCommand::Frozen,
            exclude(&args, &[FROZEN.to_string()]),
        );
    }
    if args.is_empty() || args.iter().all(|item| item.starts_with('-')) {
        return get_command(&agent, AgentCommand::Install, args);
    }
    get_command(&agent, AgentCommand::Add, args)
}

pub fn parse_nr(agent: Agent, mut args: Vec<String>) -> CommandTuple {
    if args.is_empty() {
        args.push("start".into());
    }
    let has_if_present = args.contains(&IF_PRESENT.to_string());
    if has_if_present {
        args = exclude(&args, &[IF_PRESENT.to_string()]);
    }
    args = merge_workspace_flag(args);
    let (cmd, mut cmd_args) = get_command(&agent, AgentCommand::Run, args);
    if has_if_present {
        // Insert `--if-present` right after the `run` subcommand, matching
        // upstream's `cmd.args.splice(1, 0, '--if-present')`.
        cmd_args.insert(1, IF_PRESENT.to_string());
    }
    (cmd, cmd_args)
}

pub fn parse_nun(agent: Agent, args: Vec<String>, _: Option<RunnerContext>) -> CommandTuple {
    if args.contains(&GLOBAL.into()) {
        return get_command(
            &agent,
            AgentCommand::GlobalUninstall,
            exclude(&args, &[GLOBAL.to_string()]),
        );
    }
    get_command(&agent, AgentCommand::Uninstall, args)
}

pub fn parse_nlx(agent: Agent, args: Vec<String>, _: Option<RunnerContext>) -> CommandTuple {
    if args.iter().any(|a| a == "--local") {
        return get_command(
            &agent,
            AgentCommand::ExecuteLocal,
            exclude(&args, &["--local".into()]),
        );
    }
    get_command(&agent, AgentCommand::Execute, args)
}

pub fn parse_nu(agent: Agent, args: Vec<String>, _: Option<RunnerContext>) -> CommandTuple {
    if args.contains(&"-i".to_string()) {
        return get_command(
            &agent,
            AgentCommand::UpgradeInteractive,
            exclude(&args, &["-i".to_string()]),
        );
    }
    get_command(&agent, AgentCommand::Upgrade, args)
}

pub fn parse_na(agent: Agent, args: Vec<String>, _: Option<RunnerContext>) -> CommandTuple {
    get_command(&agent, AgentCommand::Agent, args)
}

pub fn parse_nd(agent: Agent, args: Vec<String>, _: Option<RunnerContext>) -> CommandTuple {
    // pnpm uses `--check`, npm uses `--dry-run`; expose both via `-c`.
    let mut args = args;
    if matches!(agent, Agent::Pnpm | Agent::Pnpm6 | Agent::Aube) {
        args = args
            .iter()
            .map(|i| {
                if i == "-c" {
                    "--check".into()
                } else {
                    i.clone()
                }
            })
            .collect();
    } else if agent == Agent::Npm {
        args = args
            .iter()
            .map(|i| {
                if i == "-c" {
                    "--dry-run".into()
                } else {
                    i.clone()
                }
            })
            .collect();
    }
    get_command(&agent, AgentCommand::Dedupe, args)
}
