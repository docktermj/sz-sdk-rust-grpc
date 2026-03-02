use super::*;
use crate::test_support::{connect_channel, is_valid_json};
use serial_test::serial;
use sz_sdk::SzDiagnostic;

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn get_szdiagnostic() -> SzDiagnosticGrpc {
    SzDiagnosticGrpc::new(connect_channel())
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_destroy() {
    let mut diagnostic = get_szdiagnostic();
    let result = diagnostic.destroy();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_check_repository_performance() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.check_repository_performance(5);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_get_feature() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.get_feature(1);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_get_repository_info() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.get_repository_info();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_purge_repository() {
    let mut diagnostic = get_szdiagnostic();
    let result = diagnostic.purge_repository();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_get_feature_invalid_id() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.get_feature(-999);
    // Negative feature IDs should return an error.
    assert!(result.is_err(), "expected error for feature ID -999");
}

#[test]
#[serial]
fn test_check_repository_performance_zero_seconds() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.check_repository_performance(0);
    // Zero seconds may succeed (no-op) or error — should not panic.
    assert!(result.is_err() || is_valid_json(&result.unwrap()));
}

/// Verify that purge_repository is idempotent — calling it twice should not panic.
#[test]
#[serial]
fn test_purge_repository_idempotent() {
    let mut diagnostic = get_szdiagnostic();
    diagnostic.purge_repository().expect("first purge failed");
    diagnostic.purge_repository().expect("second purge failed");
}

/// Verify that get_repository_info returns valid JSON after purge_repository.
#[test]
#[serial]
fn test_get_repository_info_after_purge() {
    let mut diagnostic = get_szdiagnostic();
    diagnostic.purge_repository().expect("purge failed");
    let info = diagnostic
        .get_repository_info()
        .expect("get_repository_info failed after purge");
    assert!(
        is_valid_json(&info),
        "repo info should be valid JSON after purge"
    );
    let parsed: serde_json::Value = serde_json::from_str(&info).expect("failed to parse repo info");
    assert!(parsed.is_object());
}

// ------------------------------------------------------------------------
// Tests — Parsed convenience types
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_repository_info_parsed() {
    let diagnostic = get_szdiagnostic();
    let info = diagnostic
        .get_repository_info_parsed()
        .expect("failed to get parsed repo info");
    assert!(info.record_count >= 0);
    assert!(is_valid_json(&info.raw_json));
    // Display should work.
    let display = format!("{info}");
    assert!(display.contains("records"));
}

#[test]
#[serial]
fn test_get_performance_report() {
    let diagnostic = get_szdiagnostic();
    let report = diagnostic
        .get_performance_report(1)
        .expect("failed to get performance report");
    assert!(report.inserts_per_second >= 0);
    assert!(is_valid_json(&report.raw_json));
    let display = format!("{report}");
    assert!(display.contains("inserts/sec"));
}

#[test]
#[serial]
fn test_get_repository_info_is_valid_json() {
    let diagnostic = get_szdiagnostic();
    let info = diagnostic
        .get_repository_info()
        .expect("failed to get repo info");
    let parsed: serde_json::Value = serde_json::from_str(&info).expect("invalid JSON");
    // Should contain basic repository metadata.
    assert!(parsed.is_object());
}
