use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Release {
    pub version: String,
    pub commit: String,
    pub environment: String,
    pub built_at: u64,
    pub release: String,
}

impl Release {
    pub fn current() -> Self {
        let version =
            setting("MERCURY_VERSION").unwrap_or_else(|| env!("CARGO_PKG_VERSION").into());
        let commit = setting("MERCURY_COMMIT")
            .or_else(|| setting("GITHUB_SHA"))
            .or_else(|| {
                let output = Command::new("git").args(["rev-parse", "HEAD"]).output().ok()?;
                output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
                    .filter(|sha| !sha.is_empty())
            })
            .unwrap_or_else(|| {
                eprintln!("warning: release commit is unknown; set MERCURY_COMMIT when building outside Git");
                "unknown".into()
            });
        let environment = setting("MERCURY_ENVIRONMENT").unwrap_or_else(|| "development".into());
        let built_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock predates Unix epoch")
            .as_secs();
        let release = format!("project-mercury@{version}+{commit}");
        Self {
            version,
            commit,
            environment,
            built_at,
            release,
        }
    }
}

pub(crate) fn setting(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
