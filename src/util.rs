use std::collections::BTreeMap;

use anyhow::Result;

use crate::config;
use crate::theme::{dim, paint_warning};

pub const EMPTY_WATCH_MSG: &str =
    "No repos are being watched yet. Add one with `scout add owner/repo`.";

/// Resolve explicit repo args or fall back to the watch list.
pub fn resolve_repos(repo_args: &[String]) -> Result<Vec<String>> {
    if repo_args.is_empty() {
        config::load()
    } else {
        repo_args
            .iter()
            .map(|r| config::validate_slug(r))
            .collect()
    }
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
