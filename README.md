# nci

A Rust port of [antfu-collective/ni](https://github.com/antfu-collective/ni).

**nci** — use the right package manager.

<a href='https://docs.npmjs.com/cli/v6/commands/npm'>npm</a> · <a href='https://yarnpkg.com'>yarn</a> · <a href='https://pnpm.io/'>pnpm</a> · <a href='https://bun.sh/'>bun</a> · <a href='https://deno.land/'>deno</a>

## Install

```bash
cargo install nci
```

This installs eight binaries: `ni`, `nr`, `nlx`, `nup`, `nun`, `nci`, `nd`, `na` (plus `nu` as a legacy alias for `nup`).

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
<summary>pnpm catalogs</summary>

When a pnpm workspace declares [catalogs](https://pnpm.io/catalogs) in `pnpm-workspace.yaml`, `nci` writes `catalog:` references into `package.json` instead of pinning versions:

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

The dependency flag picks the right `package.json` section:

```bash
ni typescript -D
# → writes "typescript": "catalog:dev" to devDependencies
```

`-w` / `--workspace` targets the workspace root's `package.json`:

```bash
ni react -w
```

To disable catalog mode, set `catalog=false` in `~/.nirc` or `NI_CATALOG=false`.

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
# supports https://www.npmjs.com/package/npm-scripts-info convention
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

If the detected agent isn't installed, `nci` will offer to globally install it for you (or just do it when `NI_AUTO_INSTALL=true`).

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

; pnpm catalog support; set to false to opt out
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

`nci` assumes you work with lockfiles. Before running, it inspects (in order of precedence):

1. `deno.json` / `deno.jsonc` → deno
2. `bun.lock` / `bun.lockb` → bun
3. `pnpm-lock.yaml` → pnpm
4. `yarn.lock` → yarn
5. `package-lock.json` / `npm-shrinkwrap.json` → npm
6. `packageManager` field in `package.json`

`yarn@>1` is treated as Yarn Berry; `pnpm@<7` is treated as pnpm 6. The full command table lives in [`src/agents.rs`](src/agents.rs) and mirrors [`package-manager-detector`'s commands](https://github.com/antfu-collective/package-manager-detector/blob/main/src/commands.ts).

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
