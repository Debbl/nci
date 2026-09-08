# nci

A Rust port of [antfu-collective/ni](https://github.com/antfu-collective/ni).

**nci** — use the right package manager.

<a href='https://docs.npmjs.com/cli/v6/commands/npm'>npm</a> · <a href='https://yarnpkg.com'>yarn</a> · <a href='https://pnpm.io/'>pnpm</a> · <a href='https://bun.sh/'>bun</a> · <a href='https://deno.land/'>deno</a> · <a href='https://aube.en.dev/'>aube</a> · <a href='https://nubjs.com/'>nub</a> · <a href='https://rushjs.io/'>Rush (rush-pnpm)</a>

## Install

Eight binaries — `ni`, `nr`, `nlx`, `nup`, `nun`, `nci`, `nd`, `na` — plus `nu` as a legacy alias for `nup`.

### Homebrew (macOS, Linux)

```bash
brew install Debbl/tap/nci
```

### Prebuilt binaries

Grab the archive matching your platform from the [latest release](https://github.com/Debbl/nci/releases/latest):

- `nci-aarch64-apple-darwin.tar.gz` — macOS Apple Silicon
- `nci-x86_64-apple-darwin.tar.gz` — macOS Intel
- `nci-x86_64-unknown-linux-gnu.tar.gz` — Linux glibc
- `nci-x86_64-pc-windows-msvc.zip` — Windows

Each archive ships the same nine binaries. Drop them anywhere on your `PATH`.

### Cargo

```bash
cargo install nci
```

## Credits

- [antfu-collective/ni](https://github.com/antfu-collective/ni)
- [zhazhazhu/ni](https://github.com/zhazhazhu/ni)

---

### `ni` — install

```bash
ni

# npm install
# yarn install
# pnpm install
# bun install
# deno install
```

```bash
ni vite

# npm i vite
# yarn add vite
# pnpm add vite
# bun add vite
# deno add vite
```

```bash
ni @types/node -D

# npm i @types/node -D
# yarn add @types/node -D
# pnpm add -D @types/node
# bun add -d @types/node
# deno add -D @types/node
```

```bash
ni -P

# npm i --omit=dev
# yarn install --production
# pnpm i --production
# bun install --production
# (deno not supported)
```

```bash
ni --frozen

# npm ci
# yarn install --frozen-lockfile (Yarn 1)
# yarn install --immutable (Yarn Berry)
# pnpm i --frozen-lockfile
# bun install --frozen-lockfile
# deno install --frozen
```

```bash
ni -g eslint

# npm i -g eslint
# yarn global add eslint (Yarn 1)
# pnpm add -g eslint
# bun add -g eslint
# deno install -g eslint

# uses the global agent, regardless of your current working directory
```

```bash
ni -i

# interactively search the npm registry and pick a package to install
```

<details>
<summary>Workspace catalogs (pnpm, Yarn Berry, Bun)</summary>

When a workspace declares catalogs, `nci` writes `catalog:` references into `package.json` instead of pinning versions. Supported locations:

- pnpm: `pnpm-workspace.yaml`
- Yarn Berry: `.yarnrc.yml`
- Bun: the workspace root's `package.json`, either at the top level or inside `workspaces` (nested catalogs take precedence)

For example, with pnpm:

```bash
# pnpm-workspace.yaml:
#   catalog:
#     react: ^18.3.0

ni react
# → detects react in the default catalog
# → writes "react": "catalog:" to package.json
# → runs `pnpm i`

ni lodash
# → lodash isn't in any catalog
# → with only a default catalog: silently adds to it
# → with multiple named catalogs: prompts for a catalog (or skip / create new)
# → fetches latest from the npm registry, updates pnpm-workspace.yaml
# → writes "lodash": "catalog:..." to package.json
# → runs `pnpm i`
```

When installing multiple new packages, subsequent prompts offer **same as previous** and **apply to all remaining**, including the choice to skip catalogs. Packages already in a catalog keep their existing catalog.

The dependency flag picks the right `package.json` section:

```bash
ni typescript -D
# → writes "typescript": "catalog:dev" to devDependencies
```

`-w` / `--workspace` targets the workspace root's `package.json`:

```bash
ni react -w
```

Bun example:

```json
{
  "workspaces": {
    "packages": ["packages/*"],
    "catalogs": { "prod": { "react": "^18.3.0" } }
  }
}
```

`ni react` writes `"react": "catalog:prod"` to the current package and runs `bun install`. New catalog entries are written back to the original catalog location.

To disable catalog mode for any supported agent, set `catalog=false` in `~/.nirc` or `NI_CATALOG=false`.

</details>

<br>

### `nr` — run

```bash
nr dev --port=3000

# npm run dev -- --port=3000
# yarn run dev --port=3000
# pnpm run dev --port=3000
# bun run dev --port=3000
# deno task dev --port=3000
```

```bash
nr

# interactive picker (fzf-style fuzzy filtering)
# descriptions: scripts-info, then scripts["?name"], then the script command
# the last-run script is listed first unless noLastCommand=true
```

```bash
nr -

# rerun the last command
```

```bash
nr -p
nr -p dev

# pick a workspace package (auto-selects when only one matches), then
# run the script there
```

<details>
<summary>shell completion</summary>

```bash
# bash
nr --completion-bash >> ~/.bashrc

# zsh — for example with zim:fw
mkdir -p ~/.zim/custom/ni-completions
nr --completion-zsh > ~/.zim/custom/ni-completions/_ni
echo "zmodule $HOME/.zim/custom/ni-completions --fpath ." >> ~/.zimrc
zimfw install

# fish
mkdir -p ~/.config/fish/completions
nr --completion-fish > ~/.config/fish/completions/nr.fish
```

</details>

<br>

### `nlx` — download & execute

```bash
nlx vitest

# npx vitest
# yarn dlx vitest
# pnpm dlx vitest
# bun x vitest
# deno x vitest
```

```bash
nlx --local vitest

# npx vitest
# yarn exec vitest
# pnpm exec vitest
# bun x vitest
# deno task --eval vitest
# aube exec vitest
# nub exec vitest
# rush-pnpm exec vitest
```

`--local` uses the agent's local execution command; ordinary `nlx` uses its download-and-execute command (`dlx`, `bun x`, `nubx`, etc.).

<br>

### `nup` — upgrade

```bash
nup

# npm update
# yarn upgrade (Yarn 1)
# yarn up (Yarn Berry)
# pnpm update
# bun update
# deno outdated --update
```

```bash
nup -i

# (not available on npm)
# yarn upgrade-interactive (Yarn 1)
# yarn up -i (Yarn Berry)
# pnpm update -i
# bun update -i
# deno outdated --update
```

> Earlier versions of nci shipped this command as `nu`. The `nu` binary is still installed as a deprecated alias to keep old scripts working; new shells should use `nup` (matching upstream's rename away from a clash with Nushell).

<br>

### `nun` — uninstall

```bash
nun webpack

# npm uninstall webpack
# yarn remove webpack
# pnpm remove webpack
# bun remove webpack
# deno remove webpack
```

```bash
nun

# interactively multi-select dependencies to remove
```

```bash
nun -g silent

# npm uninstall -g silent
# yarn global remove silent
# pnpm remove -g silent
# bun remove -g silent
# deno uninstall -g silent
```

<br>

### `nci` — clean install

```bash
nci

# npm ci
# yarn install --frozen-lockfile
# pnpm i --frozen-lockfile
# bun install --frozen-lockfile
# deno install --frozen
```

`nci` automatically installs a missing detected agent. Other commands prompt first; set `NI_AUTO_INSTALL=true` to allow automatic installation there too. Programmatic mode never installs missing agents during detection. Aube uses the npm package `@endevco/aube`.

<br>

### `nd` — dedupe dependencies

```bash
nd

# npm dedupe
# yarn dedupe (Yarn Berry only — Yarn 1 doesn't support it)
# pnpm dedupe
# (bun / deno not supported)
```

`nd -c` rewrites to `--check` on pnpm and `--dry-run` on npm — both ways to preview without writing.

<br>

### `na` — agent alias

```bash
na

# npm
# yarn
# pnpm
# bun
# deno
```

```bash
na run foo

# npm run foo
# yarn run foo
# pnpm run foo
# bun run foo
# deno run foo
```

<br>

### Global flags

```bash
# ?               | dry-run: print the resolved command and exit
ni vite ?

# -C              | change directory before running anything
ni -C packages/foo vite
nr -C playground dev

# --agent         | print the detected agent name (for shell scripts)
nci --agent          # prints "pnpm", "npm", "deno", or "unknown"

# --programmatic  | suppress prompts and the "Running:" banner
ni react --programmatic

# -v, --version   | show nci / node / detected agent / global agent versions
ni -v

# -h, --help      | show help
ni -h
```

<br>

### Config

```ini
; ~/.nirc

; fallback when no lock file is detected
defaultAgent=npm                # default: "prompt"

; agent used for `-g` global installs
globalAgent=npm

; use `node --run <script>` instead of `<agent> run <script>` (requires Node 22+)
runAgent=node

; wrap every spawned command with `sfw <agent> <args>`
useSfw=true

; pnpm / Yarn Berry / Bun catalog support; set to false to opt out
catalog=true

; suppress the `nr` picker's behaviour of surfacing the last-run script
noLastCommand=false
```

Keys are also accepted in `snake_case` (`default_agent`, `global_agent`, …) for backward compatibility.

Every option has a matching environment variable that takes precedence over `~/.nirc`:

```bash
export NI_CONFIG_FILE="$HOME/.config/ni/nirc"   # alternate rc path
export NI_DEFAULT_AGENT=pnpm
export NI_GLOBAL_AGENT=npm
export NI_RUN_AGENT=node
export NI_USE_SFW=true
export NI_CATALOG=false
export NI_NO_LAST_COMMAND=true
export NI_AUTO_INSTALL=true                     # auto `npm i -g` missing agents
```

On Windows (PowerShell):

```powershell
$Env:NI_CONFIG_FILE = 'C:\path\to\your\nirc'
```

<br>

### Integrations

#### asdf

`ni` is available via the [3rd-party asdf-plugin](https://github.com/CanRau/asdf-ni.git) maintained by [CanRau](https://github.com/CanRau):

```bash
asdf plugin add ni https://github.com/CanRau/asdf-ni.git
asdf install ni latest
asdf global ni latest
```

<br>

### How?

Detection follows upstream ni v30.5.0 and `package-manager-detector` 1.8.0. A `deno.json` / `deno.jsonc` in the target directory selects Deno first. Otherwise, `nci` searches the current directory and then each parent, so the nearest project's metadata wins.

Within each directory:

1. `rush.json` selects `pnpm-rush`, which executes commands through `rush-pnpm`.
2. `packageManager` in `package.json` takes precedence over that directory's lockfile; `devEngines.packageManager` is used when `packageManager` is absent.
3. Otherwise, marker priority is: `aube-lock.yaml` / `aube-workspace.yaml`, `bun.lock` / `bun.lockb`, `deno.lock`, `nub.lock`, `pnpm-lock.yaml` / `pnpm-workspace.yaml`, `yarn.lock`, `package-lock.json` / `npm-shrinkwrap.json`.

Version ranges such as `pnpm@^10.0.0` are accepted. Yarn major versions above 1 select Yarn Berry; pnpm major versions below 7 select pnpm 6. Invalid package JSON is ignored during detection. The command table lives in [`src/agents.rs`](src/agents.rs).

<br>

### Troubleshooting

#### Conflicts with PowerShell's `ni`

PowerShell ships with a built-in alias `ni` for the `New-Item` cmdlet. Remove it in the current session:

```powershell
Remove-Item Alias:ni -Force -ErrorAction Ignore
```

To persist, drop the same line into your PowerShell profile (`$PROFILE`):

```powershell
if (-not (Test-Path $profile)) {
  New-Item -ItemType File -Path (Split-Path $profile) -Force -Name (Split-Path $profile -Leaf)
}

$profileEntry = 'Remove-Item Alias:ni -Force -ErrorAction Ignore'
$profileContent = Get-Content $profile
if ($profileContent -notcontains $profileEntry) {
  ("`n" + $profileEntry) | Out-File $profile -Append -Force -Encoding UTF8
}
```

#### `nx` / `nix` / `nu`

Upstream renamed `nx`/`nix` to `nlx` (clashes with [nx](https://nx.dev/) and [nix](https://nixos.org/)) and `nu` to `nup` (clashes with [Nushell](https://www.nushell.sh/)). `nci` keeps `nu` as an alias for `nup`. For `nlx`, alias the others yourself if you prefer:

```bash
alias nx="nlx"
alias nix="nlx"
```
