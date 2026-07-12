use crate::checks::actions::{BranchReport, LaneStatus, Run};
use crate::checks::gitops::{GitopsStatus, ReleasePr};
use crate::checks::issues::Issue;
use crate::checks::prs::Pr;
use crate::checks::vulnerabilities::{Alert, AlertSummary};
use crate::theme::{
    bold, color_severity, dim, paint_danger, paint_issue, paint_meta, paint_ok, paint_pr, paint_repo,
    paint_sev_critical, paint_sev_high, paint_sev_low, paint_sev_moderate, paint_title, paint_wait,
};
use crate::util;

fn col_width<'a>(values: impl Iterator<Item = usize> + 'a, header_len: usize) -> usize {
    values.chain(std::iter::once(header_len)).max().unwrap_or(header_len)
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

/// Render a right-aligned count cell. Zeros are dimmed dashes so the eye
/// jumps straight to repos that actually have news.
fn count_cell(n: usize, width: usize) -> String {
    if n == 0 {
        return dim(&format!("{:>width$}", "-"));
    }
    paint_issue(&format!("{n:>width$}"))
}

fn alert_plain(summary: &AlertSummary) -> String {
    if summary.total() == 0 {
        return "-".into();
    }
    alert_severity_parts(summary)
        .into_iter()
        .map(|(n, suffix)| format!("{n}{suffix}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn alert_severity_parts(summary: &AlertSummary) -> Vec<(usize, char)> {
    let mut parts = Vec::new();
    if summary.critical > 0 {
        parts.push((summary.critical, 'C'));
    }
    if summary.high > 0 {
        parts.push((summary.high, 'H'));
    }
    if summary.moderate > 0 {
        parts.push((summary.moderate, 'M'));
    }
    if summary.low > 0 {
        parts.push((summary.low, 'L'));
    }
    parts
}

fn paint_alert_part(n: usize, suffix: char) -> String {
    let label = format!("{n}{suffix}");
    match suffix {
        'C' => paint_sev_critical(&label),
        'H' => paint_sev_high(&label),
        'M' => paint_sev_moderate(&label),
        _ => paint_sev_low(&label),
    }
}

/// Color-coded severity chips: `2H 1M` (C/H/M/L).
fn alert_cell(summary: &AlertSummary, width: usize) -> String {
    if summary.total() == 0 {
        return dim(&format!("{:>width$}", "-"));
    }
    let parts: Vec<String> = alert_severity_parts(summary)
        .into_iter()
        .map(|(n, suffix)| paint_alert_part(n, suffix))
        .collect();
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
        return dim(&format!("{:>width$}", "-"));
    }
    if bots == 0 {
        return paint(&format!("{human:>width$}"));
    }
    if human == 0 {
        return dim(&format!("{bots:>width$}"));
    }
    let plain = format!("{human}+{bots}");
    let pad = width.saturating_sub(plain.len());
    format!(
        "{}{}{}{}",
        " ".repeat(pad),
        paint(&human.to_string()),
        dim("+"),
        dim(&bots.to_string()),
    )
}

fn lane_mark(status: LaneStatus) -> String {
    match status {
        LaneStatus::Success => paint_ok("✓"),
        LaneStatus::Failure => paint_danger("✗"),
        LaneStatus::Unknown => dim("-"),
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

    let repo_w = col_width(rows.iter().map(|r| r.repo.len()), REPO_H.len());
    let pw = col_width(
        rows.iter().map(|r| split_cell_plain_len(r.prs, r.bot_prs)),
        PR_H.len(),
    );
    let rw = col_width(
        rows.iter().map(|r| lanes_plain_len(r.main, r.dev)),
        RUN_H.len(),
    );
    let aw = col_width(
        rows.iter().map(|r| alert_plain(&r.alerts).len()),
        ALERT_H.len(),
    );
    let iw = ISSUE_H.len();

    println!(
        "{}  {}  {}  {}  {}",
        bold(&format!("{REPO_H:<repo_w$}")),
        bold(&format!("{PR_H:>pw$}")),
        bold(&format!("{ISSUE_H:>iw$}")),
        bold(&format!("{RUN_H:<rw$}")),
        bold(&format!("{ALERT_H:>aw$}")),
    );

    let rule_w = repo_w + pw + iw + rw + aw + 8; // 4 two-space gaps
    println!("{}", dim(&"-".repeat(rule_w)));

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
            paint_repo(&format!("{:<repo_w$}", row.repo)),
            split_count_cell(row.prs, row.bot_prs, pw, paint_pr),
            count_cell(row.issues, iw),
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
        println!("{}", dim(&"-".repeat(rule_w)));
        println!(
            "{}  {}  {}  {}  {}",
            dim(&format!("{:<repo_w$}", "TOTAL")),
            split_count_cell(tp, tpb, pw, paint_pr),
            count_cell(ti, iw),
            dim(&format!("{:rw$}", "-")),
            alert_cell(&ta, aw),
        );
    }

    if ta.total() > 0 || rows.iter().any(|r| r.alerts.total() > 0) {
        println!(
            "{}",
            dim("ALERTS  C=critical  H=high  M=moderate  L=low  (Dependabot vulns)")
        );
    }
}

fn gitops_mark(open: bool) -> String {
    if open {
        paint_wait("●")
    } else {
        paint_ok("✓")
    }
}

fn format_gitops_lanes(status: &GitopsStatus) -> String {
    format!(
        "stg[{}] prod[{}]",
        gitops_mark(status.staging.is_some()),
        gitops_mark(status.production.is_some()),
    )
}

fn print_gitops_lane(label: &str, pr: &ReleasePr) {
    println!(
        "  {} {} {}  {}",
        paint_wait(label),
        paint_title(&pr.title),
        paint_pr(&format!("#{}", pr.number)),
        dim(&pr.url),
    );
    if pr.services.is_empty() {
        println!("    {}", dim("(no services listed in PR body)"));
        return;
    }
    for service in &pr.services {
        println!("    {} {}", dim("•"), paint_repo(service));
    }
}

/// Separate GitOps section: staging / production release status + waiting services.
pub fn gitops_section(rows: &[GitopsStatus]) {
    if rows.is_empty() {
        return;
    }

    println!();
    println!("{}", bold("GitOps"));

    const REPO_H: &str = "REPO";
    const LANE_H: &str = "RELEASES";

    let repo_w = col_width(rows.iter().map(|r| r.repo.len()), REPO_H.len());
    let lane_w = "stg[●] prod[●]".len().max(LANE_H.len());

    println!(
        "{}  {}",
        bold(&format!("{REPO_H:<repo_w$}")),
        bold(&format!("{LANE_H:<lane_w$}")),
    );
    println!("{}", dim(&"-".repeat(repo_w + lane_w + 2)));

    for status in rows {
        let lanes = format_gitops_lanes(status);
        println!(
            "{}  {}",
            paint_repo(&format!("{:<repo_w$}", status.repo)),
            lanes,
        );
        if let Some(pr) = &status.staging {
            print_gitops_lane("stg", pr);
        }
        if let Some(pr) = &status.production {
            print_gitops_lane("prod", pr);
        }
    }

    println!(
        "{}",
        dim("RELEASES  ● = open PR waiting to merge  ✓ = clear")
    );
}

/// Print main/dev lane status, then any current failed runs for a repo.
pub fn actions_report(repo: &str, report: &BranchReport) {
    println!("{}  {}", paint_repo(repo), format_lanes(report.main, report.dev));

    let (human, bots): (Vec<&Run>, Vec<&Run>) =
        report.failures.iter().partition(|run| !run.is_bot());

    print_bot_section(&human, &bots, print_run);
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
        parts.push(util::plural(bot_prs.len(), "bot PR"));
    }
    if !issues.is_empty() {
        parts.push(util::plural(issues.len(), "new issue"));
    }
    if !human_runs.is_empty() {
        parts.push(util::plural(human_runs.len(), "failed run"));
    }
    if !bot_runs.is_empty() {
        parts.push(util::plural(bot_runs.len(), "bot run"));
    }
    if !alerts.is_empty() {
        parts.push(util::plural(alerts.len(), "vuln alert"));
    }

    println!(
        "{} {}",
        paint_repo(repo),
        dim(&format!("({})", parts.join(", "))),
    );

    print_bot_section(&human_prs, &bot_prs, print_pr);

    for issue in issues {
        println!(
            "  {} {}  {} {}  {}",
            paint_issue(&format!("issue #{}", issue.number)),
            paint_title(&issue.title),
            dim("by"),
            paint_meta(&format!("@{}", issue.author.login)),
            dim(&issue.url),
        );
    }

    print_bot_section(&human_runs, &bot_runs, print_run);

    for alert in alerts {
        let sev = alert.security_advisory.severity.as_str();
        let severity = color_severity(sev);
        println!(
            "  {} {} in {}  {}  {}",
            paint_danger(&format!("alert #{}", alert.number)),
            severity,
            paint_repo(&alert.dependency.package.name),
            alert.security_advisory.summary,
            dim(&alert.html_url),
        );
    }

    println!();
}

fn print_bot_section<T>(human: &[&T], bots: &[&T], print: impl Fn(&T, bool)) {
    for item in human {
        print(item, false);
    }
    if bots.is_empty() {
        return;
    }
    if !human.is_empty() {
        println!("  {}", dim("bot"));
    }
    for item in bots {
        print(item, true);
    }
}

fn print_pr(pr: &Pr, quiet: bool) {
    let draft = if pr.is_draft {
        format!(" {}", dim("(draft)"))
    } else {
        String::new()
    };

    if quiet {
        println!(
            "  {} {}{}  {} {}  {}",
            dim(&format!("PR #{}", pr.number)),
            dim(&pr.title),
            draft,
            dim("by"),
            dim(&format!("@{}", pr.author.login)),
            dim(&pr.url),
        );
    } else {
        println!(
            "  {} {}{}  {} {}  {}",
            paint_pr(&format!("PR #{}", pr.number)),
            paint_title(&pr.title),
            draft,
            dim("by"),
            paint_meta(&format!("@{}", pr.author.login)),
            dim(&pr.url),
        );
    }
}

fn print_run(run: &Run, quiet: bool) {
    let lane = format!("[{} @ {}]", run.workflow(), run.head_branch);

    if quiet {
        println!(
            "  {} {} {}  {}",
            dim(&format!("run #{}", run.database_id)),
            dim(run.title()),
            dim(&lane),
            dim(&run.url),
        );
    } else {
        println!(
            "  {} {} {}  {}",
            paint_danger(&format!("run #{}", run.database_id)),
            paint_title(run.title()),
            dim(&lane),
            dim(&run.url),
        );
    }
}
