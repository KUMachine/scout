use owo_colors::OwoColorize;

use crate::checks::actions::{BranchReport, LaneStatus, Run};
use crate::checks::issues::Issue;
use crate::checks::prs::Pr;
use crate::checks::vulnerabilities::{Alert, AlertSummary};

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
    /// Human-authored open PRs (the ones worth attention).
    pub prs: usize,
    /// Bot / Dependabot PRs — shown quieter in the PRs column.
    pub bot_prs: usize,
    pub issues: usize,
    pub main: LaneStatus,
    pub dev: LaneStatus,
    pub alerts: AlertSummary,
}

#[derive(Clone, Copy)]
enum Kind {
    Issue,
}

/// Render a right-aligned count cell. Zeros are dimmed dashes so the eye
/// jumps straight to repos that actually have news.
fn count_cell(n: usize, width: usize, kind: Kind) -> String {
    if n == 0 {
        return format!("{:>width$}", "-").dimmed().to_string();
    }
    let text = format!("{n:>width$}");
    match kind {
        Kind::Issue => text.yellow().bold().to_string(),
    }
}

fn alert_plain(summary: &AlertSummary) -> String {
    if summary.total() == 0 {
        return "-".into();
    }
    let mut parts = Vec::new();
    if summary.critical > 0 {
        parts.push(format!("{}C", summary.critical));
    }
    if summary.high > 0 {
        parts.push(format!("{}H", summary.high));
    }
    if summary.moderate > 0 {
        parts.push(format!("{}M", summary.moderate));
    }
    if summary.low > 0 {
        parts.push(format!("{}L", summary.low));
    }
    parts.join(" ")
}

/// Color-coded severity chips: `2H 1M` (C/H/M/L).
fn alert_cell(summary: &AlertSummary, width: usize) -> String {
    if summary.total() == 0 {
        return format!("{:>width$}", "-").dimmed().to_string();
    }
    let mut parts = Vec::new();
    if summary.critical > 0 {
        parts.push(format!("{}C", summary.critical).red().bold().to_string());
    }
    if summary.high > 0 {
        parts.push(
            format!("{}H", summary.high)
                .bright_red()
                .bold()
                .to_string(),
        );
    }
    if summary.moderate > 0 {
        parts.push(
            format!("{}M", summary.moderate)
                .yellow()
                .bold()
                .to_string(),
        );
    }
    if summary.low > 0 {
        parts.push(format!("{}L", summary.low).bright_black().to_string());
    }
    let plain = alert_plain(summary);
    let pad = width.saturating_sub(plain.len());
    format!("{}{}", " ".repeat(pad), parts.join(" "))
}

fn split_cell_plain_len(human: usize, bots: usize) -> usize {
    if human == 0 && bots == 0 {
        1 // "-"
    } else if bots == 0 {
        human.to_string().len()
    } else if human == 0 {
        bots.to_string().len()
    } else {
        format!("{human}+{bots}").len()
    }
}

/// Human count prominent; bot count appended dimmed as `+N`.
fn split_count_cell(
    human: usize,
    bots: usize,
    width: usize,
    paint: impl Fn(&str) -> String,
) -> String {
    if human == 0 && bots == 0 {
        return format!("{:>width$}", "-").dimmed().to_string();
    }
    if bots == 0 {
        return paint(&format!("{human:>width$}"));
    }
    if human == 0 {
        return format!("{bots:>width$}").dimmed().to_string();
    }
    let plain = format!("{human}+{bots}");
    let pad = width.saturating_sub(plain.len());
    format!(
        "{}{}{}{}",
        " ".repeat(pad),
        paint(&human.to_string()),
        "+".dimmed(),
        bots.to_string().dimmed(),
    )
}

fn lane_mark(status: LaneStatus) -> String {
    match status {
        LaneStatus::Success => "✓".green().bold().to_string(),
        LaneStatus::Failure => "✗".red().bold().to_string(),
        LaneStatus::Unknown => "-".dimmed().to_string(),
    }
}

/// `dev[✓] main[✗]` — always shows both watched branches.
pub fn format_lanes(main: LaneStatus, dev: LaneStatus) -> String {
    format!("dev[{}] main[{}]", lane_mark(dev), lane_mark(main))
}

