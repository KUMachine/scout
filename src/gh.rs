use std::io::ErrorKind;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Preflight: verify the `gh` binary exists and is authenticated.
/// Produces a single clear error rather than letting individual checks
/// fail with cryptic output.
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

/// Run a `gh` subcommand, capture stdout, and deserialize it as JSON.
/// A non-zero exit status is surfaced as an error carrying gh's stderr
/// (so callers can treat e.g. a 403 as a soft, per-repo warning).
pub fn run_json<T: DeserializeOwned>(args: &[&str]) -> Result<T> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn `gh {}`", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`gh {}` failed: {}", args.join(" "), stderr.trim());
    }

    let value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("failed to parse JSON output of `gh {}`", args.join(" ")))?;
    Ok(value)
}

/// Run a GraphQL query via `gh api graphql`.
///
/// GitHub often returns partial `data` alongside an `errors` array (and `gh`
/// exits non-zero). We accept that shape so one forbidden field (e.g. vuln
/// alerts) does not throw away PRs/issues from the same query.
pub fn run_graphql(query: &str) -> Result<Value> {
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
