use anyhow::Result;
use serde::Deserialize;

use crate::checks::Author;
use crate::gh;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pr {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub author: Author,
    pub url: String,
    #[allow(dead_code)]
    pub created_at: String,
    pub is_draft: bool,
}

pub fn fetch(repo: &str) -> Result<Vec<Pr>> {
    gh::list_items(
        "pr",
        repo,
        "number,title,author,url,createdAt,isDraft",
    )
}
