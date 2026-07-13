use std::ffi::OsStr;

use anyhow::{Context, Result, bail};
use clap::Command;
use clap_complete::{CompleteEnv, CompletionCandidate};

use crate::config;
use crate::theme::Theme;

/// Tab-complete theme names from [`Theme::ALL`].
pub fn themes() -> Vec<CompletionCandidate> {
    Theme::ALL
        .iter()
        .map(|theme| CompletionCandidate::new(theme.as_str()))
        .collect()
}

/// Tab-complete `owner/repo` slugs from the watch list, plus `.` for the current repo.
pub fn repos(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    let mut candidates = Vec::new();

    if ".".starts_with(prefix.as_ref()) {
        candidates.push(CompletionCandidate::new("."));
    }

    let Ok(repos) = config::load() else {
        return candidates;
    };

    candidates.extend(
        repos
            .into_iter()
            .filter(|repo| repo.starts_with(prefix.as_ref()))
            .map(CompletionCandidate::new),
    );
    candidates
}

/// Print a shell registration script to stdout (for `eval` / `source`).
pub fn emit_registration<F>(shell: &str, factory: F) -> Result<()>
where
    F: Fn() -> Command,
{
    // SAFETY: only called before any other threads start, during `scout complete`.
    unsafe {
        std::env::set_var("COMPLETE", shell);
    }

    let exe = std::env::current_exe().context("failed to resolve scout executable")?;
    let completed = CompleteEnv::with_factory(factory)
        .completer(exe.to_string_lossy().into_owned())
        .try_complete(
            std::iter::once(exe.as_os_str().to_os_string()),
            std::env::current_dir().ok().as_deref(),
        )?;

    if completed {
        std::process::exit(0);
    }

    bail!("failed to generate completion script for `{shell}`");
}
