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
| `scout check`             | Compact overview (PRs, issues, `dev`/`main` Actions, vuln severities) |
| `scout prs`               | Open pull requests (human first, Dependabot dimmed)                   |
| `scout issues`            | Open issues                                                           |
| `scout actions`           | `dev` / `main` Actions status + current failures                      |
| `scout vulns`             | Open Dependabot alerts (critical → low)                               |

Pass optional `owner/repo …` args to any inspect command to override the watch list for that run.

## Config

Watch list lives at:

- macOS: `~/Library/Application Support/scout/repos.json`
- Linux: `~/.config/scout/repos.json`
