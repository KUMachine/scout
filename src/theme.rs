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
    /// Anthropic Claude palette — coral, teal, amber on warm ink (see DESIGN.md).
    Claude,
    /// Discord palette — blurple, green, magenta on indigo (see discord/DESIGN.md).
    Discord,
    /// No ANSI color.
    Mono,
}

static ACTIVE: OnceLock<Theme> = OnceLock::new();

impl Theme {
    pub const ALL: &[Theme] = &[
        Theme::Cool,
        Theme::Classic,
        Theme::Claude,
        Theme::Discord,
        Theme::Mono,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Cool => "cool",
            Theme::Classic => "classic",
            Theme::Claude => "claude",
            Theme::Discord => "discord",
            Theme::Mono => "mono",
        }
    }

    pub fn parse(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "cool" => Ok(Theme::Cool),
            "classic" => Ok(Theme::Classic),
            "claude" => Ok(Theme::Claude),
            "discord" => Ok(Theme::Discord),
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
    claude: impl FnOnce(&str) -> String,
    discord: impl FnOnce(&str) -> String,
) -> String {
    match active() {
        Theme::Mono => plain(s),
        Theme::Cool => cool(s),
        Theme::Classic => classic(s),
        Theme::Claude => claude(s),
        Theme::Discord => discord(s),
    }
}

fn rgb_color(s: &str, rgb: (u8, u8, u8)) -> String {
    s.truecolor(rgb.0, rgb.1, rgb.2).to_string()
}

fn rgb_bold(s: &str, rgb: (u8, u8, u8)) -> String {
    s.truecolor(rgb.0, rgb.1, rgb.2).bold().to_string()
}

fn rgb_dimmed(s: &str, rgb: (u8, u8, u8)) -> String {
    s.truecolor(rgb.0, rgb.1, rgb.2).dimmed().to_string()
}

// Claude palette tokens from DESIGN.md (truecolor for terminal fidelity).
mod claude {
    pub const PRIMARY: (u8, u8, u8) = (0xcc, 0x78, 0x5c);
    pub const PRIMARY_ACTIVE: (u8, u8, u8) = (0xa9, 0x58, 0x3e);
    pub const ON_DARK: (u8, u8, u8) = (0xfa, 0xf9, 0xf5);
    pub const ON_DARK_SOFT: (u8, u8, u8) = (0xa0, 0x9d, 0x96);
    pub const ACCENT_TEAL: (u8, u8, u8) = (0x5d, 0xb8, 0xa6);
    pub const ACCENT_AMBER: (u8, u8, u8) = (0xe8, 0xa5, 0x5a);
    pub const SUCCESS: (u8, u8, u8) = (0x5d, 0xb8, 0x72);
    pub const WARNING: (u8, u8, u8) = (0xd4, 0xa0, 0x17);
    pub const ERROR: (u8, u8, u8) = (0xc6, 0x45, 0x45);
    pub const MUTED: (u8, u8, u8) = (0x6c, 0x6a, 0x64);
}

fn claude_color(s: &str, rgb: (u8, u8, u8)) -> String {
    rgb_color(s, rgb)
}

fn claude_bold(s: &str, rgb: (u8, u8, u8)) -> String {
    rgb_bold(s, rgb)
}

fn claude_primary_bold(s: &str) -> String {
    claude_bold(s, claude::PRIMARY)
}

fn claude_on_dark_bold(s: &str) -> String {
    claude_bold(s, claude::ON_DARK)
}

fn claude_teal_bold(s: &str) -> String {
    claude_bold(s, claude::ACCENT_TEAL)
}

fn claude_amber_bold(s: &str) -> String {
    claude_bold(s, claude::ACCENT_AMBER)
}

fn claude_success_bold(s: &str) -> String {
    claude_bold(s, claude::SUCCESS)
}

fn claude_error_bold(s: &str) -> String {
    claude_bold(s, claude::ERROR)
}

fn claude_warning_bold(s: &str) -> String {
    claude_bold(s, claude::WARNING)
}

fn claude_primary_active_bold(s: &str) -> String {
    claude_bold(s, claude::PRIMARY_ACTIVE)
}

fn claude_muted(s: &str) -> String {
    claude_color(s, claude::MUTED)
}

fn claude_on_dark_soft(s: &str) -> String {
    claude_color(s, claude::ON_DARK_SOFT)
}

