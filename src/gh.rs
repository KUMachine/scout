//! Sole gateway for invoking the GitHub CLI (`gh`).
//!
//! All subprocess calls go through typed helpers with an argv allowlist so
//! contributors cannot accidentally (or maliciously) run write operations.

use std::io::ErrorKind;
use std::io::Write;
use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::config;

/// Preflight: verify the `gh` binary exists and is authenticated.
pub fn ensure_available() -> Result<()> {
    let auth = match Command::new("gh").args(["auth", "status"]).output() {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            bail!("`gh` CLI not found on PATH. Install the GitHub CLI: https://cli.github.com/");
        }
        Err(e) => return Err(e).context("failed to run `gh auth status`"),
        Ok(out) => out,
    };
    if !auth.status.success() {
        bail!(
            "`gh` is not authenticated. Run `gh auth login`.\n{}",
            String::from_utf8_lossy(&auth.stderr).trim()
        );
    }
    Ok(())
}

/// List open pull requests via `gh pr list`.
pub fn list_prs<T: DeserializeOwned>(repo: &str, fields: &str) -> Result<Vec<T>> {
    let repo = config::validate_slug(repo)?;
    validate_json_fields(fields)?;
    run_json(&["pr", "list", "--repo", &repo, "--json", fields])
}

/// List open pull requests with `--state open` (gitops release PRs).
pub fn list_open_prs<T: DeserializeOwned>(repo: &str, fields: &str) -> Result<Vec<T>> {
    let repo = config::validate_slug(repo)?;
    validate_json_fields(fields)?;
    run_json(&[
        "pr", "list", "--repo", &repo, "--state", "open", "--json", fields,
    ])
}

/// List open issues via `gh issue list`.
pub fn list_issues<T: DeserializeOwned>(repo: &str, fields: &str) -> Result<Vec<T>> {
    let repo = config::validate_slug(repo)?;
    validate_json_fields(fields)?;
    run_json(&["issue", "list", "--repo", &repo, "--json", fields])
}

/// List workflow runs on a watched branch via `gh run list`.
pub fn list_runs<T: DeserializeOwned>(repo: &str, branch: &str, fields: &str) -> Result<Vec<T>> {
    let repo = config::validate_slug(repo)?;
    validate_branch(branch)?;
    validate_json_fields(fields)?;
    run_json(&[
        "run", "list", "--repo", &repo, "--branch", branch, "--limit", "20", "--json", fields,
    ])
}

/// Fetch open Dependabot alerts (GET `repos/{owner}/{repo}/dependabot/alerts`).
pub fn dependabot_alerts<T: DeserializeOwned>(repo: &str) -> Result<Vec<T>> {
    let repo = config::validate_slug(repo)?;
    let path = format!("repos/{repo}/dependabot/alerts?state=open");
    validate_dependabot_path(&path)?;
    run_json(&["api", &path])
}

/// Run a read-only GraphQL query via `gh api graphql`.
pub fn run_graphql(query: &str) -> Result<Value> {
    validate_graphql_query(query)?;

    let body = serde_json::json!({ "query": query });
    let mut child = Command::new("gh")
        .args(["api", "graphql", "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn `gh api graphql`")?;

    {
        let mut stdin = child
            .stdin
            .take()
            .context("failed to open stdin for `gh api graphql`")?;
        stdin
            .write_all(body.to_string().as_bytes())
            .context("failed to write GraphQL query to `gh`")?;
    }

    let output = child
        .wait_with_output()
        .context("failed to wait for `gh api graphql`")?;

    let value: Value = serde_json::from_slice(&output.stdout).with_context(|| {
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!(
            "failed to parse GraphQL response: {}",
            stderr.trim().to_string() + &String::from_utf8_lossy(&output.stdout)
        )
    })?;

    if value.get("data").is_some() {
        return Ok(value);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let msg = value
        .pointer("/errors/0/message")
        .and_then(|m| m.as_str())
        .unwrap_or(stderr.trim());
    bail!("GraphQL query failed: {msg}");
}

/// Run a validated `gh` subcommand, capture stdout, and deserialize JSON.
fn run_json<T: DeserializeOwned>(args: &[&str]) -> Result<T> {
    validate_gh_args(args)?;

    let output = Command::new("gh")
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn `gh {}`", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`gh {}` failed: {}", args.join(" "), stderr.trim());
    }

    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("failed to parse JSON output of `gh {}`", args.join(" ")))
}

/// Fail closed on any argv shape we do not explicitly allow.
fn validate_gh_args(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        bail!("refusing empty `gh` invocation");
    }

    for arg in args {
        if arg.contains('\0') {
            bail!("refusing `gh` argument containing a NUL byte");
        }
        if arg.starts_with('-') && !is_allowed_flag(arg) {
            bail!("refusing disallowed `gh` flag: `{arg}`");
        }
    }

    match args[0] {
        "pr" => validate_pr_args(args),
        "issue" => validate_issue_args(args),
        "run" => validate_run_args(args),
        "api" => validate_api_args(args),
        _ => bail!("refusing disallowed `gh` subcommand: `{}`", args[0]),
    }
}

