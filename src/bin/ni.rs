use nci::{parse::parse_ni, runner::run_cli};

fn main() {
    run_cli(parse_ni, None)
}

#[cfg(test)]
mod ni {
    #[cfg(test)]
    mod npm {
        use nci::{agents::Agent, parse::parse_ni};

        #[test]
        fn empty() {
            let (agent, args) = parse_ni(Agent::Npm, Vec::new(), None);
            assert_eq!(agent, "npm");
            assert_eq!(args, ["i"])
        }

        #[test]
        fn single_add() {
            let (agent, args) = parse_ni(Agent::Npm, vec!["axios".to_string()], None);
            assert_eq!(agent, "npm");
            assert_eq!(args, ["i", "axios"])
        }

        #[test]
        fn add_dev() {
            let (agent, args) =
                parse_ni(Agent::Npm, vec!["vite".to_string(), "-D".to_string()], None);
            assert_eq!(agent, "npm");
            assert_eq!(args, ["i", "vite", "-D"])
        }

        #[test]
        fn multiple() {
            let (agent, args) = parse_ni(
                Agent::Npm,
                vec!["eslint".to_string(), "@types/node".to_string()],
                None,
            );
            assert_eq!(agent, "npm");
            assert_eq!(args, ["i", "eslint", "@types/node"])
        }

        #[test]
        fn global() {
            let (agent, args) = parse_ni(
                Agent::Npm,
                vec!["eslint".to_string(), "-g".to_string()],
                None,
            );
            assert_eq!(agent, "npm");
            assert_eq!(args, ["i", "-g", "eslint"])
        }

        #[test]
        fn frozen() {
            let (agent, args) = parse_ni(Agent::Npm, vec!["--frozen".to_string()], None);
            assert_eq!(agent, "npm");
            assert_eq!(args, ["ci"])
        }
    }

    #[cfg(test)]
    mod bun {
        use nci::{agents::Agent, parse::parse_ni};

        #[test]
        fn empty() {
            let (agent, args) = parse_ni(Agent::Bun, Vec::new(), None);
            assert_eq!(agent, "bun");
            assert_eq!(args, ["install"])
        }

        #[test]
        fn single_add() {
            let (agent, args) = parse_ni(Agent::Bun, vec!["axios".to_string()], None);
            assert_eq!(agent, "bun");
            assert_eq!(args, ["add", "axios"])
        }

        #[test]
        fn add_dev() {
            let (agent, args) =
                parse_ni(Agent::Bun, vec!["vite".to_string(), "-D".to_string()], None);
            assert_eq!(agent, "bun");
            assert_eq!(args, ["add", "vite", "-d"])
        }

        #[test]
        fn multiple() {
            let (agent, args) = parse_ni(
                Agent::Bun,
                vec!["eslint".to_string(), "@types/node".to_string()],
                None,
            );
            assert_eq!(agent, "bun");
            assert_eq!(args, ["add", "eslint", "@types/node"])
        }

        #[test]
        fn global() {
            let (agent, args) = parse_ni(
                Agent::Bun,
                vec!["eslint".to_string(), "-g".to_string()],
                None,
            );
            assert_eq!(agent, "bun");
            assert_eq!(args, ["add", "-g", "eslint"])
        }

        #[test]
        fn frozen() {
            let (agent, args) = parse_ni(Agent::Bun, vec!["--frozen".to_string()], None);
            assert_eq!(agent, "bun");
            assert_eq!(args, ["install", "--no-save"])
        }
    }

    #[cfg(test)]
    mod pnpm {

        use nci::{agents::Agent, parse::parse_ni};

        #[test]
        fn empty() {
            let (agent, args) = parse_ni(Agent::Pnpm, Vec::new(), None);
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["i"])
        }

