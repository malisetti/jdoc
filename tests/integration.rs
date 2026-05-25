use std::fs;
use std::process::Command;

#[test]
fn extracts_documented_top_level_declarations() {
    let base = std::env::temp_dir().join(format!(
        "jdoc-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("create tempdir");

    fs::write(
        base.join("one.go"),
        r#"package one

// Alpha does the first thing.
func Alpha() {}

// Beta does the second thing.
func Beta() {}
"#,
    )
    .expect("write one.go");

    fs::write(
        base.join("two.go"),
        r#"package two

// Gamma is documented.
func Gamma() {}
"#,
    )
    .expect("write two.go");

    let output = Command::new(env!("CARGO_BIN_EXE_jdoc"))
        .arg(&base)
        .arg("--json")
        .output()
        .expect("spawn jdoc");

    let _ = fs::remove_dir_all(&base);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(stdout.trim()).expect("parse json output");
    assert_eq!(entries.len(), 3, "expected 3 doc entries, got: {stdout:?}");
}
