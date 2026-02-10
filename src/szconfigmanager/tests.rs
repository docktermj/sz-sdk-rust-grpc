use super::*;
use sz_sdk::SzConfigManager;

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_szconfigmanager() -> SzConfigManagerGrpc {
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    SzConfigManagerGrpc::new(channel)
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
fn test_destroy() {
    let mut config_manager = get_szconfigmanager();
    let result = config_manager.destroy();
    assert!(result.is_ok());
}

#[test]
fn test_get_default_config_id() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.get_default_config_id();
    assert!(result.is_ok());
}

#[test]
fn test_get_config_registry() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.get_config_registry();
    assert!(result.is_ok_and(is_valid_json));
}

#[test]
fn test_create_config_from_template() {
    let config_manager = get_szconfigmanager();
    let result = config_manager.create_config_from_template();
    assert!(result.is_ok());
}
