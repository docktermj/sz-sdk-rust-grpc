use super::*;
use crate::test_support::{connect_channel, is_valid_json};
use serial_test::serial;
use sz_sdk::{SzConfig, SzConfigManager};

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn get_szconfig() -> SzConfigGrpc {
    let channel = connect_channel();
    let config_manager = crate::szconfigmanager::SzConfigManagerGrpc::new(channel);
    let config = config_manager
        .create_config_from_template()
        .expect("failed to create config from template");
    let config_definition = config.export_config().expect("failed to export config");
    SzConfigGrpc::new(connect_channel(), config_definition)
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_export_config() {
    let config = get_szconfig();
    let result = config.export_config();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_get_data_source_registry() {
    let config = get_szconfig();
    let result = config.get_data_source_registry();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_register_data_source() {
    let mut config = get_szconfig();
    let result = config.register_data_source("RUST_TEST");
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
    // Verify the data source appears in the registry.
    let registry = config
        .get_data_source_registry()
        .expect("failed to get registry");
    assert!(registry.contains("RUST_TEST"));
}

#[test]
#[serial]
fn test_unregister_data_source() {
    let mut config = get_szconfig();
    // First register, then unregister.
    config
        .register_data_source("RUST_TEST_UNREG")
        .expect("failed to register data source");
    let result = config.unregister_data_source("RUST_TEST_UNREG");
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    // Verify the data source is gone from the registry.
    let registry = config
        .get_data_source_registry()
        .expect("failed to get registry");
    assert!(!registry.contains("RUST_TEST_UNREG"));
}

#[test]
#[serial]
fn test_register_duplicate_data_source() {
    let mut config = get_szconfig();
    config
        .register_data_source("RUST_DUP_TEST")
        .expect("first register failed");
    // Registering the same data source again should either succeed
    // (idempotent) or return an error — not panic.
    let result = config.register_data_source("RUST_DUP_TEST");
    assert!(result.is_ok() || result.is_err());
}

#[test]
#[serial]
fn test_unregister_nonexistent_data_source() {
    let mut config = get_szconfig();
    let result = config.unregister_data_source("NONEXISTENT_SOURCE_99");
    // Should not panic; may succeed as a no-op or return an error.
    assert!(result.is_ok() || result.is_err());
}

#[test]
#[serial]
fn test_export_config_is_valid_json() {
    let config = get_szconfig();
    let export = config.export_config().expect("failed to export");
    let parsed: serde_json::Value = serde_json::from_str(&export).expect("invalid JSON");
    assert!(parsed.is_object());
}

#[test]
#[serial]
fn test_register_updates_config_state() {
    let mut config = get_szconfig();
    let before = config.export_config().expect("failed to export");
    config
        .register_data_source("STATE_TEST_DS")
        .expect("failed to register");
    let after = config.export_config().expect("failed to export");
    // Config should have changed after a successful registration.
    assert_ne!(before, after);
    assert!(after.contains("STATE_TEST_DS"));
}

#[test]
#[serial]
fn test_chained_mutations() {
    let mut config = get_szconfig();
    config
        .register_data_source("CHAIN_A")
        .expect("failed to register CHAIN_A");
    config
        .register_data_source("CHAIN_B")
        .expect("failed to register CHAIN_B");
    let registry = config
        .get_data_source_registry()
        .expect("failed to get registry");
    assert!(registry.contains("CHAIN_A"));
    assert!(registry.contains("CHAIN_B"));
}

/// Verify that cloning SzConfigGrpc produces independent state.
#[test]
#[serial]
fn test_clone_independence() {
    let mut original = get_szconfig();
    let clone = original.clone();

    // Mutate only the original.
    original
        .register_data_source("CLONE_TEST_DS")
        .expect("failed to register on original");

    // Original should have the new data source.
    let original_registry = original
        .get_data_source_registry()
        .expect("failed to get original registry");
    assert!(original_registry.contains("CLONE_TEST_DS"));

    // Clone should NOT have the new data source.
    let clone_export = clone.export_config().expect("failed to export clone");
    assert!(
        !clone_export.contains("CLONE_TEST_DS"),
        "clone should be independent of original mutations"
    );
}

// ------------------------------------------------------------------------
// Tests — data source registry structure
// ------------------------------------------------------------------------

/// Verify that get_data_source_registry returns a JSON object with a
/// DATA_SOURCES array containing the expected registered data sources.
#[test]
#[serial]
fn test_get_data_source_registry_structure() {
    let config = get_szconfig();
    let registry = config
        .get_data_source_registry()
        .expect("failed to get data source registry");
    let parsed: serde_json::Value =
        serde_json::from_str(&registry).expect("registry is not valid JSON");
    assert!(parsed.is_object(), "registry should be a JSON object");
    let data_sources = parsed
        .get("DATA_SOURCES")
        .expect("registry should have DATA_SOURCES key");
    assert!(data_sources.is_array(), "DATA_SOURCES should be an array");
    let arr = data_sources.as_array().unwrap();
    // The template always has at least one data source (TEST, CUSTOMERS, etc.).
    assert!(!arr.is_empty(), "DATA_SOURCES array should not be empty");
    // Each entry should have a DSRC_CODE string field.
    for entry in arr {
        assert!(
            entry.get("DSRC_CODE").is_some(),
            "each data source entry should have DSRC_CODE: {entry}"
        );
    }
}

/// Verify that registering a data source is reflected in the registry.
#[test]
#[serial]
fn test_get_data_source_registry_after_register() {
    let mut config = get_szconfig();
    config
        .register_data_source("REGISTRY_TEST_DS")
        .expect("failed to register data source");
    let registry = config
        .get_data_source_registry()
        .expect("failed to get registry");
    let parsed: serde_json::Value = serde_json::from_str(&registry).expect("invalid JSON");
    let sources = parsed
        .get("DATA_SOURCES")
        .and_then(|v| v.as_array())
        .expect("DATA_SOURCES should be an array");
    let found = sources
        .iter()
        .any(|s| s.get("DSRC_CODE").and_then(|v| v.as_str()) == Some("REGISTRY_TEST_DS"));
    assert!(found, "REGISTRY_TEST_DS should appear in DATA_SOURCES");
}

// ------------------------------------------------------------------------
// Tests — get_data_sources convenience method
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_data_sources() {
    let config = get_szconfig();
    let sources = config
        .get_data_sources()
        .expect("failed to get data sources");
    assert!(!sources.is_empty(), "should have at least one data source");
    // All returned names should be non-empty strings.
    for name in &sources {
        assert!(!name.is_empty(), "data source name should not be empty");
    }
}

#[test]
#[serial]
fn test_get_data_sources_after_register() {
    let mut config = get_szconfig();
    config
        .register_data_source("DS_CONVENIENCE_TEST")
        .expect("failed to register");
    let sources = config
        .get_data_sources()
        .expect("failed to get data sources");
    assert!(
        sources.contains(&"DS_CONVENIENCE_TEST".to_string()),
        "newly registered source should appear in list: {sources:?}"
    );
}

// ------------------------------------------------------------------------
// Tests — verify_config (gRPC-specific, not part of the SzConfig trait)
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_verify_config() {
    let config = get_szconfig();
    let result = config.verify_config();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(result.unwrap(), "valid config should verify as true");
}

#[test]
#[serial]
fn test_verify_config_invalid() {
    let config = SzConfigGrpc::new(connect_channel(), "}{not valid json".to_string());
    let result = config.verify_config();
    // Invalid config should either return false or an error — not panic.
    match result {
        Ok(valid) => assert!(!valid, "invalid config should not verify as true"),
        Err(_) => {} // error is also acceptable
    }
}

// ------------------------------------------------------------------------
// Tests — contains_data_source and data_source_count
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_contains_data_source_true() {
    let mut config = get_szconfig();
    config
        .register_data_source("CONTAINS_TEST")
        .expect("failed to register");
    assert!(
        config.contains_data_source("CONTAINS_TEST").unwrap(),
        "should contain CONTAINS_TEST"
    );
}

#[test]
#[serial]
fn test_contains_data_source_false() {
    let config = get_szconfig();
    assert!(
        !config.contains_data_source("NONEXISTENT_XYZ").unwrap(),
        "should not contain NONEXISTENT_XYZ"
    );
}

#[test]
#[serial]
fn test_data_source_count() {
    let config = get_szconfig();
    let count = config.data_source_count().expect("failed to count");
    assert!(count > 0, "template should have at least one data source");
}

#[test]
#[serial]
fn test_data_source_count_after_register() {
    let mut config = get_szconfig();
    let before = config.data_source_count().expect("count before failed");
    config
        .register_data_source("COUNT_TEST_DS")
        .expect("failed to register");
    let after = config.data_source_count().expect("count after failed");
    assert_eq!(after, before + 1);
}

// ------------------------------------------------------------------------
// Tests — add_data_sources bulk helper
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_add_data_sources() {
    let mut config = get_szconfig();
    let added = config
        .add_data_sources(&["BULK_A", "BULK_B", "BULK_C"])
        .expect("add_data_sources failed");
    assert_eq!(added, 3);
    assert!(config.contains_data_source("BULK_A").unwrap());
    assert!(config.contains_data_source("BULK_B").unwrap());
    assert!(config.contains_data_source("BULK_C").unwrap());
}

#[test]
#[serial]
fn test_add_data_sources_idempotent() {
    let mut config = get_szconfig();
    config
        .add_data_sources(&["IDEM_A", "IDEM_B"])
        .expect("first add failed");
    // Second call with overlapping sources should add only new ones.
    let added = config
        .add_data_sources(&["IDEM_A", "IDEM_B", "IDEM_C"])
        .expect("second add failed");
    assert_eq!(added, 1, "only IDEM_C should be new");
}

#[test]
#[serial]
fn test_add_data_sources_empty() {
    let mut config = get_szconfig();
    let added = config
        .add_data_sources(&[])
        .expect("empty add should succeed");
    assert_eq!(added, 0);
}

// ------------------------------------------------------------------------
// Tests — diff_data_sources convenience method
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_diff_data_sources_identical() {
    let config_a = get_szconfig();
    let config_b = get_szconfig();
    let diff = config_a
        .diff_data_sources(&config_b)
        .expect("diff_data_sources failed");
    assert!(diff.is_empty(), "identical configs should have no diff");
    assert_eq!(format!("{diff}"), "no differences");
}

#[test]
#[serial]
fn test_diff_data_sources_with_additions() {
    let config_a = get_szconfig();
    let mut config_b = get_szconfig();
    config_b
        .register_data_source("DIFF_TEST_NEW")
        .expect("failed to register");
    let diff = config_b
        .diff_data_sources(&config_a)
        .expect("diff_data_sources failed");
    assert!(
        diff.added.contains(&"DIFF_TEST_NEW".to_string()),
        "added should contain DIFF_TEST_NEW: {diff:?}"
    );
    assert!(diff.removed.is_empty(), "nothing should be removed");
    assert!(!diff.is_empty());
}

#[test]
#[serial]
fn test_diff_data_sources_with_removals() {
    let mut config_a = get_szconfig();
    config_a
        .register_data_source("DIFF_TEST_RM")
        .expect("failed to register");
    let config_b = get_szconfig();
    let diff = config_b
        .diff_data_sources(&config_a)
        .expect("diff_data_sources failed");
    assert!(
        diff.removed.contains(&"DIFF_TEST_RM".to_string()),
        "removed should contain DIFF_TEST_RM: {diff:?}"
    );
    assert!(diff.added.is_empty(), "nothing should be added");
}

#[test]
#[serial]
fn test_diff_data_sources_display() {
    let config_a = get_szconfig();
    let mut config_b = get_szconfig();
    config_b
        .register_data_source("DIFF_DISP_A")
        .expect("failed to register");
    let diff = config_b
        .diff_data_sources(&config_a)
        .expect("diff_data_sources failed");
    let display = format!("{diff}");
    assert!(
        display.contains("added"),
        "display should mention 'added': {display}"
    );
    assert!(
        display.contains("DIFF_DISP_A"),
        "display should mention added source: {display}"
    );
}
