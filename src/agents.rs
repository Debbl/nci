#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub enum Agent {
    Npm,
    Yarn,
    YarnBerry,
    Pnpm,
    Pnpm6,
    PnpmRush,
    Aube,
    Nub,
    Bun,
    Deno,
}

impl Agent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Agent::Npm => "npm",
            Agent::Yarn => "yarn",
            Agent::YarnBerry => "yarn@berry",
            Agent::Pnpm => "pnpm",
            Agent::Pnpm6 => "pnpm@6",
            Agent::Bun => "bun",
            Agent::Deno => "deno",
            Agent::PnpmRush => "pnpm-rush",
            Agent::Aube => "aube",
            Agent::Nub => "nub",
        }
    }

    /// Executable name, stripping any `@version` suffix.
    pub fn exec(&self) -> &'static str {
        match self {
            Agent::YarnBerry => "yarn",
            Agent::Pnpm6 => "pnpm",
            Agent::PnpmRush => "rush-pnpm",
            other => other.as_str(),
        }
    }

    pub fn commands(&self) -> &'static AgentCommands {
        match self {
            Agent::Npm => &NPM_COMMAND,
            Agent::Yarn => &YARN_COMMAND,
            Agent::YarnBerry => &YARN_BERRY_COMMAND,
            Agent::Pnpm => &PNPM_COMMAND,
            Agent::Pnpm6 => &PNPM6_COMMAND,
            Agent::Bun => &BUN_COMMAND,
            Agent::Deno => &DENO_COMMAND,
            Agent::PnpmRush => &PNPM_RUSH_COMMAND,
            Agent::Aube => &AUBE_COMMAND,
            Agent::Nub => &NUB_COMMAND,
        }
    }
}

