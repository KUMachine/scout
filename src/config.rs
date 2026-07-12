use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

/// Resolve the `scout` config directory (cross-platform via `dirs`),
/// creating it if it does not yet exist.
pub fn config_dir() -> Result<std::path::PathBuf> {
    let dir = dirs::config_dir()
        .context("could not determine the user config directory")?
        .join("scout");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create config directory {}", dir.display()))?;
    Ok(dir)
}

fn repos_path() -> Result<std::path::PathBuf> {
    Ok(config_dir()?.join("repos.json"))
}

fn app_config_path() -> Result<std::path::PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

/// Path to the persisted watch list (for display in `scout list`).
pub fn repos_file() -> Result<std::path::PathBuf> {
    repos_path()
}

/// Path to settings (`theme`, …) for display.
pub fn config_file() -> Result<std::path::PathBuf> {
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
    split_repo(slug).map(|_| slug.to_string())
}

/// Split a validated `owner/repo` slug into its parts.
pub fn split_repo(slug: &str) -> Result<(&str, &str)> {
    let (owner, name) = slug
        .split_once('/')
        .with_context(|| format!("expected repo in the form `owner/repo`, got `{slug}`"))?;
    if owner.is_empty() || name.is_empty() {
        bail!("expected repo in the form `owner/repo`, got `{slug}`");
    }
    Ok((owner, name))
}

fn read_json<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let data =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if data.trim().is_empty() {
        return Ok(T::default());
    }
    serde_json::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T, label: &str) -> Result<()> {
    let data = serde_json::to_string_pretty(value).context(format!("failed to serialize {label}"))?;
    std::fs::write(path, data).with_context(|| format!("failed to write {}", path.display()))
}

/// Load the watched-repo list. A missing file is treated as an empty list.
pub fn load() -> Result<Vec<String>> {
    read_json(&repos_path()?)
}

/// Persist the watched-repo list, pretty-printed.
pub fn save(repos: &[String]) -> Result<()> {
    write_json(&repos_path()?, repos, "repo list")
}

/// Theme from `config.json`, or `cool` if the file is missing.
/// Invalid names are a hard error.
pub fn load_theme() -> Result<Theme> {
    let cfg: AppConfig = read_json(&app_config_path()?)?;
    Theme::parse(&cfg.theme)
}

/// Persist the chosen theme (creates `config.json` if needed).
pub fn save_theme(theme: Theme) -> Result<()> {
    let mut cfg: AppConfig = read_json(&app_config_path()?)?;
    cfg.theme = theme.as_str().to_string();
    write_json(&app_config_path()?, &cfg, "config")
}
