use super::*;
use sz_sdk::SzDiagnostic;

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_szdiagnostic() -> SzDiagnosticGrpc {
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    SzDiagnosticGrpc::new(channel)
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
fn test_destroy() {
    let mut diagnostic = get_szdiagnostic();
    let result = diagnostic.destroy();
    assert!(result.is_ok());
}

#[test]
fn test_check_repository_performance() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.check_repository_performance(5);
    assert!(result.is_ok_and(is_valid_json));
}

#[test]
fn test_get_feature() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.get_feature(1);
    if let Err(e) = &result {
        println!(">>>>>> get_feature error: {}", e);
    }
    assert!(result.is_err());
}

#[test]
fn test_get_repository_info() {
    let diagnostic = get_szdiagnostic();
    let result = diagnostic.get_repository_info();
    assert!(result.is_ok_and(is_valid_json));
}

#[test]
fn test_purge_repository() {
    let mut diagnostic = get_szdiagnostic();
    let result = diagnostic.purge_repository();
    assert!(result.is_ok());
}
