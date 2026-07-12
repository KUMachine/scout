use anyhow::Result;
use serde::Deserialize;

use crate::gh;

/// Dependabot alerts come from `gh api` and use snake_case keys.
#[derive(Debug, Default, Deserialize)]
pub struct SecurityAdvisory {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub severity: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Dependency {
    #[serde(default)]
    pub package: Package,
}

#[derive(Debug, Deserialize)]
pub struct Alert {
    pub number: u64,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub security_advisory: SecurityAdvisory,
    #[serde(default)]
    pub dependency: Dependency,
    #[allow(dead_code)]
    #[serde(default)]
    pub created_at: String,
}

/// Severity breakdown for the compact `check` table.
#[derive(Debug, Default, Clone, Copy)]
pub struct AlertSummary {
    pub critical: usize,
    pub high: usize,
    pub moderate: usize,
    pub low: usize,
}

impl AlertSummary {
    pub fn total(self) -> usize {
        self.critical + self.high + self.moderate + self.low
    }

    pub fn bump(&mut self, severity: &str) {
        match severity.to_ascii_lowercase().as_str() {
            "critical" => self.critical += 1,
            "high" => self.high += 1,
            "moderate" | "medium" => self.moderate += 1,
            "low" => self.low += 1,
            // Unknown severities land in moderate so they still surface.
            _ => self.moderate += 1,
        }
    }

    pub fn from_alerts(alerts: &[Alert]) -> Self {
        let mut summary = Self::default();
        for alert in alerts {
            summary.bump(&alert.security_advisory.severity);
        }
        summary
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            critical: self.critical + other.critical,
            high: self.high + other.high,
            moderate: self.moderate + other.moderate,
            low: self.low + other.low,
        }
    }
}

impl Alert {
    /// Lower = more severe. Used for sorting lists critical-first.
    fn severity_rank(&self) -> u8 {
        match self.security_advisory.severity.to_ascii_lowercase().as_str() {
            "critical" => 0,
            "high" => 1,
            "moderate" | "medium" => 2,
            "low" => 3,
            _ => 2,
        }
    }
}

/// Fetch open Dependabot alerts. Callers should treat an `Err` here
/// (e.g. a 403 on repos where you're not an admin) as a soft warning.
/// Results are sorted critical → high → moderate → low.
pub fn fetch(repo: &str) -> Result<Vec<Alert>> {
    let path = format!("repos/{repo}/dependabot/alerts?state=open");
    let mut alerts: Vec<Alert> = gh::run_json(&["api", path.as_str()])?;
    alerts.sort_by(|a, b| {
        a.severity_rank()
            .cmp(&b.severity_rank())
            .then_with(|| a.number.cmp(&b.number))
    });
    Ok(alerts)
}
