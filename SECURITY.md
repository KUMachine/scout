# Security Policy

## What scout does

`scout` is a **read-only** GitHub CLI wrapper. It lists open PRs, issues, Actions runs, Dependabot alerts, and GitOps release PRs. It does not create, modify, or delete GitHub resources.

All GitHub access goes through the [`gh` CLI](https://cli.github.com/) on your machine. Scout never opens its own network connections.

## Recommended setup

- Use a **fine-grained personal access token** or `gh auth login` scope limited to the repos you watch.
- Read scopes are sufficient: Contents, Issues, Pull requests, Actions, and Dependabot alerts (read).
- Install a **tagged release** rather than tracking `main`:

  ```bash
  cargo install --git https://github.com/KUMachine/repscout --tag v0.1.10 --locked
  ```

## Reporting a vulnerability

Please report security issues privately by opening a [GitHub Security Advisory](https://github.com/KUMachine/repscout/security/advisories/new) or emailing the repository maintainers. Do not file public issues for undisclosed vulnerabilities.

We aim to acknowledge reports within 72 hours.

## Threat model

Scout is designed to resist:

- Malicious contributor PRs that try to run write `gh` commands or spawn arbitrary shells
- Injection via repo slugs or GraphQL queries
- Unexpected network dependencies in the binary

Scout **does not** protect against:

- A compromised `gh` binary or token on your machine
- A malicious maintainer merging harmful code (use pinned releases and review)
- Exfiltration of data you can already read with your `gh` token

Shell completion scripts (`scout complete`) are loaded into your shell — treat changes to `src/complete.rs` as privileged code.
