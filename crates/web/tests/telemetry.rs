use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn generate(name: &str, analytics: bool) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&out);
    let mut command = Command::new(env!("CARGO_BIN_EXE_mercury-site"));
    command
        .args([
            "--data",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"),
            "--out",
        ])
        .arg(&out)
        .env("MERCURY_VERSION", "1.2.3")
        .env("MERCURY_ENVIRONMENT", "verification")
        .env("GITHUB_SHA", "must-not-override-explicit-commit")
        .env_remove("POSTHOG_KEY")
        .env_remove("POSTHOG_HOST");
    if analytics {
        command
            .env("MERCURY_COMMIT", "0123456789abcdef")
            .env("POSTHOG_KEY", "phc_test")
            .env("POSTHOG_HOST", "https://analytics.example.test/?quoted=\"&");
    } else {
        command
            .env_remove("MERCURY_COMMIT")
            .env_remove("GITHUB_SHA")
            .env("PATH", "");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    if !analytics {
        assert!(String::from_utf8_lossy(&output.stderr).contains("release commit is unknown"));
    }
    out
}

fn html_files(directory: &Path, paths: &mut Vec<PathBuf>) {
    for item in fs::read_dir(directory).unwrap() {
        let path = item.unwrap().path();
        if path.is_dir() {
            html_files(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "html") {
            paths.push(path);
        }
    }
}

#[test]
fn build_identity_and_opt_in_analytics_reach_every_page() {
    let out = generate("telemetry-enabled", true);
    let identity: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("version.json")).unwrap()).unwrap();
    assert_eq!(identity["version"], "1.2.3");
    assert_eq!(identity["commit"], "0123456789abcdef");
    assert_eq!(identity["environment"], "verification");
    assert_eq!(
        identity["release"],
        "project-mercury@1.2.3+0123456789abcdef"
    );
    assert!(identity["built_at"].as_u64().unwrap() > 0);
    let mut pages = Vec::new();
    html_files(&out, &mut pages);
    assert!(pages.len() > 51);
    for path in pages {
        let html = fs::read_to_string(&path).unwrap();
        assert!(
            html.contains(
                r#"<meta name="mercury-release" content="project-mercury@1.2.3+0123456789abcdef">"#
            ),
            "{}",
            path.display()
        );
        assert!(html.contains(r#"<meta name="mercury-environment" content="verification">"#));
        assert!(html.contains(r#"<script async src="/sentry.js"></script>"#));
        assert!(html.contains(
            r#"data-key="phc_test" data-host="https://analytics.example.test/?quoted=&quot;&amp;""#
        ));
    }
    assert!(out.join("sentry.js").is_file());
    assert!(out.join("posthog.js").is_file());
}

#[test]
fn missing_key_omits_analytics_and_missing_git_reports_unknown_commit() {
    let out = generate("telemetry-disabled", false);
    let identity: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("version.json")).unwrap()).unwrap();
    assert_eq!(identity["commit"], "unknown");
    assert!(!out.join("posthog.js").exists());
    let mut pages = Vec::new();
    html_files(&out, &mut pages);
    for path in pages {
        assert!(!fs::read_to_string(path).unwrap().contains("posthog"));
    }
}
