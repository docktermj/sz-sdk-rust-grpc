use super::*;
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
    // Downcast the Box<dyn SzConfig> is not needed; just create directly.
    // Instead, get the config definition and create SzConfigGrpc directly.
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
fn test_export_config() {
    let config = get_szconfig();
    let result = config.export_config();
    assert!(result.is_ok_and(is_valid_json));
}

#[test]
fn test_get_data_source_registry() {
    let config = get_szconfig();
    let result = config.get_data_source_registry();
    assert!(result.is_ok_and(is_valid_json));
}
