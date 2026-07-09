use anyhow::Result;
use serde::Deserialize;

use crate::gh;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub database_id: u64,
    pub name: String,
    pub display_title: String,
    #[allow(dead_code)]
    pub status: String,
    pub conclusion: Option<String>,
    pub url: String,
    #[allow(dead_code)]
    pub created_at: String,
}

/// Fetch the most recent runs and keep only the ones that concluded in failure.
pub fn fetch(repo: &str) -> Result<Vec<Run>> {
    let runs: Vec<Run> = gh::run_json(&[
        "run",
        "list",
        "--repo",
        repo,
        "--limit",
        "20",
        "--json",
        "databaseId,name,displayTitle,status,conclusion,url,createdAt",
    ])?;
    Ok(runs
        .into_iter()
        .filter(|r| r.conclusion.as_deref() == Some("failure"))
        .collect())
}
