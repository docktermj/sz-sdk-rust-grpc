use super::*;
use sz_sdk::SzProduct;

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_szproduct() -> SzProductGrpc {
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    SzProductGrpc::new(channel)
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
fn test_destroy() {
    let mut product = get_szproduct();
    let result = product.destroy();
    assert!(result.is_ok());
}

#[test]
fn test_get_license() {
    let product = get_szproduct();
    let result = product.get_license();
    assert!(result.is_ok_and(is_valid_json));
}

#[test]
fn test_get_version() {
    let product = get_szproduct();
    let result = product.get_version();
    assert!(result.is_ok_and(is_valid_json));
}
