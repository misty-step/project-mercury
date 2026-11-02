use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Command::new(env!("CARGO_BIN_EXE_project-mercury"))
        .arg("--data")
        .arg(data)
        .args(args)
        .output()
        .expect("run project-mercury")
}

#[test]
fn search_json_returns_sourced_entry() {
    let out = run(&["search", "reading age 5", "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let hits: Value = serde_json::from_slice(&out.stdout).unwrap();
    let hits = hits.as_array().unwrap();
    assert!(!hits.is_empty());

    for hit in hits {
        let entry = &hit["entry"];
        let (min, max) = (
            entry["ages"]["min"].as_u64().unwrap(),
            entry["ages"]["max"].as_u64().unwrap(),
        );
        assert!(min <= 5 && 5 <= max, "{entry}");
        let source_ids: Vec<&str> = hit["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .collect();
        let claims = entry["claims"].as_array().unwrap();
        assert!(!claims.is_empty());
        for claim in claims {
            assert!(
                source_ids.contains(&claim["source"].as_str().unwrap()),
                "{claim}"
            );
        }
    }
    let ids: Vec<&str> = hits
        .iter()
        .map(|h| h["entry"]["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"summer-reading-challenge"), "{ids:?}");
}

#[test]
fn search_text_and_no_match() {
    let out = run(&["search", "phonics"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Jolly Phonics"));

    let out = run(&["search", "astrophysics"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("No matches"));
}

#[test]
fn missing_data_dir_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_project-mercury"))
        .args(["--data", "/nonexistent", "search", "x"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
