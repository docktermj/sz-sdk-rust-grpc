use super::*;
use crate::test_support::{connect_channel, is_valid_json};
use serial_test::serial;
use sz_sdk::SzProduct;

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn get_szproduct() -> SzProductGrpc {
    SzProductGrpc::new(connect_channel())
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_destroy() {
    let mut product = get_szproduct();
    let result = product.destroy();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_get_license() {
    let product = get_szproduct();
    let result = product.get_license();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_get_version() {
    let product = get_szproduct();
    let result = product.get_version();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_get_version_returns_valid_structure() {
    let product = get_szproduct();
    let version = product.get_version().expect("failed to get version");
    let parsed: serde_json::Value = serde_json::from_str(&version).expect("invalid JSON");
    assert!(parsed.is_object());
}

#[test]
#[serial]
fn test_get_license_returns_valid_structure() {
    let product = get_szproduct();
    let license = product.get_license().expect("failed to get license");
    let parsed: serde_json::Value = serde_json::from_str(&license).expect("invalid JSON");
    assert!(parsed.is_object());
}

#[test]
#[serial]
fn test_destroy_is_idempotent() {
    let mut product = get_szproduct();
    product.destroy().expect("first destroy failed");
    product.destroy().expect("second destroy failed");
}

#[test]
#[serial]
fn test_get_version_info() {
    let product = get_szproduct();
    let info = product
        .get_version_info()
        .expect("failed to get version info");
    // Version string should be non-empty.
    assert!(!info.version.is_empty(), "version should not be empty");
    // Raw JSON should be valid.
    assert!(is_valid_json(&info.raw_json));
}

#[test]
#[serial]
fn test_version_info_display() {
    let product = get_szproduct();
    let info = product
        .get_version_info()
        .expect("failed to get version info");
    let display = format!("{info}");
    assert!(display.contains(&info.version));
    assert!(display.starts_with("Senzing "));
}

#[test]
#[serial]
fn test_get_license_info() {
    let product = get_szproduct();
    let info = product
        .get_license_info()
        .expect("failed to get license info");
    assert!(
        !info.license_type.is_empty(),
        "license type should not be empty"
    );
    assert!(is_valid_json(&info.raw_json));
}

#[test]
#[serial]
fn test_license_info_display() {
    let product = get_szproduct();
    let info = product
        .get_license_info()
        .expect("failed to get license info");
    let display = format!("{info}");
    assert!(!display.is_empty());
    assert!(display.contains(&info.license_type));
}

#[cfg(feature = "serde")]
#[test]
#[serial]
fn test_version_info_serde_roundtrip() {
    let product = get_szproduct();
    let info = product
        .get_version_info()
        .expect("failed to get version info");
    let json = serde_json::to_string(&info).expect("failed to serialize VersionInfo");
    let deserialized: VersionInfo =
        serde_json::from_str(&json).expect("failed to deserialize VersionInfo");
    assert_eq!(info, deserialized);
}

#[cfg(feature = "serde")]
#[test]
#[serial]
fn test_license_info_serde_roundtrip() {
    let product = get_szproduct();
    let info = product
        .get_license_info()
        .expect("failed to get license info");
    let json = serde_json::to_string(&info).expect("failed to serialize LicenseInfo");
    let deserialized: LicenseInfo =
        serde_json::from_str(&json).expect("failed to deserialize LicenseInfo");
    assert_eq!(info, deserialized);
}
