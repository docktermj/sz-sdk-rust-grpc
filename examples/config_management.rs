//! Config management example: full lifecycle of Senzing configuration.
//!
//! Demonstrates creating configs from templates, registering data sources,
//! diffing configs, exporting/importing, and version management.
//!
//! # Prerequisites
//!
//! Start a Senzing gRPC server on port 8261:
//! ```sh
//! make setup
//! ```
//!
//! # Run
//!
//! ```sh
//! cargo run --example config_management
//! ```

use sz_sdk::SzConfigManager;
use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}\n");

    let mut config_manager = SzConfigManagerGrpc::new(factory.channel());

    // --- 1. Create config from template ---
    println!("1. Creating config from template...");
    let template = config_manager
        .create_config_from_template()
        .expect("failed to create template config");
    let template_def = template.export_config().expect("failed to export");
    let template_config = SzConfigGrpc::new(factory.channel(), template_def.clone());
    let count = template_config
        .data_source_count()
        .expect("failed to count");
    println!("   Template has {count} data sources.\n");

    // --- 2. Register data sources ---
    println!("2. Registering custom data sources...");
    let mut config = config_manager
        .create_config_from_string(&template_def)
        .expect("failed to create config from string");
    config
        .register_data_source("CUSTOMERS")
        .expect("failed to register CUSTOMERS");
    config
        .register_data_source("WATCHLIST")
        .expect("failed to register WATCHLIST");
    let modified_def = config.export_config().expect("failed to export");
    let modified_config = SzConfigGrpc::new(factory.channel(), modified_def.clone());
    let new_count = modified_config
        .data_source_count()
        .expect("failed to count");
    println!("   Modified config has {new_count} data sources.\n");

    // --- 3. Diff configs ---
    println!("3. Comparing configs...");
    let diff = modified_config
        .diff_data_sources(&template_config)
        .expect("diff failed");
    println!("   {diff}\n");

    // --- 4. Check membership ---
    println!("4. Checking data source membership...");
    let has_customers = modified_config
        .contains_data_source("CUSTOMERS")
        .expect("contains check failed");
    let has_bogus = modified_config
        .contains_data_source("BOGUS")
        .expect("contains check failed");
    println!("   Contains CUSTOMERS: {has_customers}");
    println!("   Contains BOGUS: {has_bogus}\n");

    // --- 5. Register and set as default ---
    println!("5. Registering config and setting as default...");
    let config_id = config_manager
        .set_default_config(&modified_def, "example: config_management")
        .expect("failed to set default config");
    println!("   New default config ID: {config_id}\n");

    // --- 6. List all config versions ---
    println!("6. Listing registered config versions...");
    let ids = config_manager
        .get_config_ids()
        .expect("failed to get config IDs");
    println!("   {} configs registered:", ids.len());
    for id in &ids {
        println!("     - {id}");
    }
    println!();

    // --- 7. ensure_data_sources (idempotent) ---
    println!("7. Ensuring data sources (idempotent)...");
    let ensured_id = config_manager
        .ensure_data_sources(
            &["CUSTOMERS", "WATCHLIST", "REFERENCE"],
            "example: ensure data sources",
        )
        .expect("ensure_data_sources failed");
    println!("   Config ID after ensure: {ensured_id}\n");

    // --- 8. Export/import roundtrip ---
    println!("8. Export/import roundtrip...");
    let loaded = config_manager
        .create_config_from_config_id(ensured_id)
        .expect("failed to load config");
    let exported = loaded.export_config().expect("failed to export");
    let reimported = config_manager
        .create_config_from_string(&exported)
        .expect("failed to reimport");
    let reimported_export = reimported.export_config().expect("failed to re-export");
    assert_eq!(
        exported, reimported_export,
        "roundtrip should produce identical configs"
    );
    println!("   Roundtrip successful (configs match).\n");

    println!("Done.");
}
