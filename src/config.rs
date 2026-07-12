use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// Resolve the `scout` config directory (cross-platform via `dirs`),
/// creating it if it does not yet exist.
pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not determine the user config directory")?
        .join("scout");
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create config directory {}", dir.display()))?;
    Ok(dir)
}

fn repos_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("repos.json"))
}

/// Validate an `owner/repo` slug. Returns the normalized slug on success.
pub fn validate_slug(slug: &str) -> Result<String> {
    let slug = slug.trim();
    let parts: Vec<&str> = slug.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("expected repo in the form `owner/repo`, got `{slug}`");
    }
    Ok(slug.to_string())
}

/// Load the watched-repo list. A missing file is treated as an empty list.
pub fn load() -> Result<Vec<String>> {
    let path = repos_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }
    let repos: Vec<String> = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(repos)
}

/// Persist the watched-repo list, pretty-printed.
pub fn save(repos: &[String]) -> Result<()> {
    let path = repos_path()?;
    let data = serde_json::to_string_pretty(repos).context("failed to serialize repo list")?;
    fs::write(&path, data).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
