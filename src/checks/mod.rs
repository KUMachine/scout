pub mod actions;
pub mod gitops;
pub mod issues;
pub mod prs;
pub mod snapshot;
pub mod vulnerabilities;

use serde::Deserialize;

/// Shared author shape returned by `gh pr/issue list --json author`
/// and by GraphQL (`login` + `__typename`).
#[derive(Debug, Default, Deserialize)]
pub struct Author {
    #[serde(default)]
    pub login: String,
    /// Present on `gh pr list --json author` for GitHub App actors.
    #[serde(default)]
    pub is_bot: bool,
    /// GraphQL actor type (`User`, `Bot`, …).
    #[serde(default, rename = "__typename")]
    pub typename: String,
}

impl Author {
    /// Dependabot and other GitHub Apps / bots.
    pub fn is_bot(&self) -> bool {
        if self.is_bot || self.typename.eq_ignore_ascii_case("Bot") {
            return true;
        }
        let login = self.login.to_ascii_lowercase();
        login.ends_with("[bot]")
            || login.starts_with("app/")
            || login == "dependabot"
            || login.starts_with("dependabot/")
    }
}
