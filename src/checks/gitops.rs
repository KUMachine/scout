//! GitOps release PRs (`*-gitops` repos): staging / production waiting to merge.

use anyhow::Result;
use serde::Deserialize;

use crate::gh;

#[derive(Debug, Clone)]
pub struct ReleasePr {
    pub number: u64,
    pub title: String,
    pub url: String,
    /// Services listed in the PR body as waiting to ship.
    pub services: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct GitopsStatus {
    pub repo: String,
    pub staging: Option<ReleasePr>,
    pub production: Option<ReleasePr>,
}

impl GitopsStatus {
    pub fn is_gitops_repo(slug: &str) -> bool {
        slug.rsplit_once('/')
            .map(|(_, name)| name.ends_with("-gitops"))
            .unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPr {
    number: u64,
    title: String,
    url: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    head_ref_name: String,
    #[serde(default)]
    is_draft: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Staging,
    Production,
}

fn classify(pr: &RawPr) -> Option<Lane> {
    if pr.is_draft {
        return None;
    }
    let title = pr.title.trim();
    let head = pr.head_ref_name.as_str();
    if title.eq_ignore_ascii_case("Staging release") || head == "staging-release" {
        Some(Lane::Staging)
    } else if title.eq_ignore_ascii_case("Production release") || head == "prod-release" {
        Some(Lane::Production)
    } else {
        None
    }
}

fn push_unique(out: &mut Vec<String>, name: String) {
    if !name.is_empty() && !out.iter().any(|s| s == &name) {
        out.push(name);
    }
}

/// Pull service names from a release PR body.
///
/// Supports:
/// - `- service-name: Click [here](...)`
/// - `<summary>service-name v1.2.3</summary>`
pub fn parse_waiting_services(body: &str) -> Vec<String> {
    let mut out = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- ")
            && let Some((name, _)) = rest.split_once(':')
        {
            push_unique(&mut out, name.trim().to_string());
        }
    }

    let mut rest = body;
    while let Some(start) = rest.find("<summary>") {
        let after = &rest[start + "<summary>".len()..];
        let Some(end) = after.find("</summary>") else {
            break;
        };
        let inner = after[..end].trim();
        if let Some(name) = summary_service_name(inner) {
            push_unique(&mut out, name);
        }
        rest = &after[end + "</summary>".len()..];
    }

    out
}

fn summary_service_name(inner: &str) -> Option<String> {
    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    // "scout-client-test v4.0.0" → drop the version token
    if parts.len() >= 2 && is_version_token(parts[1]) {
        return Some(parts[0].to_string());
    }
    Some(inner.to_string())
}

fn is_version_token(s: &str) -> bool {
    let t = s.trim_start_matches('v');
    !t.is_empty() && t.chars().next().is_some_and(|c| c.is_ascii_digit())
}

/// Inspect open staging / production release PRs for a gitops repo.
pub fn inspect(repo: &str) -> Result<GitopsStatus> {
    let prs: Vec<RawPr> = gh::run_json(&[
        "pr",
        "list",
        "--repo",
        repo,
        "--state",
        "open",
        "--json",
        "number,title,url,body,headRefName,isDraft",
    ])?;

    let mut status = GitopsStatus {
        repo: repo.to_string(),
        staging: None,
        production: None,
    };

    for pr in prs {
        let Some(lane) = classify(&pr) else {
            continue;
        };
        let release = ReleasePr {
            number: pr.number,
            title: pr.title,
            url: pr.url,
            services: parse_waiting_services(&pr.body),
        };
        match lane {
            Lane::Staging if status.staging.is_none() => status.staging = Some(release),
            Lane::Production if status.production.is_none() => status.production = Some(release),
            _ => {}
        }
    }

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_staging_bullet_list() {
        let body = "# Releases\n\n- beas-dashboard-client: Click [here](https://x) to see the changes.\n- beas-processes-api: Click [here](https://y) to see the changes.\n";
        assert_eq!(
            parse_waiting_services(body),
            vec!["beas-dashboard-client", "beas-processes-api"]
        );
    }

    #[test]
    fn parses_production_summaries() {
        let body = r#"# Releases
<details>
<summary>scout-api-test v0.1.1</summary>
</details><details>
<summary>scout-client-test v4.0.0</summary>
</details>"#;
        assert_eq!(
            parse_waiting_services(body),
            vec!["scout-api-test", "scout-client-test"]
        );
    }
}
