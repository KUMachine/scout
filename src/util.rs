use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::theme::{dim, paint_warning};

pub const EMPTY_WATCH_MSG: &str =
    "No repos are being watched yet. Add one with `scout add owner/repo`.";

/// Resolve a repo argument: `.` means the current git repo, otherwise `owner/repo`.
pub fn resolve_repo_arg(slug: &str) -> Result<String> {
    let slug = slug.trim();
    if slug == "." {
        return current_repo_slug();
    }
    config::validate_slug(slug)
}

/// Resolve explicit repo args or fall back to the watch list.
pub fn resolve_repos(repo_args: &[String]) -> Result<Vec<String>> {
    if repo_args.is_empty() {
        config::load()
    } else {
        repo_args.iter().map(|r| resolve_repo_arg(r)).collect()
    }
}

/// Detect `owner/repo` for the git repository containing the current directory.
pub fn current_repo_slug() -> Result<String> {
    let in_tree = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("failed to run `git rev-parse`")?;

    if !in_tree.status.success()
        || String::from_utf8_lossy(&in_tree.stdout).trim() != "true"
    {
        bail!("not inside a git repository (`.` refers to the current repo)");
    }

    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("failed to run `git remote get-url origin`")?;

    if !output.status.success() {
        bail!("no `origin` remote found (`.` refers to the current repo's GitHub slug)");
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_github_remote(&url)
}

fn parse_github_remote(url: &str) -> Result<String> {
    let url = url.trim().trim_end_matches(".git");

    if let Some(rest) = url.strip_prefix("git@github.com:") {
        return config::validate_slug(rest);
    }
    if let Some(rest) = url.strip_prefix("ssh://git@github.com/") {
        return config::validate_slug(rest);
    }
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        return config::validate_slug(rest);
    }
    if let Some(rest) = url.strip_prefix("http://github.com/") {
        return config::validate_slug(rest);
    }

    bail!("could not parse GitHub repo from remote URL `{url}`");
}

/// Load repos for an inspect command; print the empty-watch hint and return `None` when empty.
pub fn repos_for_inspect(repo_args: &[String]) -> Result<Option<Vec<String>>> {
    let repos = resolve_repos(repo_args)?;
    if repos.is_empty() {
        println!("{EMPTY_WATCH_MSG}");
        return Ok(None);
    }
    Ok(Some(repos))
}

/// Map over repos in parallel, preserving input order.
pub fn map_repos_parallel<T: Send>(repos: &[String], f: impl Fn(&str) -> T + Sync) -> Vec<T> {
    std::thread::scope(|s| {
        let handles: Vec<_> = repos.iter().map(|repo| s.spawn(|| f(repo))).collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .collect()
    })
}

/// Per-repo fetch that logs a warning and returns an empty list on failure.
pub fn soft_vec<T>(repo: &str, what: &str, result: Result<Vec<T>>) -> Vec<T> {
    match result {
        Ok(items) => items,
        Err(err) => {
            warn(repo, what, &err);
            Vec::new()
        }
    }
}

/// Per-repo inspect that logs a warning and returns a caller-provided default on failure.
pub fn soft_result<T>(repo: &str, what: &str, result: Result<T>, default: impl FnOnce() -> T) -> T {
    match result {
        Ok(value) => value,
        Err(err) => {
            warn(repo, what, &err);
            default()
        }
    }
}

fn warn(repo: &str, what: &str, err: &anyhow::Error) {
    eprintln!(
        "{}: {what} check on {repo} failed: {err:#}",
        paint_warning("warning"),
    );
}

/// Group `owner/repo` slugs by owner for `scout list` output.
pub fn group_by_owner(repos: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut by_owner: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for repo in repos {
        let (owner, name) = repo.split_once('/').unwrap_or(("", repo.as_str()));
        by_owner
            .entry(owner.to_string())
            .or_default()
            .push(name.to_string());
    }
    by_owner
}

pub fn plural(n: usize, singular: &str) -> String {
    if n == 1 {
        format!("{n} {singular}")
    } else {
        format!("{n} {singular}s")
    }
}

pub fn print_config_line(label: &str, path: &std::path::Path) -> Result<()> {
    println!("{} {}", dim(label), dim(&path.display().to_string()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_github_remote, resolve_repo_arg};

    #[test]
    fn dot_resolves_to_current_repo() {
        assert_eq!(resolve_repo_arg(".").unwrap(), "KUMachine/repscout");
    }

    #[test]
    fn parse_ssh_remote() {
        assert_eq!(
            parse_github_remote("git@github.com:KUMachine/scout.git").unwrap(),
            "KUMachine/scout"
        );
    }

    #[test]
    fn parse_https_remote() {
        assert_eq!(
            parse_github_remote("https://github.com/KUMachine/scout").unwrap(),
            "KUMachine/scout"
        );
    }
}