/// One piece of a command template: either a literal token or the placeholder
/// where the user-supplied args are spliced in.
#[derive(Copy, Clone, Debug)]
pub enum Fragment {
    Lit(&'static str),
    Args,
}

/// A resolved command shape. Mirrors `AgentCommandValue` in the upstream
/// `package-manager-detector`:
/// - `None` ≡ `null` (unsupported by this agent)
/// - `Plain` ≡ `[lit, lit, 0, ...]`
/// - `DashDash` ≡ `dashDashArg(agent, sub)` — inserts a `--` separator between
///   the script and its forwarded args, preserving leading agent flags.
#[derive(Copy, Clone, Debug)]
pub enum AgentCommandValue {
    None,
    Plain(&'static [Fragment]),
    DashDash(&'static str, &'static str),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AgentCommand {
    Agent,
    Run,
    Install,
    Frozen,
    Global,
    Add,
    Upgrade,
    UpgradeInteractive,
    Dedupe,
    Execute,
    ExecuteLocal,
    Uninstall,
    GlobalUninstall,
}

pub struct AgentCommands {
    pub agent: AgentCommandValue,
    pub run: AgentCommandValue,
    pub install: AgentCommandValue,
    pub frozen: AgentCommandValue,
    pub global: AgentCommandValue,
    pub add: AgentCommandValue,
    pub upgrade: AgentCommandValue,
    pub upgrade_interactive: AgentCommandValue,
    pub dedupe: AgentCommandValue,
    pub execute: AgentCommandValue,
    pub execute_local: AgentCommandValue,
    pub uninstall: AgentCommandValue,
    pub global_uninstall: AgentCommandValue,
}

impl AgentCommands {
    pub fn get(&self, cmd: AgentCommand) -> AgentCommandValue {
        match cmd {
            AgentCommand::Agent => self.agent,
            AgentCommand::Run => self.run,
            AgentCommand::Install => self.install,
            AgentCommand::Frozen => self.frozen,
            AgentCommand::Global => self.global,
            AgentCommand::Add => self.add,
            AgentCommand::Upgrade => self.upgrade,
            AgentCommand::UpgradeInteractive => self.upgrade_interactive,
            AgentCommand::Dedupe => self.dedupe,
            AgentCommand::Execute => self.execute,
            AgentCommand::ExecuteLocal => self.execute_local,
            AgentCommand::Uninstall => self.uninstall,
            AgentCommand::GlobalUninstall => self.global_uninstall,
        }
    }
}

/// Build an `AgentCommandValue` literally. `_` denotes the args placeholder,
/// anything else is a `&'static str` literal. `cmd![]` is the unsupported case.
///
/// ```ignore
/// cmd!["npm", "i", "-g", _]   // -> npm i -g <args...>
/// cmd![]                      // -> unsupported
/// ```
macro_rules! cmd {
    () => { AgentCommandValue::None };
    ($($t:tt),+ $(,)?) => {
        AgentCommandValue::Plain(&[$( cmd!(@f $t) ),+])
    };
    (@f _) => { Fragment::Args };
    (@f $s:literal) => { Fragment::Lit($s) };
}

/// `dashDashArg(agent, sub)` from upstream: emits
/// `[agent, sub, ...flags, script, --, ...script_args]`, with no separator
/// when the script has no forwarded arguments.
macro_rules! dd {
    ($agent:literal, $sub:literal) => {
        AgentCommandValue::DashDash($agent, $sub)
    };
}

pub const NPM_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["npm", _],
    run: dd!("npm", "run"),
    install: cmd!["npm", "i", _],
    frozen: cmd!["npm", "ci", _],
    global: cmd!["npm", "i", "-g", _],
    add: cmd!["npm", "i", _],
    upgrade: cmd!["npm", "update", _],
    upgrade_interactive: cmd![],
    dedupe: cmd!["npm", "dedupe", _],
    execute: cmd!["npx", _],
    execute_local: cmd!["npx", _],
    uninstall: cmd!["npm", "uninstall", _],
    global_uninstall: cmd!["npm", "uninstall", "-g", _],
};

pub const YARN_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["yarn", _],
    run: cmd!["yarn", "run", _],
    install: cmd!["yarn", "install", _],
    frozen: cmd!["yarn", "install", "--frozen-lockfile", _],
    global: cmd!["yarn", "global", "add", _],
    add: cmd!["yarn", "add", _],
    upgrade: cmd!["yarn", "upgrade", _],
    upgrade_interactive: cmd!["yarn", "upgrade-interactive", _],
    dedupe: cmd![],
    execute: cmd!["npx", _],
    execute_local: dd!("yarn", "exec"),
    uninstall: cmd!["yarn", "remove", _],
    global_uninstall: cmd!["yarn", "global", "remove", _],
};

pub const YARN_BERRY_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["yarn", _],
    run: cmd!["yarn", "run", _],
    install: cmd!["yarn", "install", _],
    frozen: cmd!["yarn", "install", "--immutable", _],
    // Yarn 2+ removed `global`, fall back to npm.
    global: cmd!["npm", "i", "-g", _],
    add: cmd!["yarn", "add", _],
    upgrade: cmd!["yarn", "up", _],
    upgrade_interactive: cmd!["yarn", "up", "-i", _],
    dedupe: cmd!["yarn", "dedupe", _],
    execute: cmd!["yarn", "dlx", _],
    execute_local: cmd!["yarn", "exec", _],
    uninstall: cmd!["yarn", "remove", _],
    global_uninstall: cmd!["npm", "uninstall", "-g", _],
};

pub const PNPM_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["pnpm", _],
    run: cmd!["pnpm", "run", _],
    install: cmd!["pnpm", "i", _],
    frozen: cmd!["pnpm", "i", "--frozen-lockfile", _],
    global: cmd!["pnpm", "add", "-g", _],
    add: cmd!["pnpm", "add", _],
    upgrade: cmd!["pnpm", "update", _],
    upgrade_interactive: cmd!["pnpm", "update", "-i", _],
    dedupe: cmd!["pnpm", "dedupe", _],
    execute: cmd!["pnpm", "dlx", _],
    execute_local: cmd!["pnpm", "exec", _],
    uninstall: cmd!["pnpm", "remove", _],
    global_uninstall: cmd!["pnpm", "remove", "--global", _],
};

