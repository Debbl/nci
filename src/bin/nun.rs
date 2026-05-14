use std::{env, process};

use console::style;
use inquire::MultiSelect;
use nci::{
    parse::parse_nun,
    runner::run_cli,
    utils::{exclude, get_package_json},
};

#[derive(Clone)]
struct DepChoice {
    name: String,
    version: String,
}

impl std::fmt::Display for DepChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:<30} {}",
            self.name,
            style(&self.version).dim()
        )
    }
}

fn main() {
    run_cli(
        |agent, mut args, ctx| {
            let is_multiple = args.first().map(|a| a == "-m").unwrap_or(false);
            let is_global = args.iter().any(|a| a == "-g");
            let is_interactive = args.is_empty()
                && ctx.as_ref().map(|c| !c.programmatic).unwrap_or(true);

            if (is_interactive || is_multiple) && !is_global {
                let cwd = ctx
                    .as_ref()
                    .map(|c| c.cwd.clone())
                    .unwrap_or_else(|| env::current_dir().unwrap());
                let pkg_path = cwd.join("package.json");
                let pkg = get_package_json(&pkg_path.to_string_lossy());

                let mut all_deps: Vec<DepChoice> = Vec::new();
                if let Some(d) = pkg.dependencies {
                    for (name, version) in d {
                        all_deps.push(DepChoice { name, version });
                    }
                }
                if let Some(d) = pkg.devDependencies {
                    for (name, version) in d {
                        all_deps.push(DepChoice { name, version });
                    }
                }

                if all_deps.is_empty() {
                    eprintln!("No dependencies found");
                    process::exit(1);
                }

                let selection = MultiSelect::new("remove dependencies", all_deps).prompt();
                let selected = match selection {
                    Ok(s) => s,
                    Err(_) => process::exit(1),
                };

                if is_multiple {
                    args = exclude(&args, &["-m".to_string()]);
                }
                args.extend(selected.into_iter().map(|c| c.name));
            }

            parse_nun(agent, args, ctx)
        },
        None,
    )
}