fn lanes_plain_len(main: LaneStatus, dev: LaneStatus) -> usize {
    // ASCII width ignoring ANSI; marks are single cells.
    let mark = |s: LaneStatus| match s {
        LaneStatus::Unknown => 1, // "-"
        _ => 1,                   // ✓ / ✗
    };
    // "dev[" + mark + "] main[" + mark + "]"
    4 + mark(dev) + 2 + 5 + mark(main) + 1
}

/// Print the compact overview table (`scout check` output).
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

    let pw = rows
        .iter()
        .map(|r| split_cell_plain_len(r.prs, r.bot_prs))
        .chain(std::iter::once(PR_H.len()))
        .max()
        .unwrap_or(PR_H.len());
    let rw = rows
        .iter()
        .map(|r| lanes_plain_len(r.main, r.dev))
        .chain(std::iter::once(RUN_H.len()))
        .max()
        .unwrap_or(RUN_H.len());
    let aw = rows
        .iter()
        .map(|r| alert_plain(&r.alerts).len())
        .chain(std::iter::once(ALERT_H.len()))
        .max()
        .unwrap_or(ALERT_H.len());
    let iw = ISSUE_H.len();

    println!(
        "{}  {}  {}  {}  {}",
        format!("{REPO_H:<repo_w$}").bold(),
        format!("{PR_H:>pw$}").bold(),
        format!("{ISSUE_H:>iw$}").bold(),
        format!("{RUN_H:<rw$}").bold(),
        format!("{ALERT_H:>aw$}").bold(),
    );

    let rule_w = repo_w + pw + iw + rw + aw + 8; // 4 two-space gaps
    println!("{}", "-".repeat(rule_w).dimmed());

    let (mut tp, mut tpb, mut ti, mut ta) = (
        0usize,
        0usize,
        0usize,
        AlertSummary::default(),
    );
    for row in rows {
        let lanes = format_lanes(row.main, row.dev);
        let pad = rw.saturating_sub(lanes_plain_len(row.main, row.dev));
        println!(
            "{}  {}  {}  {}{}  {}",
            format!("{:<repo_w$}", row.repo).bright_green().bold(),
            split_count_cell(row.prs, row.bot_prs, pw, |s| s.cyan().bold().to_string()),
            count_cell(row.issues, iw, Kind::Issue),
            lanes,
            " ".repeat(pad),
            alert_cell(&row.alerts, aw),
        );
        tp += row.prs;
        tpb += row.bot_prs;
        ti += row.issues;
        ta = ta.merge(row.alerts);
    }

    if rows.len() > 1 {
        println!("{}", "-".repeat(rule_w).dimmed());
        println!(
            "{}  {}  {}  {}  {}",
            format!("{:<repo_w$}", "TOTAL").dimmed(),
            split_count_cell(tp, tpb, pw, |s| s.cyan().bold().to_string()),
            count_cell(ti, iw, Kind::Issue),
            format!("{:rw$}", "-").dimmed(),
            alert_cell(&ta, aw),
        );
    }

    if ta.total() > 0 || rows.iter().any(|r| r.alerts.total() > 0) {
        println!(
            "{}",
            "ALERTS  C=critical  H=high  M=moderate  L=low  (Dependabot vulns)"
                .dimmed()
        );
    }
}

/// Print main/dev lane status, then any current failed runs for a repo.
pub fn actions_report(repo: &str, report: &BranchReport) {
    println!(
        "{}  {}",
        repo.bright_green().bold(),
        format_lanes(report.main, report.dev),
    );

    let (human, bots): (Vec<&Run>, Vec<&Run>) = report
        .failures
        .iter()
        .partition(|run| !run.is_bot());

    for run in &human {
        print_run(run, false);
    }
    if !bots.is_empty() {
        if !human.is_empty() {
            println!("  {}", "bot".dimmed());
        }
        for run in &bots {
            print_run(run, true);
        }
    }
    println!();
}

