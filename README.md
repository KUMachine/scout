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
| `scout remove owner/repo` | Remove a repo                                                         |
| `scout list`              | Print the watch list                                                  |
| `scout check`             | App overview + separate GitOps staging/prod releases                  |
| `scout prs`               | Open pull requests (human first, Dependabot dimmed)                   |
| `scout issues`            | Open issues                                                           |
| `scout actions`           | `dev` / `main` Actions status + current failures                      |
| `scout gitops`            | Staging/prod release PRs for `*-gitops` repos                         |
| `scout vulns`             | Open Dependabot alerts (critical → low)                               |
| `scout theme [name]`      | View or set color theme (`cool`, `classic`, `claude`, `discord`, `mono`) |

Pass optional `owner/repo …` args to any inspect command to override the watch list for that run.

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

