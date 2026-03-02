use super::*;
use crate::test_support::{connect_channel, is_valid_json};
use serial_test::serial;
use sz_sdk::SzConfigManager;

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn get_szconfigmanager() -> SzConfigManagerGrpc {
    SzConfigManagerGrpc::new(connect_channel())
}

// ------------------------------------------------------------------------
// Tests — Existing
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_destroy() {
    let mut config_manager = get_szconfigmanager();
    let result = config_manager.destroy();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_get_default_config_id() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.get_default_config_id();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_get_config_registry() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.get_config_registry();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_create_config_from_template() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.create_config_from_template();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

// ------------------------------------------------------------------------
// Tests — New
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_create_config_from_config_id() {
    let config_manager = get_szconfigmanager();
    let config_id = config_manager
        .get_default_config_id()
        .expect("failed to get default config id");
    let config = config_manager
        .create_config_from_config_id(config_id)
        .expect("failed to create config from config id");
    let export_result = config.export_config();
    assert!(
        export_result.is_ok(),
        "{}",
        export_result.as_ref().err().unwrap()
    );
    assert!(is_valid_json(&export_result.unwrap()));
}

#[test]
#[serial]
fn test_create_config_from_string() {
    let config_manager = get_szconfigmanager();
    let template_config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    let config_definition = template_config
        .export_config()
        .expect("failed to export config");
    let config = config_manager
        .create_config_from_string(&config_definition)
        .expect("failed to create config from string");
    let export_result = config.export_config();
    assert!(
        export_result.is_ok(),
        "{}",
        export_result.as_ref().err().unwrap()
    );
    assert!(is_valid_json(&export_result.unwrap()));
}

#[test]
#[serial]
fn test_register_config() {
    let mut config_manager = get_szconfigmanager();
    let config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    let config_definition = config.export_config().expect("failed to export config");
    let result = config_manager.register_config(&config_definition, "rust test register_config");
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    let config_id = result.unwrap();
    assert!(config_id > 0);
}

#[test]
#[serial]
fn test_set_default_config() {
    let mut config_manager = get_szconfigmanager();
    let mut config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    // Register all data sources that may exist in the repository.
    for ds in &[
        "CUSTOMERS",
        "REFERENCE",
        "WATCHLIST",
        "INTEGRATION_TEST",
        "RUST_TEST_SDC",
    ] {
        let _ = config.register_data_source(ds);
    }
    let config_definition = config.export_config().expect("failed to export config");
    let result =
        config_manager.set_default_config(&config_definition, "rust test set_default_config");
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    let config_id = result.unwrap();
    assert!(config_id > 0);
}

#[test]
#[serial]
fn test_set_default_config_id() {
    let mut config_manager = get_szconfigmanager();
    let mut config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    for ds in &["CUSTOMERS", "REFERENCE", "WATCHLIST", "INTEGRATION_TEST"] {
        let _ = config.register_data_source(ds);
    }
    let config_definition = config.export_config().expect("failed to export config");
    let new_config_id = config_manager
        .register_config(&config_definition, "rust test set_default_config_id")
        .expect("failed to register config");
    let result = config_manager.set_default_config_id(new_config_id);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    let verified_id = config_manager
        .get_default_config_id()
        .expect("failed to get default config id");
    assert_eq!(verified_id, new_config_id);
}

#[test]
#[serial]
fn test_replace_default_config_id() {
    let mut config_manager = get_szconfigmanager();
    let current_default = config_manager
        .get_default_config_id()
        .expect("failed to get current default config id");
    let mut config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    for ds in &["CUSTOMERS", "REFERENCE", "WATCHLIST", "INTEGRATION_TEST"] {
        let _ = config.register_data_source(ds);
    }
    let config_definition = config.export_config().expect("failed to export config");
    let new_config_id = config_manager
        .register_config(&config_definition, "rust test replace_default_config_id")
        .expect("failed to register config");
    let result = config_manager.replace_default_config_id(current_default, new_config_id);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

/// Roundtrip: create from template, register data sources, export,
/// re-import via create_config_from_string, verify data sources survive.
#[test]
#[serial]
fn test_config_roundtrip() {
    let config_manager = get_szconfigmanager();

    // Create config from template, add data sources.
    let mut config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    config
        .register_data_source("ROUNDTRIP_A")
        .expect("failed to register ROUNDTRIP_A");
    config
        .register_data_source("ROUNDTRIP_B")
        .expect("failed to register ROUNDTRIP_B");

    // Export.
    let exported = config.export_config().expect("failed to export");

    // Re-import from string.
    let reimported = config_manager
        .create_config_from_string(&exported)
        .expect("failed to create config from string");
    let reimported_export = reimported.export_config().expect("failed to re-export");

    // The exported configs should be identical.
    assert_eq!(exported, reimported_export);

    // Verify data sources survived the roundtrip.
    let registry = reimported
        .get_data_source_registry()
        .expect("failed to get registry");
    assert!(registry.contains("ROUNDTRIP_A"));
    assert!(registry.contains("ROUNDTRIP_B"));
}

// ------------------------------------------------------------------------
// Tests — Error cases
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_create_config_from_config_id_invalid() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.create_config_from_config_id(-1);
    assert!(result.is_err(), "expected error for invalid config id");
}

#[test]
#[serial]
fn test_register_config_invalid_json() {
    let mut config_manager = get_szconfigmanager();
    let result = config_manager.register_config("}{not valid json", "bad config");
    assert!(result.is_err(), "expected error for invalid config JSON");
}

#[test]
#[serial]
fn test_replace_default_config_id_stale() {
    let mut config_manager = get_szconfigmanager();
    // Use a fabricated old_config_id that doesn't match the current default.
    let mut config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    for ds in &["CUSTOMERS", "REFERENCE", "WATCHLIST", "INTEGRATION_TEST"] {
        let _ = config.register_data_source(ds);
    }
    let config_definition = config.export_config().expect("failed to export config");
    let new_config_id = config_manager
        .register_config(&config_definition, "rust test stale replace")
        .expect("failed to register config");
    // Pass a stale old_config_id (0 is never a valid default).
    let result = config_manager.replace_default_config_id(0, new_config_id);
    assert!(result.is_err(), "expected error for stale old config id");
}

// ------------------------------------------------------------------------
// Tests — Convenience methods
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_ensure_data_sources() {
    let mut config_manager = get_szconfigmanager();
    let config_id = config_manager
        .ensure_data_sources(
            &[
                "CUSTOMERS",
                "REFERENCE",
                "WATCHLIST",
                "INTEGRATION_TEST",
                "ENSURE_TEST",
            ],
            "test ensure_data_sources",
        )
        .expect("ensure_data_sources failed");
    assert!(config_id > 0);

    // Verify the data source was registered by loading the config.
    let config = config_manager
        .create_config_from_config_id(config_id)
        .expect("failed to load config");
    let registry = config
        .get_data_source_registry()
        .expect("failed to get registry");
    assert!(registry.contains("ENSURE_TEST"));
}

#[test]
#[serial]
fn test_ensure_data_sources_idempotent() {
    let mut config_manager = get_szconfigmanager();
    // Call twice with the same sources — should succeed both times.
    let sources = &["CUSTOMERS", "REFERENCE", "WATCHLIST", "INTEGRATION_TEST"];
    let id1 = config_manager
        .ensure_data_sources(sources, "idempotent call 1")
        .expect("first ensure failed");
    let id2 = config_manager
        .ensure_data_sources(sources, "idempotent call 2")
        .expect("second ensure failed");
    assert!(id1 > 0);
    assert!(id2 > 0);
}

#[test]
#[serial]
fn test_get_config_ids() {
    let config_manager = get_szconfigmanager();
    let ids = config_manager
        .get_config_ids()
        .expect("get_config_ids failed");
    // There should always be at least one config registered.
    assert!(!ids.is_empty(), "expected at least one config ID");
    // All IDs should be positive.
    for id in &ids {
        assert!(*id > 0, "config ID should be positive, got {id}");
    }
}
