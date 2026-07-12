mod checks;
mod config;
mod gh;
mod render;

use anyhow::Result;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use crate::checks::{actions, issues, prs, vulnerabilities};

#[derive(Parser)]
#[command(
    name = "scout",
    about = "Watch a list of GitHub repos and see what's currently open",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
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
    /// Show current open PRs, issues, failed runs, and vulnerability alerts
    Check {
        /// Repos to check (owner/repo ...); defaults to the watch list
        repos: Vec<String>,
        /// Print the full per-item breakdown instead of the compact table
        #[arg(long)]
        detailed: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        // One clean top-level message, not a Rust panic / backtrace wall.
        eprintln!("{}: {:#}", "error".red().bold(), err);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Add { repo } => cmd_add(&repo),
        Command::Remove { repo } => cmd_remove(&repo),
        Command::List => cmd_list(),
        Command::Check { repos, detailed } => cmd_check(repos, detailed),
    }
}

fn cmd_add(repo: &str) -> Result<()> {
    let slug = config::validate_slug(repo)?;
    let mut repos = config::load()?;
    if repos.iter().any(|r| r == &slug) {
        println!("{slug} is already on the watch list.");
        return Ok(());
    }
    repos.push(slug.clone());
    repos.sort();
    config::save(&repos)?;
    println!("{} added {slug}", "+".green().bold());
    Ok(())
}

fn cmd_remove(repo: &str) -> Result<()> {
    let slug = config::validate_slug(repo)?;
    let mut repos = config::load()?;
    let before = repos.len();
    repos.retain(|r| r != &slug);
    if repos.len() == before {
        println!("{slug} is not on the watch list.");
        return Ok(());
    }
    config::save(&repos)?;
    println!("{} removed {slug}", "-".red().bold());
    Ok(())
}

fn cmd_list() -> Result<()> {
    let repos = config::load()?;
    if repos.is_empty() {
        println!("No repos are being watched yet. Add one with `scout add owner/repo`.");
        return Ok(());
    }
    println!("Watching {} repo(s):", repos.len());
    for repo in &repos {
        println!("  {repo}");
    }
    Ok(())
}

/// Run a per-repo check, downgrading any failure to a stderr warning so
/// that one bad repo/check never aborts the whole run.
fn soft<T>(repo: &str, what: &str, result: Result<Vec<T>>) -> Vec<T> {
    match result {
        Ok(items) => items,
        Err(err) => {
            eprintln!(
                "{}: {what} check on {repo} failed: {:#}",
                "warning".yellow().bold(),
                err
            );
            Vec::new()
        }
    }
}

fn cmd_check(repo_args: Vec<String>, detailed: bool) -> Result<()> {
    gh::ensure_available()?;

    let repos = if repo_args.is_empty() {
        config::load()?
    } else {
        repo_args
            .iter()
            .map(|r| config::validate_slug(r))
            .collect::<Result<Vec<_>>>()?
    };

    if repos.is_empty() {
        println!("No repos are being watched yet. Add one with `scout add owner/repo`.");
        return Ok(());
    }

    let mut total = 0usize;
    // Compact-table rows, only populated when not running --detailed.
    let mut summaries: Vec<render::RepoSummary> = Vec::new();

    for repo in &repos {
        let prs = soft(repo, "PRs", prs::fetch(repo));
        let issues = soft(repo, "issues", issues::fetch(repo));
        let runs = soft(repo, "Actions", actions::fetch(repo));
        let alerts = soft(repo, "vulnerability", vulnerabilities::fetch(repo));

        let has_any =
            !(prs.is_empty() && issues.is_empty() && runs.is_empty() && alerts.is_empty());
        if !has_any {
            continue;
        }
        total += prs.len() + issues.len() + runs.len() + alerts.len();

        if detailed {
            let pr_refs: Vec<&_> = prs.iter().collect();
            let issue_refs: Vec<&_> = issues.iter().collect();
            let run_refs: Vec<&_> = runs.iter().collect();
            let alert_refs: Vec<&_> = alerts.iter().collect();
            render::repo_report(repo, &pr_refs, &issue_refs, &run_refs, &alert_refs);
        } else {
            summaries.push(render::RepoSummary {
                repo: repo.clone(),
                prs: prs.len(),
                issues: issues.len(),
                runs: runs.len(),
                alerts: alerts.len(),
            });
        }
    }

    if total == 0 {
        println!("Nothing open across {} repo(s).", repos.len());
    } else if !detailed {
        render::summary_table(&summaries);
    }

    Ok(())
}
