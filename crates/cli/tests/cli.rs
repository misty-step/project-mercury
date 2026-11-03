use std::process::Command;

#[test]
fn search_subject_math_json() {
    let out = Command::new(env!("CARGO_BIN_EXE_project-mercury"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .args(["search", "--subject", "math", "--json"])
        .output()
        .expect("binary runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let hits: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    let ids: Vec<_> = hits
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert!(
        ids.contains(&"math-academy") && ids.contains(&"khan-academy"),
        "{ids:?}"
    );
    assert!(!ids.contains(&"mentava"), "{ids:?}");
}

#[test]
fn search_by_format_and_cost_tier_flags() {
    let out = Command::new(env!("CARGO_BIN_EXE_project-mercury"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .args(["search", "--format", "in-person", "--cost-tier", "high"])
        .output()
        .expect("binary runs");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.starts_with("alpha-school"), "{stdout}");
}
