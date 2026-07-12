//! Batched overview fetch for `scout check`: one GraphQL call for PR/issue/vuln
//! counts across many repos (instead of 3 `gh` invocations per repo).

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::checks::Author;
use crate::checks::vulnerabilities::AlertSummary;
use crate::config;
use crate::gh;

#[derive(Debug, Default)]
pub struct RepoCounts {
    pub human_prs: usize,
    pub bot_prs: usize,
    pub issues: usize,
    pub alerts: AlertSummary,
}

#[derive(Debug, Deserialize)]
struct GqlResponse {
    data: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlRepo {
    pull_requests: GqlPrConnection,
    issues: GqlCount,
    #[serde(default)]
    vulnerability_alerts: Option<GqlAlertConnection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlPrConnection {
    total_count: usize,
    nodes: Vec<GqlPrNode>,
}

#[derive(Debug, Deserialize)]
struct GqlPrNode {
    #[serde(default)]
    author: Author,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlCount {
    total_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlAlertConnection {
    #[serde(default)]
    total_count: usize,
    #[serde(default)]
    nodes: Vec<GqlAlertNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlAlertNode {
    #[serde(default)]
    security_advisory: GqlAdvisory,
}

#[derive(Debug, Default, Deserialize)]
struct GqlAdvisory {
    #[serde(default)]
    severity: String,
}

fn gql_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Fetch open PR / issue / vuln **counts** for many repos in one GraphQL
/// round-trip. PR authors are included so human vs bot can be split; vuln
/// nodes include severity for the ALERTS column.
/// Results are returned in the same order as `repos`.
pub fn fetch_counts(repos: &[String]) -> Result<Vec<RepoCounts>> {
    if repos.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = String::from("query {\n");
    for (i, repo) in repos.iter().enumerate() {
        let (owner, name) = config::split_repo(repo)?;
        let owner = gql_escape(owner);
        let name = gql_escape(name);
        // Cap PR/alert nodes at 100 — enough for summary classification.
        query.push_str(&format!(
            r#"  r{i}: repository(owner: "{owner}", name: "{name}") {{
    pullRequests(states: OPEN, first: 100) {{
      totalCount
      nodes {{ author {{ login __typename }} }}
    }}
    issues(states: OPEN) {{ totalCount }}
    vulnerabilityAlerts(states: OPEN, first: 100) {{
      totalCount
      nodes {{ securityAdvisory {{ severity }} }}
    }}
  }}
"#
        ));
    }
    query.push('}');

    let raw = gh::run_graphql(&query)?;
    let parsed: GqlResponse =
        serde_json::from_value(raw).context("failed to decode GraphQL overview response")?;
    let data = parsed
        .data
        .context("GraphQL overview response missing data")?;

    let mut out = Vec::with_capacity(repos.len());
    for (i, repo) in repos.iter().enumerate() {
        let key = format!("r{i}");
        let Some(value) = data.get(&key) else {
            out.push(RepoCounts::default());
            continue;
        };
        if value.is_null() {
            out.push(RepoCounts::default());
            continue;
        }
        let gql: GqlRepo = serde_json::from_value(value.clone())
            .with_context(|| format!("failed to decode GraphQL data for {repo}"))?;

        let bot_prs = gql
            .pull_requests
            .nodes
            .iter()
            .filter(|n| n.author.is_bot())
            .count();
        // Nodes may be capped at 100; treat any unseen remainder as human PRs.
        let seen = gql.pull_requests.nodes.len();
        let total = gql.pull_requests.total_count;
        let human_prs = (seen - bot_prs) + total.saturating_sub(seen);

        let mut alerts = AlertSummary::default();
        if let Some(conn) = gql.vulnerability_alerts {
            for node in &conn.nodes {
                alerts.bump(&node.security_advisory.severity);
            }
            // If GitHub truncated nodes, bucket the remainder as moderate so
            // the count still reflects that something is open.
            let unseen = conn.total_count.saturating_sub(conn.nodes.len());
            alerts.moderate += unseen;
        }

        out.push(RepoCounts {
            human_prs,
            bot_prs,
            issues: gql.issues.total_count,
            alerts,
        });
    }

    Ok(out)
}
