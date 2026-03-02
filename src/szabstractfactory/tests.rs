use super::*;
use crate::test_support::{is_valid_json, GRPC_URL};
use serial_test::serial;
use sz_sdk::SzAbstractFactory;

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

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
        assert!(
            version_result.is_ok(),
            "{}",
            version_result.as_ref().err().unwrap()
        );
        assert!(is_valid_json(&version_result.unwrap()));
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
        assert!(
            repo_info_result.is_ok(),
            "{}",
            repo_info_result.as_ref().err().unwrap()
        );
        assert!(is_valid_json(&repo_info_result.unwrap()));
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
    assert!(
        license_result.is_ok(),
        "{}",
        license_result.as_ref().err().unwrap()
    );
    assert!(is_valid_json(&license_result.unwrap()));

    let version_result = product.get_version();
    assert!(
        version_result.is_ok(),
        "{}",
        version_result.as_ref().err().unwrap()
    );
    assert!(is_valid_json(&version_result.unwrap()));

    let destroy_result = product.destroy();
    assert!(
        destroy_result.is_ok(),
        "{}",
        destroy_result.as_ref().err().unwrap()
    );
}

#[test]
#[serial]
fn test_factory_creates_working_diagnostic() {
    let factory = get_factory();
    let mut diagnostic = factory
        .create_diagnostic()
        .expect("Failed to create diagnostic");

    let repo_info_result = diagnostic.get_repository_info();
    assert!(
        repo_info_result.is_ok(),
        "{}",
        repo_info_result.as_ref().err().unwrap()
    );
    assert!(is_valid_json(&repo_info_result.unwrap()));

    let destroy_result = diagnostic.destroy();
    assert!(
        destroy_result.is_ok(),
        "{}",
        destroy_result.as_ref().err().unwrap()
    );
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
    assert!(
        diagnostic1.is_ok(),
        "{}",
        diagnostic1.as_ref().err().unwrap()
    );

    let diagnostic2 = factory.create_diagnostic();
    assert!(
        diagnostic2.is_ok(),
        "{}",
        diagnostic2.as_ref().err().unwrap()
    );
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
fn test_invalid_url_scheme() {
    let result = SzAbstractFactoryGrpc::new_from_url("ftp://localhost:8261");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, sz_sdk::SzError::BadInput { .. }),
        "expected BadInput, got {err:?}"
    );
}

#[test]
#[serial]
fn test_check_health() {
    let factory = get_factory();
    let result = factory.check_health();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_check_health_after_close() {
    let mut factory = get_factory();
    factory.close().unwrap();
    assert!(factory.check_health().is_err());
}

#[test]
#[serial]
fn test_close_prevents_creation() {
    let mut factory = get_factory();
    let close_result = factory.close();
    assert!(
        close_result.is_ok(),
        "{}",
        close_result.as_ref().err().unwrap()
    );

    assert!(factory.create_config_manager().is_err());
    assert!(factory.create_diagnostic().is_err());
    assert!(factory.create_engine().is_err());
    assert!(factory.create_product().is_err());
}

#[test]
#[serial]
fn test_close_is_idempotent() {
    let mut factory = get_factory();
    factory.close().unwrap();
    factory.close().unwrap(); // second close should not panic or error
    assert!(factory.is_closed());
}

#[test]
#[serial]
fn test_is_closed() {
    let mut factory = get_factory();
    assert!(!factory.is_closed());
    factory.close().unwrap();
    assert!(factory.is_closed());
}

#[test]
#[serial]
fn test_builder() {
    let factory = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .build()
        .expect("failed to build factory via builder");
    let result = factory.check_health();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_builder_missing_url() {
    let result = SzAbstractFactoryGrpc::builder().build();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), sz_sdk::SzError::BadInput { .. }),
        "expected BadInput for missing URL"
    );
}

#[test]
fn test_grpc_url_valid() {
    let url: GrpcUrl = "http://localhost:8261".parse().unwrap();
    assert_eq!(url.as_str(), "http://localhost:8261");
    assert_eq!(format!("{url}"), "http://localhost:8261");
}

#[test]
fn test_grpc_url_https() {
    let url = GrpcUrl::try_from("https://senzing.example.com:443").unwrap();
    assert_eq!(url.as_str(), "https://senzing.example.com:443");
}

#[test]
fn test_grpc_url_invalid_scheme() {
    let result = "ftp://localhost:8261".parse::<GrpcUrl>();
    assert!(result.is_err());
}

#[test]
fn test_grpc_url_from_string() {
    let url = GrpcUrl::try_from("http://localhost:8261".to_string()).unwrap();
    assert_eq!(url.as_str(), "http://localhost:8261");
}