// Discord palette tokens from discord/DESIGN.md (truecolor for terminal fidelity).
mod discord {
    pub const PRIMARY: (u8, u8, u8) = (0x58, 0x65, 0xf2);
    pub const GREEN: (u8, u8, u8) = (0x35, 0xed, 0x7e);
    pub const MAGENTA: (u8, u8, u8) = (0xec, 0x48, 0xbd);
    pub const LINK: (u8, u8, u8) = (0x00, 0xb0, 0xf4);
    pub const INK: (u8, u8, u8) = (0xff, 0xff, 0xff);
}

fn discord_primary_bold(s: &str) -> String {
    rgb_bold(s, discord::PRIMARY)
}

fn discord_ink_bold(s: &str) -> String {
    rgb_bold(s, discord::INK)
}

fn discord_link_bold(s: &str) -> String {
    rgb_bold(s, discord::LINK)
}

fn discord_magenta_bold(s: &str) -> String {
    rgb_bold(s, discord::MAGENTA)
}

fn discord_green_bold(s: &str) -> String {
    rgb_bold(s, discord::GREEN)
}

fn discord_link(s: &str) -> String {
    rgb_color(s, discord::LINK)
}

fn discord_muted(s: &str) -> String {
    rgb_dimmed(s, discord::INK)
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
    themed(
        s,
        bright_cyan_bold,
        bright_green_bold,
        claude_primary_bold,
        discord_primary_bold,
    )
}

pub fn paint_title(s: &str) -> String {
    themed(
        s,
        bright_white_bold,
        bright_white_bold,
        claude_on_dark_bold,
        discord_ink_bold,
    )
}

pub fn paint_pr(s: &str) -> String {
    themed(
        s,
        bright_blue_bold,
        bright_cyan_bold,
        claude_teal_bold,
        discord_link_bold,
    )
}

pub fn paint_issue(s: &str) -> String {
    themed(
        s,
        bright_magenta_bold,
        bright_yellow_bold,
        claude_amber_bold,
        discord_magenta_bold,
    )
}

pub fn paint_ok(s: &str) -> String {
    themed(
        s,
        bright_cyan_bold,
        bright_green_bold,
        claude_success_bold,
        discord_green_bold,
    )
}

pub fn paint_danger(s: &str) -> String {
    themed(
        s,
        bright_red_bold,
        bright_red_bold,
        claude_error_bold,
        discord_magenta_bold,
    )
}

pub fn paint_wait(s: &str) -> String {
    themed(
        s,
        bright_magenta_bold,
        bright_yellow_bold,
        claude_primary_active_bold,
        discord_primary_bold,
    )
}

pub fn paint_meta(s: &str) -> String {
    themed(s, blue, green, claude_on_dark_soft, discord_link)
}

pub fn paint_warning(s: &str) -> String {
    themed(
        s,
        bright_magenta_bold,
        bright_yellow_bold,
        claude_warning_bold,
        discord_magenta_bold,
    )
}

/// Color-code a Dependabot severity label for quick scanning.
pub fn color_severity(severity: &str) -> String {
    let label = format!("[{}]", severity.to_uppercase());
    match (active(), severity.to_ascii_lowercase().as_str()) {
        (Theme::Mono, _) => label,
        (Theme::Claude, "low") => claude_muted(&label),
        (Theme::Discord, "low") => discord_muted(&label),
        (_, "low") => bright_black(&label),
        (Theme::Classic, "moderate" | "medium") => yellow_bold(&label),
        (Theme::Classic, "high") => bright_yellow_bold(&label),
        (Theme::Claude, "moderate" | "medium") => claude_warning_bold(&label),
        (Theme::Claude, "high") => claude_primary_bold(&label),
        (Theme::Claude, "critical") => claude_error_bold(&label),
        (Theme::Discord, "moderate" | "medium") => discord_link_bold(&label),
        (Theme::Discord, "high") => discord_primary_bold(&label),
        (Theme::Discord, "critical") => discord_magenta_bold(&label),
        (_, "moderate" | "medium") => bright_blue_bold(&label),
        (_, "high") => magenta_bold(&label),
        (_, "critical") => bright_red_bold(&label),
        _ => label,
    }
}

pub fn paint_sev_critical(s: &str) -> String {
    themed(
        s,
        bright_red_bold,
        bright_red_bold,
        claude_error_bold,
        discord_magenta_bold,
    )
}

pub fn paint_sev_high(s: &str) -> String {
    themed(
        s,
        magenta_bold,
        bright_yellow_bold,
        claude_primary_bold,
        discord_primary_bold,
    )
}

pub fn paint_sev_moderate(s: &str) -> String {
    themed(
        s,
        bright_blue_bold,
        yellow_bold,
        claude_warning_bold,
        discord_link_bold,
    )
}

pub fn paint_sev_low(s: &str) -> String {
    themed(s, bright_black, bright_black, claude_muted, discord_muted)
}
