use std::sync::OnceLock;

use anyhow::{Result, bail};
use owo_colors::OwoColorize;

/// Built-in color themes for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Cyan / blue / magenta (current default).
    Cool,
    /// Traffic-light green / yellow / red.
    Classic,
    /// No ANSI color.
    Mono,
}

static ACTIVE: OnceLock<Theme> = OnceLock::new();

impl Theme {
    pub const ALL: &[Theme] = &[Theme::Cool, Theme::Classic, Theme::Mono];

    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Cool => "cool",
            Theme::Classic => "classic",
            Theme::Mono => "mono",
        }
    }

    pub fn parse(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "cool" => Ok(Theme::Cool),
            "classic" => Ok(Theme::Classic),
            "mono" => Ok(Theme::Mono),
            other => bail!(
                "unknown theme `{other}`; expected one of: {}",
                Self::ALL
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    fn colored(self) -> bool {
        !matches!(self, Theme::Mono)
    }
}

/// Install the active theme for this process. Safe to call once at startup.
pub fn apply(theme: Theme) {
    let _ = ACTIVE.set(theme);
}

pub fn active() -> Theme {
    *ACTIVE.get().unwrap_or(&Theme::Cool)
}

/// Effective theme for this run: `NO_COLOR` forces mono without touching config.
pub fn effective_from_config(configured: Theme) -> Theme {
    if std::env::var_os("NO_COLOR").is_some() {
        Theme::Mono
    } else {
        configured
    }
}

fn plain(s: &str) -> String {
    s.to_string()
}

fn when_colored(s: &str, paint: impl FnOnce(&str) -> String) -> String {
    if active().colored() {
        paint(s)
    } else {
        plain(s)
    }
}

fn themed(
    s: &str,
    cool: impl FnOnce(&str) -> String,
    classic: impl FnOnce(&str) -> String,
) -> String {
    match active() {
        Theme::Mono => plain(s),
        Theme::Cool => cool(s),
        Theme::Classic => classic(s),
    }
}

fn bright_cyan_bold(s: &str) -> String {
    s.bright_cyan().bold().to_string()
}
fn bright_blue_bold(s: &str) -> String {
    s.bright_blue().bold().to_string()
}
fn bright_green_bold(s: &str) -> String {
    s.bright_green().bold().to_string()
}
fn bright_yellow_bold(s: &str) -> String {
    s.bright_yellow().bold().to_string()
}
fn bright_magenta_bold(s: &str) -> String {
    s.bright_magenta().bold().to_string()
}
fn bright_red_bold(s: &str) -> String {
    s.bright_red().bold().to_string()
}
fn bright_white_bold(s: &str) -> String {
    s.bright_white().bold().to_string()
}
fn yellow_bold(s: &str) -> String {
    s.yellow().bold().to_string()
}
fn green(s: &str) -> String {
    s.green().to_string()
}
fn blue(s: &str) -> String {
    s.blue().to_string()
}
fn magenta_bold(s: &str) -> String {
    s.magenta().bold().to_string()
}
fn bright_black(s: &str) -> String {
    s.bright_black().to_string()
}

pub fn dim(s: &str) -> String {
    when_colored(s, |t| t.dimmed().to_string())
}

pub fn bold(s: &str) -> String {
    when_colored(s, |t| t.bold().to_string())
}

pub fn paint_repo(s: &str) -> String {
    themed(s, bright_cyan_bold, bright_green_bold)
}

pub fn paint_title(s: &str) -> String {
    themed(s, bright_white_bold, bright_white_bold)
}

pub fn paint_pr(s: &str) -> String {
    themed(s, bright_blue_bold, bright_cyan_bold)
}

pub fn paint_issue(s: &str) -> String {
    themed(s, bright_magenta_bold, bright_yellow_bold)
}

pub fn paint_ok(s: &str) -> String {
    themed(s, bright_cyan_bold, bright_green_bold)
}

pub fn paint_danger(s: &str) -> String {
    themed(s, bright_red_bold, bright_red_bold)
}

pub fn paint_wait(s: &str) -> String {
    themed(s, bright_magenta_bold, bright_yellow_bold)
}

pub fn paint_meta(s: &str) -> String {
    themed(s, blue, green)
}

pub fn paint_warning(s: &str) -> String {
    themed(s, bright_magenta_bold, bright_yellow_bold)
}

/// Color-code a Dependabot severity label for quick scanning.
pub fn color_severity(severity: &str) -> String {
    let label = format!("[{}]", severity.to_uppercase());
    match (active(), severity.to_ascii_lowercase().as_str()) {
        (Theme::Mono, _) => label,
        (_, "low") => bright_black(&label),
        (Theme::Classic, "moderate" | "medium") => yellow_bold(&label),
        (Theme::Classic, "high") => bright_yellow_bold(&label),
        (_, "moderate" | "medium") => bright_blue_bold(&label),
        (_, "high") => magenta_bold(&label),
        (_, "critical") => bright_red_bold(&label),
        _ => label,
    }
}

pub fn paint_sev_critical(s: &str) -> String {
    themed(s, bright_red_bold, bright_red_bold)
}

pub fn paint_sev_high(s: &str) -> String {
    themed(s, magenta_bold, bright_yellow_bold)
}

pub fn paint_sev_moderate(s: &str) -> String {
    themed(s, bright_blue_bold, yellow_bold)
}

pub fn paint_sev_low(s: &str) -> String {
    themed(s, bright_black, bright_black)
}
