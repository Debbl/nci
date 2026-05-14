use std::env;
use std::fmt::Display;
use std::process;

use console::style;
use inquire::Select;
use nci::{
    completion::{
        completion_suggestions, RAW_BASH_COMPLETION_SCRIPT, RAW_FISH_COMPLETION_SCRIPT,
        RAW_ZSH_COMPLETION_SCRIPT,
    },
    config::{get_run_agent, RunAgent},
    fuzzy,
    monorepo::{load_packages, PackageEntry},
    parse::parse_nr,
    runner::run_cli,
    storage::{dump, load, STORAGE},
    utils::{exclude, get_package_json, merge_workspace_flag},
};

#[derive(Debug, Clone)]
struct ScriptRaw {
    pub key: String,
    pub _cmd: String,
    pub description: String,
}

impl Display for ScriptRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = self.key.clone();
        let description = self.description.clone();
        let item = format!("{}    {}", style(key).cyan(), style(description).dim());
        write!(f, "{}", item)
    }
}

fn main() {
    // Completion paths bypass agent detection — they only need the cwd's
    // package.json. Handled in main() so a stale lockfile / missing agent
    // never blocks completion.
    let argv: Vec<String> = env::args().skip(1).collect();
    if let Some(first) = argv.first() {
        match first.as_str() {
            "--completion-bash" => {
                println!("{}", RAW_BASH_COMPLETION_SCRIPT.trim());
                return;
            }
            "--completion-zsh" => {
                println!("{}", RAW_ZSH_COMPLETION_SCRIPT.trim());
                return;
            }
            "--completion-fish" => {
                println!("{}", RAW_FISH_COMPLETION_SCRIPT.trim());
                return;
            }
            "--completion" => {
                // In bash COMP_LINE+COMP_CWORD are set and argv shape is
                // ["--completion", "nr", "<typed>"…]. In zsh/fish, only
                // ["--completion", "<typed>"…].
                let prefix = if env::var("COMP_LINE").is_ok() && env::var("COMP_CWORD").is_ok() {
                    let cword: usize = env::var("COMP_CWORD")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    if cword == 1 {
                        argv.get(2).cloned().unwrap_or_default()
                    } else {
                        return;
                    }
                } else {
                    argv.get(1).cloned().unwrap_or_default()
                };
                let cwd = env::current_dir().unwrap_or_default();
                for s in completion_suggestions(&cwd, &prefix) {
                    println!("{}", s);
                }
                return;
            }
            _ => {}
        }
    }

    run_cli(
        |agent, mut args, mut ctx| {
            load();

            // `-p` flag: pick a package in the workspace, then run a script
            // inside it. We update ctx.cwd AND the process cwd so both the
            // script picker (which reads ctx.cwd/package.json) and the
            // eventual spawned `npm run …` see the new directory.
            if args.first().map(|s| s.as_str()) == Some("-p") {
                let script_filter = args
                    .get(1)
                    .cloned()
                    .filter(|s| !s.starts_with('-'));
                let base = ctx
                    .as_ref()
                    .map(|c| c.cwd.clone())
                    .unwrap_or_else(|| env::current_dir().unwrap_or_default());
                let mut pkgs = load_packages(&base, script_filter.as_deref());
                if pkgs.is_empty() {
                    eprintln!("{}", style("No packages found").red());
                    process::exit(1);
                }
                let selected = if pkgs.len() == 1 {
                    pkgs.remove(0)
                } else {
                    let filter = |input: &str,
                                  opt: &PackageEntry,
                                  _opt_str: &str,
                                  _idx: usize|
                     -> bool {
                        fuzzy::matches(input, &opt.name)
                    };
                    match Select::new("select a package", pkgs)
                        .with_filter(&filter)
                        .prompt()
                    {
                        Ok(p) => p,
                        Err(_) => process::exit(1),
                    }
                };
                env::set_current_dir(&selected.cwd)
                    .expect("change to selected package directory");
                if let Some(c) = ctx.as_mut() {
                    c.cwd = selected.cwd.clone();
                }
                args.remove(0); // strip `-p`
            }

            if args.len() > 0 && args[0] == "-" {
                let storage_guard = STORAGE.lock();
                let storage = storage_guard.as_ref().unwrap();
                let storage = storage.clone();
                if storage.last_run_command.is_none() {
                    println!("{}", style("No last command found").red());
                    process::exit(1)
                }
                args[0] = storage.last_run_command.unwrap();
            }

            if args.len() == 0 {
                match ctx {
                    Some(ctx) => {
                        if !ctx.programmatic {
                            let path = ctx.cwd.join("package.json");
                            match path.to_str() {
                                Some(path) => {
                                    let storage_guard = STORAGE.lock();
                                    let storage = storage_guard.as_ref().unwrap();
                                    let pkg = get_package_json(path);
                                    let scripts = pkg.scripts.unwrap_or_default();
                                    let scripts_info = pkg.scripts_info.unwrap_or_default();
                                    let names = scripts
                                        .iter()
                                        .map(|(key, value)| [key, value])
                                        .collect::<Vec<[&String; 2]>>();
                                    let raw = names
                                        .iter()
                                        .filter(|x| !x[0].starts_with("?"))
                                        .map(|[key, value]| {
                                            let key = key.to_string();
                                            let cmd = value.to_string();
                                            let description = scripts_info
                                                .get(&key)
                                                .map_or_else(|| cmd.clone(), |v| v.to_string());
                                            ScriptRaw {
                                                key: key,
                                                _cmd: cmd,
                                                description,
                                            }
                                        })
                                        .collect::<Vec<ScriptRaw>>();

                                    if let Some(command) = &storage.last_run_command {
                                        let last = raw.iter().find(|x| command == &x.key);
                                        match last {
                                            Some(_) => {
                                                // raw.insert(0, last.clone())
                                            }
                                            None => {}
                                        };
                                    }

                                    // fuzzy filter against `key + description`,
                                    // matching upstream's fzf selector.
                                    let filter = |input: &str,
                                                  opt: &ScriptRaw,
                                                  _opt_str: &str,
                                                  _idx: usize|
                                     -> bool {
                                        let combined =
                                            format!("{} {}", opt.key, opt.description);
                                        fuzzy::matches(input, &combined)
                                    };
                                    let ans = Select::new("script to run:", raw)
                                        .with_filter(&filter)
                                        .prompt();

                                    if let Ok(ans) = ans {
                                        args.push(ans.key);
                                    } else {
                                        process::exit(1)
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                    None => {}
                }
            }
            let storage_guard = STORAGE.lock();
            let mut storage = storage_guard.as_ref().unwrap().clone();
            match storage.last_run_command.clone() {
                Some(command) => {
                    if command != args[0] {
                        storage.last_run_command = Some(args[0].to_string());
                        dump(&storage).unwrap();
                    }
                }
                None => {
                    storage.last_run_command = Some(args[0].to_string());
                    dump(&storage).unwrap();
                }
            };

            drop(storage_guard);

            let mut storage_guard = STORAGE.lock();
            *storage_guard = Some(storage);
            drop(storage_guard);

            // `runAgent=node` swaps `<agent> run` for `node --run`. Node's
            // `--run` doesn't support `--if-present`, so we drop it.
            if get_run_agent() == Some(RunAgent::Node) {
                let stripped = exclude(&args, &["--if-present".to_string()]);
                let merged = merge_workspace_flag(stripped);
                let mut cmd_args = vec!["--run".to_string()];
                cmd_args.extend(merged);
                return ("node".to_string(), cmd_args);
            }

            parse_nr(agent, args)
        },
        None,
    )
}
