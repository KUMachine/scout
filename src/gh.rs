use std::io::ErrorKind;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

/// Preflight: verify the `gh` binary exists and is authenticated.
/// Produces a single clear error rather than letting individual checks
/// fail with cryptic output.
pub fn ensure_available() -> Result<()> {
    match Command::new("gh").arg("--version").output() {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            bail!("`gh` CLI not found on PATH. Install the GitHub CLI: https://cli.github.com/");
        }
        Err(e) => return Err(e).context("failed to run `gh --version`"),
        Ok(out) if !out.status.success() => {
            bail!(
                "`gh --version` failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(_) => {}
    }

    let auth = Command::new("gh")
        .args(["auth", "status"])
        .output()
        .context("failed to run `gh auth status`")?;
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
