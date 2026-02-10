use super::*;
use sz_sdk::SzEngine;

const GRPC_URL: &str = "http://localhost:8261";

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_szengine() -> SzEngineGrpc {
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    SzEngineGrpc::new(channel)
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[test]
fn test_destroy() {
    let mut engine = get_szengine();
    let result = engine.destroy();
    assert!(result.is_ok());
}

#[test]
fn test_get_stats() {
    let engine = get_szengine();
    let result = engine.get_stats();
    assert!(result.is_ok_and(is_valid_json));
}

#[test]
fn test_get_active_config_id() {
    let engine = get_szengine();
    let result = engine.get_active_config_id();
    assert!(result.is_ok());
}

#[test]
fn test_count_redo_records() {
    let engine = get_szengine();
    let result = engine.count_redo_records();
    assert!(result.is_ok());
}

#[test]
fn test_prime_engine() {
    let engine = get_szengine();
    let result = engine.prime_engine();
    assert!(result.is_ok());
}
