pub mod actions;
pub mod issues;
pub mod prs;
pub mod vulnerabilities;

use serde::Deserialize;

/// Shared author shape returned by `gh pr/issue list --json author`.
#[derive(Debug, Default, Deserialize)]
pub struct Author {
    #[serde(default)]
    pub login: String,
}
