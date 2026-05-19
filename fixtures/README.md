# nci playground

Sandbox for verifying `nci` against every package manager it supports. Each
sub-directory is the minimal fixture that triggers detection for one agent:

| Dir            | Lockfile             | Detected as |
| -------------- | -------------------- | ----------- |
| `npm/`         | `package-lock.json`  | `npm`       |
| `yarn/`        | `yarn.lock`          | `yarn` (v1) |
| `yarn-berry/`  | `yarn.lock` + `packageManager: yarn@4.x` | `yarn@berry` |
| `pnpm/`        | `pnpm-lock.yaml`     | `pnpm`      |
| `bun/`         | `bun.lock`           | `bun`       |
| `deno/`        | `deno.json`          | `deno`      |

All `package.json` files declare the same `build`/`dev`/`test` scripts so
`nr <script>` is exercising the run-command paths.

## How to use

Drop into one of the dirs and run the dry-run form `?` to see the resolved
command without actually executing the package manager.

> **zsh users**: `?` is a glob char — quote it as `ni "?"` (or `noglob ni ?`).
> Bash and fish leave it alone.

> If the detected package manager isn't on your `PATH`, nci tries to prompt
> you to install it, which panics in a non-TTY shell (loops, pipes). Install
> the agent first (`brew install bun deno`, etc.) or set `CI=1` so nci exits
> cleanly instead of prompting.

```bash
cd npm
ni "?"                 # → "npm i"
ni vite "?"            # → "npm i vite"
ni @types/node -D "?"  # → "npm i @types/node -D"
ni --frozen "?"        # → "npm ci"
ni -g tsx "?"          # → "npm i -g tsx"  (uses NI_GLOBAL_AGENT / config)
nr build "?"           # → "npm run build"
nr dev "?"             # → "npm run dev"
nun lodash "?"         # → "npm uninstall lodash"
nlx vite "?"           # → "npx vite"
na "?"                 # → "npm"  (just the agent binary)
nci "?"                # → "npm ci"
nd "?"                 # → "npm dedupe"
nu "?"                 # → "npm update"  (legacy alias for nup)
nup "?"                # → "npm update"
```

Swap `npm` for any other dir and the same commands should produce the
appropriate variant. A few of the more interesting differences:

```bash
cd pnpm        && ni vite "?"            # → "pnpm add vite"
cd yarn        && ni @types/node -D "?"  # → "yarn add @types/node -D"
cd bun         && ni @types/node -D "?"  # → "bun add -d @types/node"  (lowercase -d)
cd deno        && ni vite "?"            # → "deno add vite"
cd yarn-berry  && nu -i "?"              # → "yarn up -i"  (interactive upgrade)
```

## Detected-only check

`--agent` prints just the agent name (useful for shell scripts):

```bash
cd npm && ni --agent          # npm
cd pnpm && ni --agent         # pnpm
cd yarn-berry && ni --agent   # yarn@berry
```

## After a `brew upgrade nci`

After every release, this is a quick sanity loop:

```bash
brew upgrade nci

# `--agent` only inspects the lockfile; doesn't try to spawn the agent,
# so this works even without bun/deno installed.
for dir in npm yarn yarn-berry pnpm bun deno; do
  printf "%-12s  " "$dir"
  (cd "$dir" && ni --agent)
done
# Expected:
# npm           npm
# yarn          yarn
# yarn-berry    yarn@berry
# pnpm          pnpm
# bun           bun
# deno          deno
```