#[test]
fn test_grpc_url_from_str_trait() {
    use std::str::FromStr;
    let url = GrpcUrl::from_str("http://localhost:8261").unwrap();
    assert_eq!(url.as_str(), "http://localhost:8261");
    let bad = GrpcUrl::from_str("ftp://localhost");
    assert!(bad.is_err());
}

#[test]
fn test_grpc_url_as_ref() {
    let url: GrpcUrl = "http://localhost:8261".parse().unwrap();
    let s: &str = url.as_ref();
    assert_eq!(s, "http://localhost:8261");
}

#[test]
fn test_grpc_url_display() {
    let url: GrpcUrl = "https://senzing.example.com:443".parse().unwrap();
    assert_eq!(url.to_string(), "https://senzing.example.com:443");
}

#[test]
fn test_grpc_url_clone_and_eq() {
    let url1: GrpcUrl = "http://localhost:8261".parse().unwrap();
    let url2 = url1.clone();
    assert_eq!(url1, url2);
}

#[test]
#[serial]
fn test_new_from_grpc_url() {
    let url: GrpcUrl = GRPC_URL.parse().unwrap();
    let factory = SzAbstractFactoryGrpc::new_from_grpc_url(&url).unwrap();
    assert!(factory.check_health().is_ok());
}

#[test]
fn test_config_validate_zero_connect_timeout() {
    let config = GrpcConnectionConfig {
        connect_timeout: std::time::Duration::ZERO,
        ..GrpcConnectionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_rpc_timeout() {
    let config = GrpcConnectionConfig {
        rpc_timeout: std::time::Duration::ZERO,
        ..GrpcConnectionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_keepalive_interval_too_small() {
    let config = GrpcConnectionConfig {
        keepalive_interval: Some(std::time::Duration::from_secs(5)),
        keepalive_timeout: std::time::Duration::from_secs(20),
        ..GrpcConnectionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_default_ok() {
    assert!(GrpcConnectionConfig::default().validate().is_ok());
}

#[test]
fn test_config_validate_presets_ok() {
    assert!(GrpcConnectionConfig::for_development().validate().is_ok());
    assert!(GrpcConnectionConfig::for_production().validate().is_ok());
}

#[test]
fn test_config_for_development() {
    let config = GrpcConnectionConfig::for_development();
    assert_eq!(config.connect_timeout, std::time::Duration::from_secs(3));
    assert_eq!(config.rpc_timeout, std::time::Duration::from_secs(30));
    assert!(config.keepalive_interval.is_none());
}

#[test]
fn test_config_for_production() {
    let config = GrpcConnectionConfig::for_production();
    assert_eq!(config.connect_timeout, std::time::Duration::from_secs(30));
    assert_eq!(config.rpc_timeout, std::time::Duration::from_secs(300));
    assert_eq!(
        config.keepalive_interval,
        Some(std::time::Duration::from_secs(60))
    );
}

#[test]
fn test_config_display() {
    let config = GrpcConnectionConfig::default();
    let display = format!("{config}");
    assert!(display.contains("connect_timeout"));
    assert!(display.contains("rpc_timeout"));
    assert!(display.contains("keepalive_interval"));
}

#[test]
#[serial]
fn test_builder_with_keepalive() {
    use std::time::Duration;
    let factory = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .keepalive_interval(Duration::from_secs(30))
        .keepalive_timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build with keepalive");
    let result = factory.check_health();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

// ------------------------------------------------------------------------
// Integration tests
// ------------------------------------------------------------------------

/// End-to-end workflow: factory -> engine -> add -> search -> export -> delete.
#[test]
#[serial]
fn test_full_workflow() {
    use crate::szengine::SzEngineGrpc;
    use crate::test_support::connect_channel;
    use sz_sdk::flags::*;
    use sz_sdk::parameters::*;

    let factory = get_factory();
    factory.check_health().expect("server not reachable");

    // Use concrete type so we can call streaming convenience methods.
    let mut engine = SzEngineGrpc::new(connect_channel());

    // Add records.
    let record_json = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "9901", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_LAST": "TestWorkflow", "PRIMARY_NAME_FIRST": "Alice"}"#;
    engine
        .add_record("CUSTOMERS", "9901", record_json, SZ_WITHOUT_INFO)
        .expect("failed to add record");

    // Search for the record.
    let search_attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "TestWorkflow"}]}"#;
    let search_result = engine
        .search_by_attributes(search_attrs, SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS)
        .expect("search failed");
    assert!(is_valid_json(&search_result));

    // Export via streaming convenience method.
    let export = engine
        .export_json_as_string(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
        .expect("export failed");
    assert!(!export.is_empty());

    // Delete and verify product info works.
    engine
        .delete_record("CUSTOMERS", "9901", SZ_WITHOUT_INFO)
        .expect("failed to delete record");

    let product = factory.create_product().expect("failed to create product");
    let version = product.get_version().expect("failed to get version");
    assert!(is_valid_json(&version));
}

/// Verify that a very short RPC timeout does not panic.
///
/// On a local server the RPC may still succeed within nanoseconds, so this
/// test only checks that no panic occurs — any `Ok` or `Err` is acceptable.
#[test]
fn test_short_rpc_timeout_does_not_panic() {
    use std::time::Duration;

    // Both timeouts must be set short so rpc_timeout >= connect_timeout.
    let result = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .connect_timeout(Duration::from_nanos(1))
        .rpc_timeout(Duration::from_nanos(1))
        .build();
    // Connection itself might succeed or fail — either way, no panic.
    if let Ok(factory) = result {
        let product_result = factory.create_product();
        if let Ok(product) = product_result {
            // Whether this times out or succeeds depends on server latency.
            // The important thing is no panic.
            let _ = product.get_version();
        }
    }
}

#[test]
fn test_builder_invalid_keepalive_interval_too_small() {
    use std::time::Duration;
    let result = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .keepalive_interval(Duration::from_secs(5))
        .keepalive_timeout(Duration::from_secs(20))
        .build();
    assert!(
        result.is_err(),
        "build() should fail when keepalive_interval < keepalive_timeout"
    );
}

#[test]
fn test_builder_invalid_timeout_rpc_less_than_connect() {
    use std::time::Duration;
    let result = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .connect_timeout(Duration::from_secs(30))
        .rpc_timeout(Duration::from_secs(5))
        .build();
    assert!(
        result.is_err(),
        "build() should fail when rpc_timeout < connect_timeout"
    );
}

#[test]
fn test_config_validate_rpc_timeout_less_than_connect_timeout() {
    let config = GrpcConnectionConfig {
        connect_timeout: std::time::Duration::from_secs(30),
        rpc_timeout: std::time::Duration::from_secs(5),
        ..GrpcConnectionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[cfg(feature = "serde")]
#[test]
fn test_grpc_url_serde_roundtrip() {
    let url: GrpcUrl = "http://localhost:8261".parse().unwrap();
    let json = serde_json::to_string(&url).expect("failed to serialize GrpcUrl");
    let deserialized: GrpcUrl = serde_json::from_str(&json).expect("failed to deserialize GrpcUrl");
    assert_eq!(url, deserialized);
}

#[cfg(feature = "serde")]
#[test]
fn test_grpc_connection_config_serde_roundtrip() {
    let config = GrpcConnectionConfig::for_production();
    let json = serde_json::to_string(&config).expect("failed to serialize config");
    let deserialized: GrpcConnectionConfig =
        serde_json::from_str(&json).expect("failed to deserialize config");
    assert_eq!(config, deserialized);
}

/// Cross-module integration: config manager → register data source →
/// set default config → reinitialize → engine add/search/delete.
#[test]
#[serial]
fn test_cross_module_integration() {
    let mut factory = get_factory();
    factory.check_health().expect("server not reachable");

    // 1. Use config manager to create and register a config with a new data source.
    let mut config_manager = factory
        .create_config_manager()
        .expect("failed to create config manager");
    let mut config = config_manager
        .create_config_from_template()
        .expect("failed to create config from template");
    for ds in &["CUSTOMERS", "REFERENCE", "WATCHLIST", "INTEGRATION_TEST"] {
        let _ = config.register_data_source(ds);
    }
    let config_definition = config.export_config().expect("failed to export config");
    let config_id = config_manager
        .set_default_config(&config_definition, "cross-module integration test")
        .expect("failed to set default config");
    assert!(config_id > 0);

    // 2. Reinitialize the engine to pick up the new config.
    factory
        .reinitialize(config_id)
        .expect("failed to reinitialize");

    // 3. Add a record using the new data source, search, and delete.
    let mut engine = factory.create_engine().expect("failed to create engine");
    let record = r#"{"DATA_SOURCE": "INTEGRATION_TEST", "RECORD_ID": "INT001", "PRIMARY_NAME_FULL": "Integration Test Person"}"#;
    engine
        .add_record("INTEGRATION_TEST", "INT001", record, 0)
        .expect("failed to add record");

    let search = engine
        .search_by_attributes(
            r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_FULL": "Integration Test Person"}]}"#,
            "",
            0,
        )
        .expect("search failed");
    assert!(is_valid_json(&search));

    engine
        .delete_record("INTEGRATION_TEST", "INT001", 0)
        .expect("failed to delete record");
}

/// Verify that clients created before close() remain usable after close().
#[test]
#[serial]
fn test_clients_survive_factory_close() {
    let mut factory = get_factory();
    let product = factory.create_product().expect("failed to create product");
    let engine = factory.create_engine().expect("failed to create engine");

    // Close the factory.
    factory.close().expect("failed to close factory");
    assert!(factory.is_closed());

    // Pre-existing clients should still work.
    let version = product
        .get_version()
        .expect("product should still work after factory close");
    assert!(is_valid_json(&version));

    let stats = engine
        .get_stats()
        .expect("engine should still work after factory close");
    assert!(is_valid_json(&stats));
}

/// Verify that a cloned builder can produce multiple working factories.
#[test]
#[serial]
fn test_builder_clone_reuse() {
    let builder = SzAbstractFactoryGrpc::builder().url(GRPC_URL);

    let factory1 = builder.clone().build().expect("first build failed");
    factory1
        .check_health()
        .expect("first factory health check failed");

    let factory2 = builder.build().expect("second build failed");
    factory2
        .check_health()
        .expect("second factory health check failed");
}

/// Verify that multiple threads can safely use cloned factories concurrently.
#[test]
#[serial]
fn test_concurrent_usage() {
    let factory = get_factory();
    let mut handles = Vec::new();
    for _ in 0..4 {
        let f = factory.clone();
        handles.push(std::thread::spawn(move || {
            let product = f.create_product().expect("failed to create product");
            let version = product.get_version().expect("failed to get version");
            assert!(is_valid_json(&version));
        }));
    }
    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

/// Verify that builder `.from_config()` applies all settings from a preset.
#[test]
#[serial]
fn test_builder_from_config() {
    let config = GrpcConnectionConfig::for_development();
    let factory = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .from_config(config)
        .build()
        .expect("failed to build with from_config");
    factory
        .check_health()
        .expect("health check failed after from_config");
}

/// Verify that `.from_config()` can be overridden by subsequent setters.
#[test]
#[serial]
fn test_builder_from_config_override() {
    use std::time::Duration;
    let factory = SzAbstractFactoryGrpc::builder()
        .url(GRPC_URL)
        .from_config(GrpcConnectionConfig::for_production())
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("failed to build with from_config + override");
    factory
        .check_health()
        .expect("health check failed after from_config override");
}

/// Verify that multiple independent factories can coexist, and closing one
/// does not affect others. Also verifies that new factories can be created
/// after a previous one was closed.
#[test]
#[serial]
fn test_multi_factory_lifecycle() {
    // Create two independent factories.
    let mut factory1 = get_factory();
    let factory2 = get_factory();

    // Both should be healthy.
    factory1
        .check_health()
        .expect("factory1 health check failed");
    factory2
        .check_health()
        .expect("factory2 health check failed");

    // Create a client from factory1 before closing it.
    let product1 = factory1
        .create_product()
        .expect("failed to create product from factory1");

    // Close factory1.
    factory1.close().expect("failed to close factory1");
    assert!(factory1.is_closed());

    // factory2 should still work fine.
    factory2
        .check_health()
        .expect("factory2 should work after factory1 closed");
    let product2 = factory2
        .create_product()
        .expect("factory2 should still create clients");
    let version2 = product2.get_version().expect("product2 should work");
    assert!(is_valid_json(&version2));

    // Pre-existing client from factory1 should still work.
    let version1 = product1
        .get_version()
        .expect("product1 should survive factory1 close");
    assert!(is_valid_json(&version1));

    // Creating a brand new factory should also work.
    let factory3 = get_factory();
    factory3
        .check_health()
        .expect("new factory3 should be healthy");
    let product3 = factory3
        .create_product()
        .expect("factory3 should create clients");
    let version3 = product3.get_version().expect("product3 should work");
    assert!(is_valid_json(&version3));
}

/// Verify that connecting to a non-existent server produces an informative error.
#[test]
fn test_connection_error_message_is_informative() {
    // Use a port that is almost certainly not running a gRPC server.
    let result = SzAbstractFactoryGrpc::new_from_url("http://127.0.0.1:19999");
    assert!(
        result.is_err(),
        "should fail to connect to non-existent server"
    );
    let err_msg = format!("{}", result.unwrap_err());
    // Error message should contain something useful — not be empty or opaque.
    assert!(!err_msg.is_empty(), "error message should not be empty");
    assert!(
        err_msg.contains("connect")
            || err_msg.contains("Connect")
            || err_msg.contains("refused")
            || err_msg.contains("error"),
        "error message should mention connection failure: {err_msg}"
    );
}