fn is_allowed_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--repo" | "--json" | "--branch" | "--limit" | "--state" | "--input"
    )
}

fn validate_pr_args(args: &[&str]) -> Result<()> {
    if expect_shape(
        args,
        &["pr", "list", "--repo", "*", "--json", "*"],
        "gh pr list",
    )
    .is_ok()
    {
        return Ok(());
    }
    expect_shape(
        args,
        &["pr", "list", "--repo", "*", "--state", "open", "--json", "*"],
        "gh pr list --state open",
    )
}

fn validate_issue_args(args: &[&str]) -> Result<()> {
    expect_shape(
        args,
        &["issue", "list", "--repo", "*", "--json", "*"],
        "gh issue list",
    )?;
    Ok(())
}

fn validate_run_args(args: &[&str]) -> Result<()> {
    expect_shape(
        args,
        &[
            "run",
            "list",
            "--repo",
            "*",
            "--branch",
            "*",
            "--limit",
            "20",
            "--json",
            "*",
        ],
        "gh run list",
    )?;
    Ok(())
}

fn validate_api_args(args: &[&str]) -> Result<()> {
    if args.len() == 3 && args[0] == "api" && args[1] == "graphql" && args[2] == "--input" {
        return Ok(());
    }
    if args.len() == 2 && args[0] == "api" {
        validate_dependabot_path(args[1])?;
        return Ok(());
    }
    bail!("refusing disallowed `gh api` invocation");
}

fn validate_dependabot_path(path: &str) -> Result<()> {
    const PREFIX: &str = "repos/";
    const SUFFIX: &str = "/dependabot/alerts?state=open";

    let slug = path
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_suffix(SUFFIX))
        .ok_or_else(|| anyhow::anyhow!("refusing disallowed `gh api` path: `{path}`"))?;
    config::validate_slug(slug)?;
    Ok(())
}

fn validate_json_fields(fields: &str) -> Result<()> {
    if fields.is_empty()
        || !fields
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == ',')
    {
        bail!("refusing disallowed `--json` fields: `{fields}`");
    }
    Ok(())
}

fn validate_branch(branch: &str) -> Result<()> {
    if branch.is_empty()
        || !branch
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/')
    {
        bail!("refusing disallowed branch name: `{branch}`");
    }
    Ok(())
}

fn validate_graphql_query(query: &str) -> Result<()> {
    let trimmed = query.trim();
    if !trimmed.starts_with("query {") && !trimmed.starts_with("query{") {
        bail!("refusing GraphQL: only read `query` operations are allowed");
    }

    let lower = trimmed.to_ascii_lowercase();
    for forbidden in [
        "mutation",
        "subscription",
        "__schema",
        "__type",
        "create",
        "update",
        "delete",
    ] {
        if lower.contains(forbidden) {
            bail!("refusing GraphQL: forbidden token `{forbidden}`");
        }
    }
    Ok(())
}

/// Match argv against a template (`"*"` = any single token).
fn expect_shape(args: &[&str], template: &[&str], label: &str) -> Result<()> {
    if args.len() != template.len() {
        bail!("refusing disallowed `{label}` argv shape");
    }
    for (arg, expected) in args.iter().zip(template.iter()) {
        if *expected != "*" && arg != expected {
            bail!("refusing disallowed `{label}` argv shape");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_pr_list_shape() {
        validate_gh_args(&[
            "pr",
            "list",
            "--repo",
            "octo/repo",
            "--json",
            "number,title",
        ])
        .unwrap();
    }

    #[test]
    fn allows_open_pr_list_shape() {
        validate_gh_args(&[
            "pr",
            "list",
            "--repo",
            "octo/repo",
            "--state",
            "open",
            "--json",
            "number,title",
        ])
        .unwrap();
    }

    #[test]
    fn rejects_pr_merge() {
        assert!(validate_gh_args(&["pr", "merge", "1"]).is_err());
    }

    #[test]
    fn rejects_api_post() {
        assert!(validate_gh_args(&["api", "-X", "POST", "repos/o/r/issues"]).is_err());
    }

    #[test]
    fn allows_dependabot_path() {
        validate_dependabot_path("repos/octo/repo/dependabot/alerts?state=open").unwrap();
    }

    #[test]
    fn rejects_arbitrary_api_path() {
        assert!(validate_dependabot_path("repos/octo/repo/hooks").is_err());
    }

    #[test]
    fn rejects_graphql_mutation() {
        assert!(validate_graphql_query("mutation { deleteRepository }").is_err());
    }

    #[test]
    fn allows_read_query() {
        validate_graphql_query("query { repository(owner: \"o\", name: \"r\") { name } }").unwrap();
    }

    #[test]
    fn rejects_disallowed_json_fields() {
        assert!(validate_json_fields("number;rm -rf").is_err());
    }
}
