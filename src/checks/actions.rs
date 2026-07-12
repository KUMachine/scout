use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::Deserialize;

use crate::gh;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub database_id: u64,
    pub name: String,
    pub display_title: String,
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub head_branch: String,
    #[serde(default)]
    pub workflow_name: String,
    #[allow(dead_code)]
    pub status: String,
    pub conclusion: Option<String>,
    pub url: String,
    #[allow(dead_code)]
    pub created_at: String,
}

impl Run {
    /// Dependabot (and similar) update workflows — noisy, not actionable CI.
    pub fn is_bot(&self) -> bool {
        let workflow = self.workflow_name.to_ascii_lowercase();
        let branch = self.head_branch.to_ascii_lowercase();
        let name = self.name.to_ascii_lowercase();
        let title = self.display_title.to_ascii_lowercase();

        workflow.contains("dependabot")
            || workflow.contains("renovate")
            || branch.starts_with("dependabot/")
            || branch.starts_with("renovate/")
            || (self.event.eq_ignore_ascii_case("dynamic")
                && (name.contains("update #") || title.contains("update #")))
    }

    fn workflow_key(&self) -> &str {
        if self.workflow_name.is_empty() {
            self.name.as_str()
        } else {
            self.workflow_name.as_str()
        }
    }
}

/// Latest success/failure for a watched branch (non-bot workflows only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaneStatus {
    #[default]
    Unknown,
    Success,
    Failure,
}

impl LaneStatus {
    fn absorb(self, next: LaneStatus) -> LaneStatus {
        match (self, next) {
            (LaneStatus::Failure, _) | (_, LaneStatus::Failure) => LaneStatus::Failure,
            (LaneStatus::Success, _) | (_, LaneStatus::Success) => LaneStatus::Success,
            _ => LaneStatus::Unknown,
        }
    }
}

#[derive(Debug, Default)]
pub struct BranchReport {
    pub main: LaneStatus,
    pub dev: LaneStatus,
    /// Current failures (including bot), for detailed listing.
    pub failures: Vec<Run>,
}

const RUN_JSON_FIELDS: &str =
    "databaseId,name,displayTitle,event,headBranch,workflowName,status,conclusion,url,createdAt";

/// Branches we care about for Actions health.
pub const WATCHED_BRANCHES: &[&str] = &["dev", "main"];

fn is_watched_branch(branch: &str) -> bool {
    WATCHED_BRANCHES
        .iter()
        .any(|b| branch.eq_ignore_ascii_case(b))
}

fn normalize_branch(branch: &str) -> &str {
    if branch.eq_ignore_ascii_case("main") {
        "main"
    } else if branch.eq_ignore_ascii_case("dev") {
        "dev"
    } else {
        branch
    }
}

/// For each (branch, workflow), keep the newest success/failure. Then:
/// - lane glyphs use non-bot workflows only (Failure wins)
/// - `failures` lists every current failure (bot + human) for detail views
fn analyze(runs: Vec<Run>) -> BranchReport {
    // `gh run list` returns newest first — first insert wins.
    let mut latest: HashMap<(String, String), Run> = HashMap::new();
    let mut order: Vec<(String, String)> = Vec::new();

    for run in runs {
        if !is_watched_branch(&run.head_branch) {
            continue;
        }
        match run.conclusion.as_deref() {
            Some("success") | Some("failure") => {}
            _ => continue,
        }
        let key = (
            normalize_branch(&run.head_branch).to_string(),
            run.workflow_key().to_string(),
        );
        if let std::collections::hash_map::Entry::Vacant(e) = latest.entry(key.clone()) {
            e.insert(run);
            order.push(key);
        }
    }

    let mut report = BranchReport::default();
    let mut seen_fail = HashSet::new();

    for key in order {
        let Some(run) = latest.remove(&key) else {
            continue;
        };
        let failed = run.conclusion.as_deref() == Some("failure");
        if failed {
            let id = run.database_id;
            if seen_fail.insert(id) {
                report.failures.push(run.clone());
            }
        }

        if run.is_bot() {
            continue;
        }

        let status = if failed {
            LaneStatus::Failure
        } else {
            LaneStatus::Success
        };
        match key.0.as_str() {
            "main" => report.main = report.main.absorb(status),
            "dev" => report.dev = report.dev.absorb(status),
            _ => {}
        }
    }

    report
}

fn list_branch(repo: &str, branch: &str) -> Result<Vec<Run>> {
    gh::run_json(&[
        "run",
        "list",
        "--repo",
        repo,
        "--branch",
        branch,
        "--limit",
        "20",
        "--json",
        RUN_JSON_FIELDS,
    ])
}

fn list_watched_branches(repo: &str) -> Result<Vec<Run>> {
    std::thread::scope(|s| {
        let handles: Vec<_> = WATCHED_BRANCHES
            .iter()
            .map(|branch| s.spawn(|| list_branch(repo, branch)))
            .collect();
        let mut all = Vec::new();
        for handle in handles {
            all.extend(handle.join().expect("branch fetch panicked")?);
        }
        Ok(all)
    })
}

/// Inspect `main` / `dev` Actions health for a repo.
pub fn inspect(repo: &str) -> Result<BranchReport> {
    Ok(analyze(list_watched_branches(repo)?))
}