/// Print the per-item report for a single repo. Assumes the caller has
/// already checked that at least one category is non-empty. Pass empty
/// slices for categories you don't want to show.
pub fn repo_report(
    repo: &str,
    prs: &[&Pr],
    issues: &[&Issue],
    runs: &[&Run],
    alerts: &[&Alert],
) {
    let (human_prs, bot_prs): (Vec<&Pr>, Vec<&Pr>) =
        prs.iter().copied().partition(|pr| !pr.author.is_bot());
    let (human_runs, bot_runs): (Vec<&Run>, Vec<&Run>) =
        runs.iter().copied().partition(|run| !run.is_bot());

    let mut parts: Vec<String> = Vec::new();
    if !human_prs.is_empty() {
        parts.push(format!(
            "{} new PR{}",
            human_prs.len(),
            if human_prs.len() == 1 { "" } else { "s" }
        ));
    }
    if !bot_prs.is_empty() {
        parts.push(pluralize(bot_prs.len(), "bot PR"));
    }
    if !issues.is_empty() {
        parts.push(pluralize(issues.len(), "new issue"));
    }
    if !human_runs.is_empty() {
        parts.push(pluralize(human_runs.len(), "failed run"));
    }
    if !bot_runs.is_empty() {
        parts.push(pluralize(bot_runs.len(), "bot run"));
    }
    if !alerts.is_empty() {
        parts.push(pluralize(alerts.len(), "vuln alert"));
    }

    println!(
        "{} {}",
        repo.bright_green().bold(),
        format!("({})", parts.join(", ")).dimmed(),
    );

    for pr in &human_prs {
        print_pr(pr, false);
    }
    if !bot_prs.is_empty() {
        if !human_prs.is_empty() {
            println!("  {}", "bot".dimmed());
        }
        for pr in &bot_prs {
            print_pr(pr, true);
        }
    }

    for issue in issues {
        println!(
            "  {} {}  {} {}  {}",
            format!("issue #{}", issue.number).yellow().bold(),
            issue.title.bright_cyan().bold(),
            "by".dimmed(),
            format!("@{}", issue.author.login).yellow(),
            issue.url.dimmed(),
        );
    }

    for run in &human_runs {
        print_run(run, false);
    }
    if !bot_runs.is_empty() {
        if !human_runs.is_empty() {
            println!("  {}", "bot".dimmed());
        }
        for run in &bot_runs {
            print_run(run, true);
        }
    }

    for alert in alerts {
        let sev = alert.security_advisory.severity.as_str();
        let severity = color_severity(sev);
        println!(
            "  {} {} in {}  {}  {}",
            format!("alert #{}", alert.number).red().bold(),
            severity,
            alert.dependency.package.name.bright_cyan().bold(),
            alert.security_advisory.summary,
            alert.html_url.dimmed(),
        );
    }

    println!();
}

fn print_pr(pr: &Pr, dim: bool) {
    let draft = if pr.is_draft {
        format!(" {}", "(draft)".dimmed())
    } else {
        String::new()
    };

    if dim {
        println!(
            "  {} {}{}  {} {}  {}",
            format!("PR #{}", pr.number).dimmed(),
            pr.title.dimmed(),
            draft,
            "by".dimmed(),
            format!("@{}", pr.author.login).dimmed(),
            pr.url.dimmed(),
        );
    } else {
        println!(
            "  {} {}{}  {} {}  {}",
            format!("PR #{}", pr.number).cyan().bold(),
            pr.title.bright_cyan().bold(),
            draft,
            "by".dimmed(),
            format!("@{}", pr.author.login).green(),
            pr.url.dimmed(),
        );
    }
}

fn print_run(run: &Run, dim: bool) {
    let title = if run.display_title.is_empty() {
        run.name.as_str()
    } else {
        run.display_title.as_str()
    };
    let workflow = if run.workflow_name.is_empty() {
        run.name.as_str()
    } else {
        run.workflow_name.as_str()
    };
    let lane = format!("[{workflow} @ {}]", run.head_branch);

    if dim {
        println!(
            "  {} {} {}  {}",
            format!("run #{}", run.database_id).dimmed(),
            title.dimmed(),
            lane.dimmed(),
            run.url.dimmed(),
        );
    } else {
        println!(
            "  {} {} {}  {}",
            format!("run #{}", run.database_id).bright_red().bold(),
            title.bright_cyan().bold(),
            lane.dimmed(),
            run.url.dimmed(),
        );
    }
}

/// Color-code a Dependabot severity label for quick scanning.
fn color_severity(severity: &str) -> String {
    let label = format!("[{}]", severity.to_uppercase());
    match severity.to_ascii_lowercase().as_str() {
        "low" => label.bright_black().to_string(),
        "moderate" | "medium" => label.yellow().bold().to_string(),
        "high" => label.bright_red().bold().to_string(),
        "critical" => label.red().bold().to_string(),
        _ => label.to_string(),
    }
}
