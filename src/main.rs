mod checks;
mod cli;
mod complete;
mod config;
mod gh;
mod render;
mod theme;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{ArgValueCandidates, ArgValueCompleter, Shell};
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
  theme    view or set color theme (cool, classic, claude, discord, mono)
  complete print shell tab-completion script (bash, zsh, fish, …)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Repos to check (`owner/repo` …); defaults to the watch list.
#[derive(clap::Args)]
struct InspectArgs {
    #[arg(add = ArgValueCompleter::new(complete::repos))]
    repos: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Add a repo to the watch list
    Add {
        /// Repository in `owner/repo` form
        #[arg(add = ArgValueCompleter::new(complete::repos))]
        repo: String,
    },
    /// Remove a repo from the watch list
    Remove {
        /// Repository in `owner/repo` form
        #[arg(add = ArgValueCompleter::new(complete::repos))]
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
        /// Theme name: cool, classic, claude, discord, or mono (omit to list)
        #[arg(add = ArgValueCandidates::new(complete::themes))]
        name: Option<String>,
    },
    /// Print shell tab-completion script (pipe into eval/source)
    Complete {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("{}: {:#}", paint_danger("error"), err);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    if !matches!(
        cli.command,
        Command::Theme { .. } | Command::Complete { .. }
    ) {
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
        Command::Complete { shell } => {
            complete::emit_registration(&shell.to_string(), Cli::command)
        }
    }
}
