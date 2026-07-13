# scout

Watch a list of GitHub repos and see what's currently open — PRs, issues, Actions (`dev` / `main`), and Dependabot vulns.

Requires the [GitHub CLI](https://cli.github.com/) (`gh`) authenticated on your machine.

## Install

```bash
cargo install --git https://github.com/KUMachine/scout --locked
```

That builds a release binary named `scout` into `~/.cargo/bin`.

Update to the latest `main`:

```bash
cargo install --git https://github.com/KUMachine/scout --locked --force
```

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

