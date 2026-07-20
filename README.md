# scout

Watch a list of GitHub repos and see what's currently open — PRs, issues, Actions (`dev` / `main`), and Dependabot vulns.

Requires the [GitHub CLI](https://cli.github.com/) (`gh`) authenticated on your machine.

## Install

```bash
cargo install --git https://github.com/KUMachine/repscout --tag v0.1.10 --locked
```

That builds a release binary named `scout` into `~/.cargo/bin`.

Update to a newer release:

```bash
cargo install --git https://github.com/KUMachine/repscout --tag v0.1.10 --locked --force
```

Prefer installing a **tagged release** over tracking `main` so you get a known, reviewable version.

See [CHANGELOG.md](CHANGELOG.md) for release notes.

### Requirements

- Recent Rust toolchain (`edition = "2024"`)
- `gh` on `PATH`, logged in (`gh auth login`)

## Quick start

```bash
scout add owner/repo
scout list
scout check
```

## Commands

| Command                   | What it does                                                          |
| ------------------------- | --------------------------------------------------------------------- |
| `scout add owner/repo`    | Add a repo to the watch list                                          |
| `scout add .`             | Add the current git repo (from `origin`)                              |
| `scout remove owner/repo` | Remove a repo                                                         |
| `scout remove .`          | Remove the current git repo                                           |
| `scout list`              | Print the watch list                                                  |
| `scout check`             | App overview + separate GitOps staging/prod releases                  |
| `scout prs`               | Open pull requests (human first, Dependabot dimmed)                   |
| `scout issues`            | Open issues                                                           |
| `scout actions`           | `dev` / `main` Actions status + current failures                      |
| `scout gitops`            | Staging/prod release PRs for `*-gitops` repos                         |
| `scout vulns`             | Open Dependabot alerts (critical → low)                               |
| `scout theme [name]`      | View or set color theme (`cool`, `classic`, `claude`, `discord`, `mono`) |
| `scout complete <shell>`  | Print tab-completion script (`bash`, `zsh`, `fish`, `elvish`, `powershell`) |

Pass optional `owner/repo …` args to any inspect command to override the watch list for that run. Use `.` to target only the current git repo (even if it is not on the watch list), e.g. `scout check .` or `scout vulns .`.

Repos ending in `-gitops` are skipped in the app table and shown under **GitOps** instead (`stg[●]` / `prod[✓]`), with waiting services parsed from the release PR body.

## Config

Watch list:

- macOS: `~/Library/Application Support/scout/repos.json`
- Linux: `~/.config/scout/repos.json`

Theme (`cool` by default — missing file means cool, nothing is created until you set one):

```bash
scout theme           # current + available
scout theme classic   # green/yellow/red
scout theme cool      # cyan/blue/magenta
scout theme claude    # coral/teal/amber (Anthropic palette)
scout theme discord   # blurple/green/magenta (Discord palette)
scout theme mono      # no color
```

Stored in `config.json` next to `repos.json`. Set `NO_COLOR` in the environment to force mono for a single run without changing the saved theme.

## Shell completion

Tab-complete subcommands, theme names, and repos from your watch list.

**zsh** (recommended — re-source on each shell start so completions stay in sync):

```bash
echo 'source <(scout complete zsh)' >> ~/.zshrc
```

**bash**:

```bash
echo 'eval "$(scout complete bash)"' >> ~/.bashrc
```

**fish**:

```bash
scout complete fish > ~/.config/fish/completions/scout.fish
```

Supported shells: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

Disable with `COMPLETE=` or `COMPLETE=0` in the environment.

## Security

Scout is **read-only**: it only lists PRs, issues, Actions, and Dependabot alerts via `gh`. It does not create or modify GitHub resources and has no direct network stack.

- All `gh` invocations go through an audited allowlist in `src/gh.rs`.
- CI blocks new subprocess or network dependencies outside that gateway.
- Use a read-scoped `gh` token limited to the repos you watch when possible.

See [SECURITY.md](SECURITY.md) for the threat model and how to report issues.

## Releasing

`scout` uses [SemVer](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/).

1. Add bullets under `[Unreleased]` in [CHANGELOG.md](CHANGELOG.md) for user-visible changes.
2. Run `./scripts/release.sh X.Y.Z` (bumps `Cargo.toml`, folds the changelog, updates install pins, commits, and tags).
3. Push: `git push origin main --tags` — CI creates a GitHub Release from the changelog section.

Versioning while still `0.x`: **patch** for fixes/polish, **minor** for new commands or behavior, **major** (`1.0.0`) when the CLI contract is stable.

