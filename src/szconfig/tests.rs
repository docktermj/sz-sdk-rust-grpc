use super::*;
use serial_test::serial;
use sz_sdk::{SzConfig, SzConfigManager};

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_szconfig() -> SzConfigGrpc {
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    let config_manager = crate::szconfigmanager::SzConfigManagerGrpc::new(channel);
    let config = config_manager
        .create_config_from_template()
        .expect("failed to create config from template");
    let config_definition = config.export_config().expect("failed to export config");
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    SzConfigGrpc::new(channel, config_definition)
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
    assert!(is_valid_json(result.unwrap()));
}

#[test]
#[serial]
fn test_get_data_source_registry() {
    let config = get_szconfig();
    let result = config.get_data_source_registry();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(result.unwrap()));
}

#[test]
#[serial]
fn test_register_data_source() {
    let mut config = get_szconfig();
    let result = config.register_data_source("RUST_TEST");
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(result.unwrap()));
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
