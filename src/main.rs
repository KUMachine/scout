mod checks;
mod config;
mod gh;
mod render;
mod theme;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::checks::{actions, gitops, issues, prs, snapshot, vulnerabilities};
use crate::theme::{Theme, bold, dim, paint_danger, paint_ok, paint_repo, paint_warning};

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
    Check {
        /// Repos to check (owner/repo ...); defaults to the watch list
        repos: Vec<String>,
    },
    /// List open pull requests
    Prs {
        /// Repos to check (owner/repo ...); defaults to the watch list
        repos: Vec<String>,
    },
    /// List open issues
    Issues {
        /// Repos to check (owner/repo ...); defaults to the watch list
        repos: Vec<String>,
    },
    /// Show main/dev Actions status (and any current failures)
    Actions {
        /// Repos to check (owner/repo ...); defaults to the watch list
        repos: Vec<String>,
    },
    /// Show staging/prod GitOps release status
    Gitops {
        /// GitOps repos (owner/repo ...); defaults to `*-gitops` on the watch list
        repos: Vec<String>,
    },
    /// List open Dependabot vulnerability alerts
    Vulns {
        /// Repos to check (owner/repo ...); defaults to the watch list
        repos: Vec<String>,
    },
    /// View or set the color theme
    Theme {
        /// Theme name: cool, classic, or mono (omit to list)
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        // One clean top-level message, not a Rust panic / backtrace wall.
        // Theme may not be applied yet if init failed early.
        eprintln!("{}: {:#}", paint_danger("error"), err);
        std::process::exit(1);
    }
}

