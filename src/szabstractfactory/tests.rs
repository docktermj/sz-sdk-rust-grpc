use super::*;
use serial_test::serial;
use sz_sdk::SzAbstractFactory;

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_factory() -> SzAbstractFactoryGrpc {
    SzAbstractFactoryGrpc::new_from_url(GRPC_URL).expect("failed to create factory")
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_create_product() {
    let factory = get_factory();
    let result = factory.create_product();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());

    if let Ok(product) = result {
        let version_result = product.get_version();
        assert!(version_result.is_ok(), "{}", version_result.as_ref().err().unwrap());
    assert!(is_valid_json(version_result.unwrap()));
    }
}

#[test]
#[serial]
fn test_create_diagnostic() {
    let factory = get_factory();
    let result = factory.create_diagnostic();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());

    if let Ok(diagnostic) = result {
        let repo_info_result = diagnostic.get_repository_info();
        assert!(repo_info_result.is_ok(), "{}", repo_info_result.as_ref().err().unwrap());
    assert!(is_valid_json(repo_info_result.unwrap()));
    }
}

#[test]
#[serial]
fn test_create_engine() {
    let factory = get_factory();
    let result = factory.create_engine();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_create_config_manager() {
    let factory = get_factory();
    let result = factory.create_config_manager();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_factory_creates_working_product() {
    let factory = get_factory();
    let mut product = factory.create_product().expect("Failed to create product");

    let license_result = product.get_license();
    assert!(license_result.is_ok(), "{}", license_result.as_ref().err().unwrap());
    assert!(is_valid_json(license_result.unwrap()));

    let version_result = product.get_version();
    assert!(version_result.is_ok(), "{}", version_result.as_ref().err().unwrap());
    assert!(is_valid_json(version_result.unwrap()));

    let destroy_result = product.destroy();
    assert!(destroy_result.is_ok(), "{}", destroy_result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_factory_creates_working_diagnostic() {
    let factory = get_factory();
    let mut diagnostic = factory
        .create_diagnostic()
        .expect("Failed to create diagnostic");

    let repo_info_result = diagnostic.get_repository_info();
    assert!(repo_info_result.is_ok(), "{}", repo_info_result.as_ref().err().unwrap());
    assert!(is_valid_json(repo_info_result.unwrap()));

    let destroy_result = diagnostic.destroy();
    assert!(destroy_result.is_ok(), "{}", destroy_result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_factory_creates_multiple_instances() {
    let factory = get_factory();

    let product1 = factory.create_product();
    assert!(product1.is_ok(), "{}", product1.as_ref().err().unwrap());

    let product2 = factory.create_product();
    assert!(product2.is_ok(), "{}", product2.as_ref().err().unwrap());

    let diagnostic1 = factory.create_diagnostic();
    assert!(diagnostic1.is_ok(), "{}", diagnostic1.as_ref().err().unwrap());

    let diagnostic2 = factory.create_diagnostic();
    assert!(diagnostic2.is_ok(), "{}", diagnostic2.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_reinitialize() {
    let mut factory = get_factory();
    let engine = factory.create_engine().expect("failed to create engine");
    let config_id = engine
        .get_active_config_id()
        .expect("failed to get active config id");
    let result = factory.reinitialize(config_id);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_close_prevents_creation() {
    let mut factory = get_factory();
    let close_result = factory.close();
    assert!(close_result.is_ok(), "{}", close_result.as_ref().err().unwrap());

    assert!(factory.create_config_manager().is_err());
    assert!(factory.create_diagnostic().is_err());
    assert!(factory.create_engine().is_err());
    assert!(factory.create_product().is_err());
}
