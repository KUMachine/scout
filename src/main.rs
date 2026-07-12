mod checks;
mod cli;
mod config;
mod gh;
mod render;
mod theme;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::theme::paint_danger;

#[derive(Parser)]
#[command(
    name = "scout",
    about = "Watch a list of GitHub repos and see what's currently open",
    version,
    after_help = "\
Watch list:
  add, remove, list

Inspect:
  check   compact overview table
  prs     open pull requests
  issues  open issues
  actions main/dev Actions status
  gitops  staging/prod release PRs
  vulns   open Dependabot alerts

Settings:
  theme   view or set color theme (cool, classic, mono)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Repos to check (`owner/repo` …); defaults to the watch list.
#[derive(clap::Args)]
struct InspectArgs {
    repos: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Add a repo to the watch list
    Add {
        /// Repository in `owner/repo` form
        repo: String,
    },
    /// Remove a repo from the watch list
    Remove {
        /// Repository in `owner/repo` form
        repo: String,
    },
    /// Print the watch list
    List,

    /// Compact overview of open PRs, issues, failed runs, and vulns
    Check(InspectArgs),
    /// List open pull requests
    Prs(InspectArgs),
    /// List open issues
    Issues(InspectArgs),
    /// Show main/dev Actions status (and any current failures)
    Actions(InspectArgs),
    /// Show staging/prod GitOps release status
    Gitops(InspectArgs),
    /// List open Dependabot vulnerability alerts
    Vulns(InspectArgs),
    /// View or set the color theme
    Theme {
        /// Theme name: cool, classic, or mono (omit to list)
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("{}: {:#}", paint_danger("error"), err);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    if !matches!(cli.command, Command::Theme { .. }) {
        cli::apply_theme()?;
    }

    match cli.command {
        Command::Add { repo } => cli::add(&repo),
        Command::Remove { repo } => cli::remove(&repo),
        Command::List => cli::list(),
        Command::Check(args) => cli::check(&args.repos),
        Command::Prs(args) => cli::prs(&args.repos),
        Command::Issues(args) => cli::issues(&args.repos),
        Command::Actions(args) => cli::actions(&args.repos),
        Command::Gitops(args) => cli::gitops(&args.repos),
        Command::Vulns(args) => cli::vulns(&args.repos),
        Command::Theme { name } => cli::theme(name),
    }
}
