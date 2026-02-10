use super::*;
use sz_sdk::SzAbstractFactory;

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_factory() -> SzAbstractFactoryGrpc {
    SzAbstractFactoryGrpc::new(GRPC_URL).expect("failed to create factory")
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
fn test_create_product() {
    let factory = get_factory();
    let result = factory.create_product();
    assert!(result.is_ok());

    if let Ok(product) = result {
        let version_result = product.get_version();
        assert!(version_result.is_ok_and(is_valid_json));
    }
}

#[test]
fn test_create_diagnostic() {
    let factory = get_factory();
    let result = factory.create_diagnostic();
    assert!(result.is_ok());

    if let Ok(diagnostic) = result {
        let repo_info_result = diagnostic.get_repository_info();
        assert!(repo_info_result.is_ok_and(is_valid_json));
    }
}

#[test]
fn test_create_engine() {
    let factory = get_factory();
    let result = factory.create_engine();
    assert!(result.is_ok());
}

#[test]
fn test_create_config_manager() {
    let factory = get_factory();
    let result = factory.create_config_manager();
    assert!(result.is_ok());
}

#[test]
fn test_factory_creates_working_product() {
    let factory = get_factory();
    let mut product = factory.create_product().expect("Failed to create product");

    let license_result = product.get_license();
    assert!(license_result.is_ok_and(is_valid_json));

    let version_result = product.get_version();
    assert!(version_result.is_ok_and(is_valid_json));

    let destroy_result = product.destroy();
    assert!(destroy_result.is_ok());
}

#[test]
fn test_factory_creates_working_diagnostic() {
    let factory = get_factory();
    let mut diagnostic = factory
        .create_diagnostic()
        .expect("Failed to create diagnostic");

    let repo_info_result = diagnostic.get_repository_info();
    assert!(repo_info_result.is_ok_and(is_valid_json));

    let destroy_result = diagnostic.destroy();
    assert!(destroy_result.is_ok());
}

#[test]
fn test_factory_creates_multiple_instances() {
    let factory = get_factory();

    let product1 = factory.create_product();
    assert!(product1.is_ok());

    let product2 = factory.create_product();
    assert!(product2.is_ok());

    let diagnostic1 = factory.create_diagnostic();
    assert!(diagnostic1.is_ok());

    let diagnostic2 = factory.create_diagnostic();
    assert!(diagnostic2.is_ok());
}
