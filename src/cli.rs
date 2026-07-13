use anyhow::Result;

use crate::checks::{actions, gitops, issues, prs, snapshot, vulnerabilities};
use crate::config;
use crate::gh;
use crate::render;
use crate::theme::{self, Theme, bold, paint_danger, paint_ok, paint_repo, paint_warning};
use crate::util::{
    self, map_repos_parallel, print_config_line, repos_for_inspect, resolve_repos, soft_result,
    soft_vec,
};

pub fn apply_theme() -> Result<()> {
    let configured = config::load_theme()?;
    theme::apply(theme::effective_from_config(configured));
    Ok(())
}

pub fn add(repo: &str) -> Result<()> {
    let slug = util::resolve_repo_arg(repo)?;
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

pub fn remove(repo: &str) -> Result<()> {
    let slug = util::resolve_repo_arg(repo)?;
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

pub fn list() -> Result<()> {
    let repos = config::load()?;
    if repos.is_empty() {
        println!("{}", util::EMPTY_WATCH_MSG);
        return Ok(());
    }

    println!(
        "{} {}",
        bold("Watching"),
        theme::dim(&format!(
            "{} repo{}",
            repos.len(),
            if repos.len() == 1 { "" } else { "s" }
        )),
    );

    for (owner, names) in util::group_by_owner(&repos) {
        println!("{}", theme::dim(&owner));
        for name in names {
            println!("  {}", paint_repo(&name));
        }
    }

    println!();
    print_config_line("config", &config::repos_file()?)
}

pub fn theme(name: Option<String>) -> Result<()> {
    match name {
        None => {
            let current = config::load_theme()?;
            theme::apply(theme::effective_from_config(current));
            println!("{} {}", bold("theme"), paint_ok(current.as_str()));
            if std::env::var_os("NO_COLOR").is_some() {
                println!("{}", theme::dim("(NO_COLOR set — output is mono for this run)"));
            }
            println!();
            for t in Theme::ALL {
                let marker = if *t == current { "*" } else { " " };
                println!("{marker} {}", t.as_str());
            }
            println!();
            print_config_line("config", &config::config_file()?)?;
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

pub fn check(repo_args: &[String]) -> Result<()> {
    gh::ensure_available()?;
    let Some(repos) = repos_for_inspect(repo_args)? else {
        return Ok(());
    };

    let (app_repos, gitops_repos): (Vec<String>, Vec<String>) = repos
        .into_iter()
        .partition(|r| !gitops::GitopsStatus::is_gitops_repo(r));

    let (counts, runs_by_repo, gitops_statuses) = std::thread::scope(|s| {
        let counts_h = s.spawn(|| fetch_overview_counts(&app_repos));
        let runs_h = s.spawn(|| {
            if app_repos.is_empty() {
                return Vec::new();
            }
            map_repos_parallel(&app_repos, |repo| {
                soft_result(repo, "Actions", actions::inspect(repo), actions::BranchReport::default)
            })
        });
        let gitops_h = s.spawn(|| {
            map_repos_parallel(&gitops_repos, |repo| {
                soft_result(
                    repo,
                    "gitops",
                    gitops::inspect(repo),
                    || gitops::GitopsStatus {
                        repo: repo.to_string(),
                        ..Default::default()
                    },
                )
            })
        });
        (
            counts_h.join().expect("overview thread panicked"),
            runs_h.join().expect("actions thread panicked"),
            gitops_h.join().expect("gitops thread panicked"),
        )
    });

    if !app_repos.is_empty() {
        let summaries = app_repos
            .iter()
            .zip(counts)
            .zip(runs_by_repo)
            .map(|((repo, snap), branches)| render::RepoSummary {
                repo: repo.clone(),
                prs: snap.human_prs,
                bot_prs: snap.bot_prs,
                issues: snap.issues,
                main: branches.main,
                dev: branches.dev,
                alerts: snap.alerts,
            })
            .collect::<Vec<_>>();
        render::summary_table(&summaries);
    }

    render::gitops_section(&gitops_statuses);
    Ok(())
}

fn fetch_overview_counts(app_repos: &[String]) -> Vec<snapshot::RepoCounts> {
    if app_repos.is_empty() {
        return Vec::new();
    }
    match snapshot::fetch_counts(app_repos) {
        Ok(counts) => counts,
        Err(err) => {
            eprintln!(
                "{}: batched overview failed ({err:#}); falling back to per-repo fetches",
                paint_warning("warning"),
            );
            map_repos_parallel(app_repos, |repo| {
                let prs = soft_vec(repo, "PRs", prs::fetch(repo));
                let issues = soft_vec(repo, "issues", issues::fetch(repo));
                let alerts = soft_vec(repo, "vulnerability", vulnerabilities::fetch(repo));
                snapshot::RepoCounts {
                    human_prs: prs.iter().filter(|p| !p.author.is_bot()).count(),
                    bot_prs: prs.iter().filter(|p| p.author.is_bot()).count(),
                    issues: issues.len(),
                    alerts: vulnerabilities::AlertSummary::from_alerts(&alerts),
                }
            })
        }
    }
}

pub fn prs(repo_args: &[String]) -> Result<()> {
    focused(repo_args, "PRs", "No open PRs", prs::fetch, |repo, items| {
        let refs: Vec<&_> = items.iter().collect();
        render::repo_report(repo, &refs, &[], &[], &[]);
    })
}

pub fn issues(repo_args: &[String]) -> Result<()> {
    focused(
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

pub fn actions(repo_args: &[String]) -> Result<()> {
    gh::ensure_available()?;
    let Some(repos) = repos_for_inspect(repo_args)? else {
        return Ok(());
    };

    let reports = map_repos_parallel(&repos, |repo| {
        soft_result(repo, "Actions", actions::inspect(repo), actions::BranchReport::default)
    });

    for (repo, report) in repos.iter().zip(reports) {
        render::actions_report(repo, &report);
    }
    Ok(())
}

pub fn gitops(repo_args: &[String]) -> Result<()> {
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

    let statuses = map_repos_parallel(&repos, |repo| {
        soft_result(
            repo,
            "gitops",
            gitops::inspect(repo),
            || gitops::GitopsStatus {
                repo: repo.to_string(),
                ..Default::default()
            },
        )
    });
    render::gitops_section(&statuses);
    Ok(())
}

pub fn vulns(repo_args: &[String]) -> Result<()> {
    focused(
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

fn focused<T: Send>(
    repo_args: &[String],
    what: &str,
    empty_msg: &str,
    fetch: impl Fn(&str) -> Result<Vec<T>> + Sync,
    render_one: impl Fn(&str, &[T]),
) -> Result<()> {
    gh::ensure_available()?;
    let Some(repos) = repos_for_inspect(repo_args)? else {
        return Ok(());
    };

    let results = map_repos_parallel(&repos, |repo| soft_vec(repo, what, fetch(repo)));

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