fn apply_theme() -> Result<()> {
    let configured = config::load_theme()?;
    theme::apply(theme::effective_from_config(configured));
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    // Theme command applies after resolving; everything else needs paint helpers ready.
    if !matches!(cli.command, Command::Theme { .. }) {
        apply_theme()?;
    }

    match cli.command {
        Command::Add { repo } => cmd_add(&repo),
        Command::Remove { repo } => cmd_remove(&repo),
        Command::List => cmd_list(),
        Command::Check { repos } => cmd_check(repos),
        Command::Prs { repos } => cmd_prs(repos),
        Command::Issues { repos } => cmd_issues(repos),
        Command::Actions { repos } => cmd_actions(repos),
        Command::Gitops { repos } => cmd_gitops(repos),
        Command::Vulns { repos } => cmd_vulns(repos),
        Command::Theme { name } => cmd_theme(name),
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
    println!("{} added {slug}", paint_ok("+"));
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
    println!("{} removed {slug}", paint_danger("-"));
    Ok(())
}

fn cmd_list() -> Result<()> {
    let repos = config::load()?;
    if repos.is_empty() {
        println!("No repos are being watched yet. Add one with `scout add owner/repo`.");
        return Ok(());
    }

    println!(
        "{} {}",
        bold("Watching"),
        dim(&format!(
            "{} repo{}",
            repos.len(),
            if repos.len() == 1 { "" } else { "s" }
        )),
    );

    // Group by owner so org-heavy watch lists stay scannable.
    let mut by_owner: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for repo in &repos {
        let (owner, name) = repo.split_once('/').unwrap_or(("", repo.as_str()));
        by_owner
            .entry(owner.to_string())
            .or_default()
            .push(name.to_string());
    }

    for (owner, names) in &by_owner {
        println!("{}", dim(owner));
        for name in names {
            println!("  {}", paint_repo(name));
        }
    }

    println!();
    println!(
        "{} {}",
        dim("config"),
        dim(&config::repos_file()?.display().to_string())
    );
    Ok(())
}

fn cmd_theme(name: Option<String>) -> Result<()> {
    match name {
        None => {
            let current = config::load_theme()?;
            theme::apply(theme::effective_from_config(current));
            println!("{} {}", bold("theme"), paint_ok(current.as_str()));
            if std::env::var_os("NO_COLOR").is_some() {
                println!("{}", dim("(NO_COLOR set — output is mono for this run)"));
            }
            println!();
            for t in Theme::ALL {
                let marker = if *t == current { "*" } else { " " };
                println!("{marker} {}", t.as_str());
            }
            println!();
            println!(
                "{} {}",
                dim("config"),
                dim(&config::config_file()?.display().to_string())
            );
        }
        Some(name) => {
            let theme = Theme::parse(&name)?;
            config::save_theme(theme)?;
            theme::apply(theme::effective_from_config(theme));
            println!("{} theme set to {}", paint_ok("✓"), paint_ok(theme.as_str()));
        }
    }
    Ok(())
}

/// Resolve the target repo list: explicit args, or the watch list.
fn resolve_repos(repo_args: Vec<String>) -> Result<Vec<String>> {
    if repo_args.is_empty() {
        config::load()
    } else {
        repo_args
            .iter()
            .map(|r| config::validate_slug(r))
            .collect()
    }
}

/// Run a per-repo check, downgrading any failure to a stderr warning so
/// that one bad repo/check never aborts the whole run.
fn soft<T>(repo: &str, what: &str, result: Result<Vec<T>>) -> Vec<T> {
    match result {
        Ok(items) => items,
        Err(err) => {
            eprintln!(
                "{}: {what} check on {repo} failed: {:#}",
                paint_warning("warning"),
                err
            );
            Vec::new()
        }
    }
}

/// Map over repos in parallel, preserving input order.
fn map_repos_parallel<T: Send>(repos: &[String], f: impl Fn(&str) -> T + Sync) -> Vec<T> {
    std::thread::scope(|s| {
        let handles: Vec<_> = repos.iter().map(|repo| s.spawn(|| f(repo))).collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .collect()
    })
}

fn cmd_check(repo_args: Vec<String>) -> Result<()> {
    gh::ensure_available()?;
    let repos = resolve_repos(repo_args)?;

    if repos.is_empty() {
        println!("No repos are being watched yet. Add one with `scout add owner/repo`.");
        return Ok(());
    }

    let (app_repos, gitops_repos): (Vec<String>, Vec<String>) = repos
        .into_iter()
        .partition(|r| !gitops::GitopsStatus::is_gitops_repo(r));

    // App repos: GraphQL overview + Actions. GitOps repos: release PRs only.
    let (counts, runs_by_repo, gitops_statuses) = std::thread::scope(|s| {
        let counts_h = s.spawn(|| {
            if app_repos.is_empty() {
                return Vec::new();
            }
            match snapshot::fetch_counts(&app_repos) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!(
                        "{}: batched overview failed ({:#}); falling back to per-repo fetches",
                        paint_warning("warning"),
                        err
                    );
                    map_repos_parallel(&app_repos, |repo| {
                        let prs = soft(repo, "PRs", prs::fetch(repo));
                        let issues = soft(repo, "issues", issues::fetch(repo));
                        let alerts = soft(repo, "vulnerability", vulnerabilities::fetch(repo));
                        snapshot::RepoCounts {
                            human_prs: prs.iter().filter(|p| !p.author.is_bot()).count(),
                            bot_prs: prs.iter().filter(|p| p.author.is_bot()).count(),
                            issues: issues.len(),
                            alerts: vulnerabilities::AlertSummary::from_alerts(&alerts),
                        }
                    })
                }
            }
        });
        let runs_h = s.spawn(|| {
            if app_repos.is_empty() {
                return Vec::new();
            }
            map_repos_parallel(&app_repos, |repo| soft_branches(repo, actions::inspect(repo)))
        });
        let gitops_h = s.spawn(|| {
            map_repos_parallel(&gitops_repos, |repo| soft_gitops(repo, gitops::inspect(repo)))
        });
        (
            counts_h.join().expect("overview thread panicked"),
            runs_h.join().expect("actions thread panicked"),
            gitops_h.join().expect("gitops thread panicked"),
        )
    });

    if !app_repos.is_empty() {
        let mut summaries: Vec<render::RepoSummary> = Vec::new();
        for ((repo, snap), branches) in app_repos.iter().zip(counts).zip(runs_by_repo) {
            summaries.push(render::RepoSummary {
                repo: repo.clone(),
                prs: snap.human_prs,
                bot_prs: snap.bot_prs,
                issues: snap.issues,
                main: branches.main,
                dev: branches.dev,
                alerts: snap.alerts,
            });
        }
        render::summary_table(&summaries);
    }

    render::gitops_section(&gitops_statuses);
    Ok(())
}

