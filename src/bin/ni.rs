use std::{process, vec};

use console::style;
use inquire::{Select, Text};
use nci::{fetch::fetch_npm_packages, parse::parse_ni, runner::run_cli, utils::exclude};
use tokio::runtime::Runtime;

fn main() {
    run_cli(
        |agent, mut args, ctx| {
            let is_interactive = args.get(0) == Some(&"-i".to_string());

            if is_interactive {
                let fetch_pattern = match args.get(1) {
                    Some(pattern) => pattern.clone(),
                    None => match Text::new("search for package").prompt() {
                        Ok(pattern) => pattern,
                        Err(_) => process::exit(1),
                    },
                };

                // fetch npm packages
                let packages = Runtime::new()
                    .unwrap()
                    .block_on(fetch_npm_packages(&fetch_pattern));
                let packages = match packages {
                    Ok(packages) => {
                        if packages.len() == 0 {
                            println!("No results found");
                            process::exit(1);
                        } else {
                            packages
                        }
                    }
                    Err(_) => {
                        eprintln!("Failed to fetch packages");
                        process::exit(1);
                    }
                };

                // select package
                let dependency = Select::new("choose a package to install", {
                    packages.iter().map(|pkg| pkg.value.name.clone()).collect()
                })
                .prompt();
                let dependency = match dependency {
                    Ok(dependency) => dependency,
                    Err(_) => process::exit(1),
                };

                args = exclude(
                    &args,
                    &["-d".to_string(), "-p".to_string(), "-i".to_string()],
                );

                // yarn and bun do not support the installation of peers programmatically
                let can_install_peers = ["npm", "pnpm"].contains(&agent.as_str());

                // select install mode
                let mode = Select::new(&format!("install {} as", style(&dependency).yellow()), {
                    if can_install_peers {
                        vec!["-prod", "-dev", "--save-peer"]
                    } else {
                        vec!["-prod", "-dev"]
                    }
                })
                .prompt();
                let mode = match mode {
                    Ok(mode) => mode,
                    Err(_) => process::exit(1),
                };

                args.extend(vec![dependency.to_string(), mode.to_string()]);
            }

            parse_ni(agent, args, ctx)
        },
        None,
    );
}