pub const PNPM6_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["pnpm", _],
    run: dd!("pnpm", "run"),
    install: cmd!["pnpm", "i", _],
    frozen: cmd!["pnpm", "i", "--frozen-lockfile", _],
    global: cmd!["pnpm", "add", "-g", _],
    add: cmd!["pnpm", "add", _],
    upgrade: cmd!["pnpm", "update", _],
    upgrade_interactive: cmd!["pnpm", "update", "-i", _],
    dedupe: cmd!["pnpm", "dedupe", _],
    execute: cmd!["pnpm", "dlx", _],
    execute_local: cmd!["pnpm", "exec", _],
    uninstall: cmd!["pnpm", "remove", _],
    global_uninstall: cmd!["pnpm", "remove", "--global", _],
};

pub const BUN_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["bun", _],
    run: cmd!["bun", "run", _],
    install: cmd!["bun", "install", _],
    frozen: cmd!["bun", "install", "--frozen-lockfile", _],
    global: cmd!["bun", "add", "-g", _],
    add: cmd!["bun", "add", _],
    upgrade: cmd!["bun", "update", _],
    upgrade_interactive: cmd!["bun", "update", "-i", _],
    dedupe: cmd![],
    execute: cmd!["bun", "x", _],
    execute_local: cmd!["bun", "x", _],
    uninstall: cmd!["bun", "remove", _],
    global_uninstall: cmd!["bun", "remove", "-g", _],
};

pub const DENO_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["deno", _],
    run: cmd!["deno", "task", _],
    install: cmd!["deno", "install", _],
    frozen: cmd!["deno", "install", "--frozen", _],
    global: cmd!["deno", "install", "-g", _],
    add: cmd!["deno", "add", _],
    upgrade: cmd!["deno", "outdated", "--update", _],
    upgrade_interactive: cmd!["deno", "outdated", "--update", _],
    dedupe: cmd![],
    execute: cmd!["deno", "x", _],
    execute_local: cmd!["deno", "task", "--eval", _],
    uninstall: cmd!["deno", "remove", _],
    global_uninstall: cmd!["deno", "uninstall", "-g", _],
};

pub const PNPM_RUSH_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["rush-pnpm", _],
    run: cmd!["rush-pnpm", "run", _],
    install: cmd!["rush-pnpm", "i", _],
    frozen: cmd!["rush-pnpm", "i", "--frozen-lockfile", _],
    global: cmd!["rush-pnpm", "add", "-g", _],
    add: cmd!["rush-pnpm", "add", _],
    upgrade: cmd!["rush-pnpm", "update", _],
    upgrade_interactive: cmd!["rush-pnpm", "update", "-i", _],
    dedupe: cmd!["rush-pnpm", "dedupe", _],
    execute: cmd!["rush-pnpm", "dlx", _],
    execute_local: cmd!["rush-pnpm", "exec", _],
    uninstall: cmd!["rush-pnpm", "remove", _],
    global_uninstall: cmd!["rush-pnpm", "remove", "--global", _],
};

pub const AUBE_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["aube", _],
    run: cmd!["aube", "run", _],
    install: cmd!["aube", "install", _],
    frozen: cmd!["aube", "install", "--frozen-lockfile", _],
    global: cmd!["aube", "add", "-g", _],
    add: cmd!["aube", "add", _],
    upgrade: cmd!["aube", "update", _],
    upgrade_interactive: cmd!["aube", "update", "-i", _],
    dedupe: cmd!["aube", "dedupe", _],
    execute: cmd!["aube", "dlx", _],
    execute_local: cmd!["aube", "exec", _],
    uninstall: cmd!["aube", "remove", _],
    global_uninstall: cmd!["aube", "remove", "-g", _],
};

pub const NUB_COMMAND: AgentCommands = AgentCommands {
    agent: cmd!["nub", _],
    run: cmd!["nub", "run", _],
    install: cmd!["nub", "install", _],
    frozen: cmd!["nub", "install", "--frozen-lockfile", _],
    global: cmd!["nub", "add", "-g", _],
    add: cmd!["nub", "add", _],
    upgrade: cmd!["nub", "update", _],
    upgrade_interactive: cmd!["nub", "update", "-i", _],
    dedupe: cmd!["nub", "dedupe", _],
    execute: cmd!["nubx", _],
    execute_local: cmd!["nub", "exec", _],
    uninstall: cmd!["nub", "remove", _],
    global_uninstall: cmd!["nub", "remove", "-g", _],
};
