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

/// Dim / de-emphasized text (no-op under mono).
pub fn dim(s: &str) -> String {
    if active().colored() {
        s.dimmed().to_string()
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    if active().colored() {
        s.bold().to_string()
    } else {
        s.to_string()
    }
}

pub fn paint_repo(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_green().bold().to_string(),
        Theme::Cool => s.bright_cyan().bold().to_string(),
    }
}

pub fn paint_title(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        _ => s.bright_white().bold().to_string(),
    }
}

pub fn paint_pr(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_cyan().bold().to_string(),
        Theme::Cool => s.bright_blue().bold().to_string(),
    }
}

pub fn paint_issue(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_yellow().bold().to_string(),
        Theme::Cool => s.bright_magenta().bold().to_string(),
    }
}

pub fn paint_ok(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_green().bold().to_string(),
        Theme::Cool => s.bright_cyan().bold().to_string(),
    }
}

pub fn paint_danger(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        _ => s.bright_red().bold().to_string(),
    }
}

pub fn paint_wait(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_yellow().bold().to_string(),
        Theme::Cool => s.bright_magenta().bold().to_string(),
    }
}

pub fn paint_meta(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.green().to_string(),
        Theme::Cool => s.blue().to_string(),
    }
}

pub fn paint_warning(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_yellow().bold().to_string(),
        Theme::Cool => s.bright_magenta().bold().to_string(),
    }
}

/// Color-code a Dependabot severity label for quick scanning.
pub fn color_severity(severity: &str) -> String {
    let label = format!("[{}]", severity.to_uppercase());
    match (active(), severity.to_ascii_lowercase().as_str()) {
        (Theme::Mono, _) => label,
        (_, "low") => label.bright_black().to_string(),
        (Theme::Classic, "moderate" | "medium") => label.yellow().bold().to_string(),
        (Theme::Classic, "high") => label.bright_yellow().bold().to_string(),
        (_, "moderate" | "medium") => label.bright_blue().bold().to_string(),
        (_, "high") => label.magenta().bold().to_string(),
        (_, "critical") => label.bright_red().bold().to_string(),
        _ => label,
    }
}

pub fn paint_sev_critical(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        _ => s.bright_red().bold().to_string(),
    }
}

pub fn paint_sev_high(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.bright_yellow().bold().to_string(),
        Theme::Cool => s.magenta().bold().to_string(),
    }
}

pub fn paint_sev_moderate(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        Theme::Classic => s.yellow().bold().to_string(),
        Theme::Cool => s.bright_blue().bold().to_string(),
    }
}

pub fn paint_sev_low(s: &str) -> String {
    match active() {
        Theme::Mono => s.to_string(),
        _ => s.bright_black().to_string(),
    }
}
