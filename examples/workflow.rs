//! End-to-end workflow: add a record, search, and delete.
//!
//! # Prerequisites
//!
//! Start a Senzing gRPC server on port 8261 with the CUSTOMERS data source
//! configured:
//! ```sh
//! make setup
//! ```
//!
//! # Run
//!
//! ```sh
//! cargo run --example workflow
//! ```

use sz_sdk::flags::*;
use sz_sdk::parameters::*;
use sz_sdk::SzAbstractFactory;
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // Connect using the builder pattern.
    let factory = SzAbstractFactoryGrpc::builder()
        .url(&grpc_url)
        .build()
        .expect("failed to connect");

    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}");

    // Print version.
    let product = factory.create_product().expect("failed to create product");
    let version = product.get_version().expect("failed to get version");
    println!("Senzing version: {version}");

    // Create an engine for record operations.
    let mut engine = factory.create_engine().expect("failed to create engine");

    // Add a record.
    let record_json = r#"{
        "DATA_SOURCE": "CUSTOMERS",
        "RECORD_ID": "EXAMPLE-1",
        "RECORD_TYPE": "PERSON",
        "PRIMARY_NAME_LAST": "Smith",
        "PRIMARY_NAME_FIRST": "John",
        "ADDR_FULL": "123 Main St, Springfield, IL 62701"
    }"#;
    engine
        .add_record("CUSTOMERS", "EXAMPLE-1", record_json, SZ_WITHOUT_INFO)
        .expect("failed to add record");
    println!("\nAdded record CUSTOMERS/EXAMPLE-1");

    // Get the entity for the record.
    let entity = engine
        .get_entity_by_record_id("CUSTOMERS", "EXAMPLE-1", SZ_ENTITY_DEFAULT_FLAGS)
        .expect("failed to get entity");
    println!("Entity: {entity}");

    // Search by attributes.
    let search_attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "Smith"}]}"#;
    let results = engine
        .search_by_attributes(
            search_attrs,
            SZ_NO_SEARCH_PROFILE,
            SZ_SEARCH_BY_ATTRIBUTES_DEFAULT_FLAGS,
        )
        .expect("search failed");
    println!("\nSearch results: {results}");

    // Clean up.
    engine
        .delete_record("CUSTOMERS", "EXAMPLE-1", SZ_WITHOUT_INFO)
        .expect("failed to delete record");
    println!("\nDeleted record CUSTOMERS/EXAMPLE-1");
    println!("Done!");
}
