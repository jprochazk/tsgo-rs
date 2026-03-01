use std::path::PathBuf;
use tsgo::{DiagnosticCategory, check_project};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("rust/tests/fixtures")
        .join(name)
        .join("tsconfig.json")
}

#[test]
fn test_version() {
    let v = tsgo::version();
    assert!(!v.is_empty(), "version should not be empty");
}

#[test]
fn test_error_project() {
    let config = fixture_path("error");
    let diagnostics = check_project(&config).expect("check_project should succeed");

    assert!(
        !diagnostics.is_empty(),
        "expected at least one diagnostic for error fixture"
    );

    let ts2322 = diagnostics
        .iter()
        .find(|d| d.code == 2322)
        .expect("expected TS2322 diagnostic (Type 'string' is not assignable to type 'number')");

    assert_eq!(ts2322.category, DiagnosticCategory::Error);
    assert!(
        ts2322.file.as_ref().is_some_and(|f| f.contains("test.ts")),
        "diagnostic should reference test.ts, got: {:?}",
        ts2322.file
    );
    assert_eq!(ts2322.line, 0, "should be on line 0 (0-indexed)");
}

#[test]
fn test_clean_project() {
    let config = fixture_path("clean");
    let diagnostics = check_project(&config).expect("check_project should succeed");

    assert!(
        diagnostics.is_empty(),
        "expected zero diagnostics for clean fixture, got: {diagnostics:?}"
    );
}