        #[test]
        fn single_add() {
            let (agent, args) = parse_ni(Agent::Pnpm, vec!["axios".to_string()], None);
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["add", "axios"])
        }

        #[test]
        fn multiple() {
            let (agent, args) = parse_ni(
                Agent::Pnpm,
                vec!["eslint".to_string(), "@types/node".to_string()],
                None,
            );
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["add", "eslint", "@types/node"])
        }

        #[test]
        fn add_dev() {
            let (agent, args) = parse_ni(
                Agent::Pnpm,
                vec![
                    "-D".to_string(),
                    "eslint".to_string(),
                    "@types/node".to_string(),
                ],
                None,
            );
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["add", "-D", "eslint", "@types/node"])
        }

        #[test]
        fn global() {
            let (agent, args) = parse_ni(
                Agent::Pnpm,
                vec!["eslint".to_string(), "-g".to_string()],
                None,
            );
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["add", "-g", "eslint"])
        }

        #[test]
        fn frozen() {
            let (agent, args) = parse_ni(Agent::Pnpm, vec!["--frozen".to_string()], None);
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["i", "--frozen-lockfile"])
        }

        #[test]
        fn forward1() {
            let (agent, args) = parse_ni(Agent::Pnpm, vec!["--anything".to_string()], None);
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["i", "--anything"])
        }
        #[test]
        fn forward2() {
            let (agent, args) = parse_ni(Agent::Pnpm, vec!["-a".to_string()], None);
            assert_eq!(agent, "pnpm");
            assert_eq!(args, ["i", "-a"])
        }
    }

    #[cfg(test)]
    mod yarn {
        use nci::{agents::Agent, parse::parse_ni};

        #[test]
        fn empty() {
            let (agent, args) = parse_ni(Agent::Yarn, Vec::new(), None);
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["install"])
        }

        #[test]
        fn single_add() {
            let (agent, args) =
                nci::parse::parse_ni(nci::agents::Agent::Yarn, vec!["axios".to_string()], None);
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["add", "axios"])
        }

        #[test]
        fn multiple() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::Yarn,
                vec!["eslint".to_string(), "@types/node".to_string()],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["add", "eslint", "@types/node"])
        }

        #[test]
        fn add_dev() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::Yarn,
                vec![
                    "-D".to_string(),
                    "eslint".to_string(),
                    "@types/node".to_string(),
                ],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["add", "-D", "eslint", "@types/node",])
        }

        #[test]
        fn global() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::Yarn,
                vec!["eslint".to_string(), "-g".to_string()],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["global", "add", "eslint"])
        }

        #[test]
        fn frozen() {
            let (agent, args) =
                nci::parse::parse_ni(nci::agents::Agent::Yarn, vec!["--frozen".to_string()], None);
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["install", "--frozen-lockfile"])
        }
    }

    #[cfg(test)]
    mod yarn_berry {
        #[test]
        fn empty() {
            let (agent, args) =
                nci::parse::parse_ni(nci::agents::Agent::YarnBerry, Vec::new(), None);
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["install"])
        }

        #[test]
        fn single_add() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::YarnBerry,
                vec!["axios".to_string()],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["add", "axios"])
        }

        #[test]
        fn multiple() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::YarnBerry,
                vec!["eslint".to_string(), "@types/node".to_string()],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["add", "eslint", "@types/node"])
        }

        #[test]
        fn add_dev() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::YarnBerry,
                vec![
                    "-D".to_string(),
                    "eslint".to_string(),
                    "@types/node".to_string(),
                ],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["add", "-D", "eslint", "@types/node"])
        }

        #[test]
        fn global() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::YarnBerry,
                vec!["eslint".to_string(), "-g".to_string()],
                None,
            );
            assert_eq!(agent, "npm");
            assert_eq!(args, ["i", "-g", "eslint"])
        }

        #[test]
        fn frozen() {
            let (agent, args) = nci::parse::parse_ni(
                nci::agents::Agent::YarnBerry,
                vec!["--frozen".to_string()],
                None,
            );
            assert_eq!(agent, "yarn");
            assert_eq!(args, ["install", "--immutable"])
        }
    }
}
