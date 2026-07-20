# Changelog

All notable changes to `scout` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.10] - 2026-07-20

- Add release cycle scripts and workflows


## [0.1.9] - 2026-07-20
## [0.1.9] - 2026-07-20

### Changed

- Make PR, issue, alert, and Actions URLs clickable via OSC 8 hyperlinks (Cmd+click in supported terminals)

## [0.1.8] - 2026-07-16
## [0.1.8] - 2026-07-16

### Added

- `scout add .` and `scout remove .` to manage the current git repo from `origin`
- `SECURITY.md`, CI security checks, and audited read-only `gh` gateway

### Changed

- Tighten argument validation for `gh pr list` and `gh run list`

## [0.1.7] - 2026-07-13
## [0.1.7] - 2026-07-13

### Added

- Shell tab completion via `scout complete` (bash, zsh, fish, elvish, powershell)

## [0.1.6] - 2026-07-12
## [0.1.6] - 2026-07-12

### Added

- `claude` and `discord` color themes

## [0.1.5] - 2026-07-11
## [0.1.5] - 2026-07-11

### Changed

- Refactor CLI into subcommands (`add`, `remove`, `list`, `check`, `prs`, `issues`, `actions`, `vulns`, `theme`)

## [0.1.4] - 2026-07-10
## [0.1.4] - 2026-07-10

### Added

- `scout theme` command with persistent theme config (`cool`, `classic`, `mono`)

## [0.1.3] - 2026-07-09
## [0.1.3] - 2026-07-09

### Added

- `scout gitops` command for staging/production release PRs on `*-gitops` repos

## [0.1.2] - 2026-07-09
## [0.1.2] - 2026-07-09

### Added

- GitOps section in `scout check` for `*-gitops` repos (staging/prod release status)

## [0.1.1] - 2026-07-08
## [0.1.1] - 2026-07-08

### Changed

- Group watched repos by owner in `scout list`
