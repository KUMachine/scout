use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

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

fn app_config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

/// Path to the persisted watch list (for display in `scout list`).
pub fn repos_file() -> Result<PathBuf> {
    repos_path()
}

/// Path to settings (`theme`, …) for display.
pub fn config_file() -> Result<PathBuf> {
    app_config_path()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default = "default_theme_name")]
    theme: String,
}

fn default_theme_name() -> String {
    Theme::Cool.as_str().to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
        }
    }
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

fn load_app_config() -> Result<AppConfig> {
    let path = app_config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if data.trim().is_empty() {
        return Ok(AppConfig::default());
    }
    serde_json::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

fn save_app_config(cfg: &AppConfig) -> Result<()> {
    let path = app_config_path()?;
    let data = serde_json::to_string_pretty(cfg).context("failed to serialize config")?;
    fs::write(&path, &data).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

/// Theme from `config.json`, or `cool` if the file is missing.
/// Invalid names are a hard error.
pub fn load_theme() -> Result<Theme> {
    let cfg = load_app_config()?;
    Theme::parse(&cfg.theme)
}

/// Persist the chosen theme (creates `config.json` if needed).
pub fn save_theme(theme: Theme) -> Result<()> {
    let mut cfg = load_app_config()?;
    cfg.theme = theme.as_str().to_string();
    save_app_config(&cfg)
}
