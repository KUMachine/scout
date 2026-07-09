use owo_colors::OwoColorize;

use crate::checks::actions::Run;
use crate::checks::issues::Issue;
use crate::checks::prs::Pr;
use crate::checks::vulnerabilities::Alert;

fn pluralize(n: usize, singular: &str) -> String {
    if n == 1 {
        format!("{n} {singular}")
    } else {
        format!("{n} {singular}s")
    }
}

/// One row of the compact `check` table: new-item counts per repo.
pub struct RepoSummary {
    pub repo: String,
    pub prs: usize,
    pub issues: usize,
    pub runs: usize,
    pub alerts: usize,
}

#[derive(Clone, Copy)]
enum Kind {
    Pr,
    Issue,
    Run,
    Alert,
}

/// Render a right-aligned count cell. Zeros are dimmed dashes so the eye
/// jumps straight to repos that actually have news.
fn count_cell(n: usize, width: usize, kind: Kind) -> String {
    if n == 0 {
        return format!("{:>width$}", "-").dimmed().to_string();
    }
    let text = format!("{n:>width$}");
    match kind {
        Kind::Pr => text.cyan().bold().to_string(),
        Kind::Issue => text.yellow().bold().to_string(),
        Kind::Run | Kind::Alert => text.red().bold().to_string(),
    }
}

/// Print the compact overview table (default `check` output).
pub fn summary_table(rows: &[RepoSummary]) {
    const REPO_H: &str = "REPO";
    const PR_H: &str = "PRs";
    const ISSUE_H: &str = "ISSUES";
    const RUN_H: &str = "RUNS";
    const ALERT_H: &str = "ALERTS";

    let repo_w = rows
        .iter()
        .map(|r| r.repo.len())
        .chain(std::iter::once(REPO_H.len()))
        .max()
        .unwrap_or(REPO_H.len());

    let (pw, iw, rw, aw) = (PR_H.len(), ISSUE_H.len(), RUN_H.len(), ALERT_H.len());

    println!(
        "{}  {}  {}  {}  {}",
        format!("{REPO_H:<repo_w$}").bold(),
        format!("{PR_H:>pw$}").bold(),
        format!("{ISSUE_H:>iw$}").bold(),
        format!("{RUN_H:>rw$}").bold(),
        format!("{ALERT_H:>aw$}").bold(),
    );

    let rule_w = repo_w + pw + iw + rw + aw + 8; // 4 two-space gaps
    println!("{}", "-".repeat(rule_w).dimmed());

    let (mut tp, mut ti, mut tr, mut ta) = (0usize, 0usize, 0usize, 0usize);
    for row in rows {
        println!(
            "{}  {}  {}  {}  {}",
            format!("{:<repo_w$}", row.repo).bold(),
            count_cell(row.prs, pw, Kind::Pr),
            count_cell(row.issues, iw, Kind::Issue),
            count_cell(row.runs, rw, Kind::Run),
            count_cell(row.alerts, aw, Kind::Alert),
        );
        tp += row.prs;
        ti += row.issues;
        tr += row.runs;
        ta += row.alerts;
    }

    if rows.len() > 1 {
        println!("{}", "-".repeat(rule_w).dimmed());
        println!(
            "{}  {}  {}  {}  {}",
            format!("{:<repo_w$}", "TOTAL").dimmed(),
            count_cell(tp, pw, Kind::Pr),
            count_cell(ti, iw, Kind::Issue),
            count_cell(tr, rw, Kind::Run),
            count_cell(ta, aw, Kind::Alert),
        );
    }
}

/// Print the report for a single repo. Assumes the caller has already
/// checked that at least one category is non-empty.
pub fn repo_report(
    repo: &str,
    prs: &[&Pr],
    issues: &[&Issue],
    runs: &[&Run],
    alerts: &[&Alert],
) {
    let mut parts: Vec<String> = Vec::new();
    if !prs.is_empty() {
        parts.push(format!("{} new PR{}", prs.len(), if prs.len() == 1 { "" } else { "s" }));
    }
    if !issues.is_empty() {
        parts.push(pluralize(issues.len(), "new issue"));
    }
    if !runs.is_empty() {
        parts.push(pluralize(runs.len(), "failed run"));
    }
    if !alerts.is_empty() {
        parts.push(pluralize(alerts.len(), "vuln alert"));
    }

    println!("{} ({})", repo.bold(), parts.join(", "));

    for pr in prs {
        let draft = if pr.is_draft {
            format!(" {}", "(draft)".dimmed())
        } else {
            String::new()
        };
        println!(
            "  {} {}{}  {} {}  {}",
            format!("PR #{}", pr.number).cyan().bold(),
            pr.title,
            draft,
            "by".dimmed(),
            format!("@{}", pr.author.login).green(),
            pr.url.dimmed(),
        );
    }

    for issue in issues {
        println!(
            "  {} {}  {} {}  {}",
            format!("issue #{}", issue.number).yellow().bold(),
            issue.title,
            "by".dimmed(),
            format!("@{}", issue.author.login).yellow(),
            issue.url.dimmed(),
        );
    }

    for run in runs {
        let title = if run.display_title.is_empty() {
            run.name.as_str()
        } else {
            run.display_title.as_str()
        };
        println!(
            "  {} {} {}  {}",
            format!("run #{}", run.database_id).red().bold(),
            title.red(),
            format!("[{}]", run.name).dimmed(),
            run.url.dimmed(),
        );
    }

    for alert in alerts {
        let severity = alert.security_advisory.severity.to_uppercase();
        println!(
            "  {} {} in {}  {}  {}",
            format!("alert #{}", alert.number).red().bold(),
            format!("[{severity}]").red(),
            alert.dependency.package.name.bold(),
            alert.security_advisory.summary,
            alert.html_url.dimmed(),
        );
    }

    println!();
}
