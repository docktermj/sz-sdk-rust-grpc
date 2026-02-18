use super::*;
use serial_test::serial;
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
    assert!(is_valid_json(result.unwrap()));
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
    assert!(export_result.is_ok(), "{}", export_result.as_ref().err().unwrap());
    assert!(is_valid_json(export_result.unwrap()));
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
    assert!(export_result.is_ok(), "{}", export_result.as_ref().err().unwrap());
    assert!(is_valid_json(export_result.unwrap()));
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
    let _ = config.register_data_source("RUST_TEST_SDC");
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
    let config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
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
    let config = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    let config_definition = config.export_config().expect("failed to export config");
    let new_config_id = config_manager
        .register_config(&config_definition, "rust test replace_default_config_id")
        .expect("failed to register config");
    let result = config_manager.replace_default_config_id(current_default, new_config_id);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}
