use anyhow::Result;
use serde::Deserialize;

use crate::gh;

/// Dependabot alerts come from `gh api` and use snake_case keys.
#[derive(Debug, Default, Deserialize)]
pub struct SecurityAdvisory {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub severity: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Dependency {
    #[serde(default)]
    pub package: Package,
}

#[derive(Debug, Deserialize)]
pub struct Alert {
    pub number: u64,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub security_advisory: SecurityAdvisory,
    #[serde(default)]
    pub dependency: Dependency,
    #[allow(dead_code)]
    #[serde(default)]
    pub created_at: String,
}

/// Fetch open Dependabot alerts. Callers should treat an `Err` here
/// (e.g. a 403 on repos where you're not an admin) as a soft warning.
pub fn fetch(repo: &str) -> Result<Vec<Alert>> {
    let path = format!("repos/{repo}/dependabot/alerts?state=open");
    gh::run_json(&["api", path.as_str()])
}