fn soft_gitops(repo: &str, result: Result<gitops::GitopsStatus>) -> gitops::GitopsStatus {
    match result {
        Ok(n) => n,
        Err(err) => {
            eprintln!(
                "{}: gitops check on {repo} failed: {:#}",
                paint_warning("warning"),
                err
            );
            gitops::GitopsStatus {
                repo: repo.to_string(),
                ..Default::default()
            }
        }
    }
}

fn soft_branches(repo: &str, result: Result<actions::BranchReport>) -> actions::BranchReport {
    match result {
        Ok(n) => n,
        Err(err) => {
            eprintln!(
                "{}: Actions check on {repo} failed: {:#}",
                paint_warning("warning"),
                err
            );
            actions::BranchReport::default()
        }
    }
}

/// Shared loop for focused inspect commands: fetch one category per repo
/// (in parallel), skip quiet repos, and print the full item list.
fn cmd_focused<T: Send>(
    repo_args: Vec<String>,
    what: &str,
    empty_msg: &str,
    fetch: impl Fn(&str) -> Result<Vec<T>> + Sync,
    render_one: impl Fn(&str, &[T]),
) -> Result<()> {
    gh::ensure_available()?;
    let repos = resolve_repos(repo_args)?;

    if repos.is_empty() {
        println!("No repos are being watched yet. Add one with `scout add owner/repo`.");
        return Ok(());
    }

    let results = map_repos_parallel(&repos, |repo| soft(repo, what, fetch(repo)));

    let mut any = false;
    for (repo, items) in repos.iter().zip(results) {
        if items.is_empty() {
            continue;
        }
        any = true;
        render_one(repo, &items);
    }

    if !any {
        println!("{empty_msg} across {} repo(s).", repos.len());
    }
    Ok(())
}

fn cmd_prs(repo_args: Vec<String>) -> Result<()> {
    cmd_focused(repo_args, "PRs", "No open PRs", prs::fetch, |repo, items| {
        let refs: Vec<&_> = items.iter().collect();
        render::repo_report(repo, &refs, &[], &[], &[]);
    })
}

fn cmd_issues(repo_args: Vec<String>) -> Result<()> {
    cmd_focused(
        repo_args,
        "issues",
        "No open issues",
        issues::fetch,
        |repo, items| {
            let refs: Vec<&_> = items.iter().collect();
            render::repo_report(repo, &[], &refs, &[], &[]);
        },
    )
}

fn cmd_actions(repo_args: Vec<String>) -> Result<()> {
    gh::ensure_available()?;
    let repos = resolve_repos(repo_args)?;

    if repos.is_empty() {
        println!("No repos are being watched yet. Add one with `scout add owner/repo`.");
        return Ok(());
    }

    let reports = map_repos_parallel(&repos, |repo| soft_branches(repo, actions::inspect(repo)));

    for (repo, report) in repos.iter().zip(reports) {
        render::actions_report(repo, &report);
    }
    Ok(())
}

fn cmd_gitops(repo_args: Vec<String>) -> Result<()> {
    gh::ensure_available()?;
    let repos = if repo_args.is_empty() {
        config::load()?
            .into_iter()
            .filter(|r| gitops::GitopsStatus::is_gitops_repo(r))
            .collect::<Vec<_>>()
    } else {
        resolve_repos(repo_args)?
    };

    if repos.is_empty() {
        println!(
            "No gitops repos to inspect. Add one with `scout add owner/something-gitops`."
        );
        return Ok(());
    }

    let statuses = map_repos_parallel(&repos, |repo| soft_gitops(repo, gitops::inspect(repo)));
    render::gitops_section(&statuses);
    Ok(())
}

fn cmd_vulns(repo_args: Vec<String>) -> Result<()> {
    cmd_focused(
        repo_args,
        "vulnerability",
        "No vulnerability alerts",
        vulnerabilities::fetch,
        |repo, items| {
            let refs: Vec<&_> = items.iter().collect();
            render::repo_report(repo, &[], &[], &[], &refs);
        },
    )
}
