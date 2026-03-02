//! Quickstart example for the Senzing gRPC SDK.
//!
//! Connects to a Senzing gRPC server, adds a record, and queries it back.
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
//! cargo run --example quickstart
//! ```

use sz_sdk::SzAbstractFactory;
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // 1. Connect and verify the server is reachable.
    println!("Connecting to {grpc_url}...");
    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url)?;
    factory.check_health()?;
    println!("Connected.");

    // 2. Print the Senzing version.
    let product = factory.create_product()?;
    let version = product.get_version()?;
    println!("\nSenzing version: {version}");

    // 3. Add a record.
    let mut engine = factory.create_engine()?;
    let record = r#"{
        "NAME_FULL": "Jane Smith",
        "ADDR_FULL": "100 Main St, Springfield, IL 62701"
    }"#;
    println!("\nAdding record...");
    engine.add_record("CUSTOMERS", "1001", record, 0)?;
    println!("Record added: CUSTOMERS / 1001");

    // 4. Retrieve the record.
    let fetched = engine.get_record("CUSTOMERS", "1001", 0)?;
    println!("\nRetrieved record: {fetched}");

    // 5. Look up the resolved entity.
    let entity = engine.get_entity_by_record_id("CUSTOMERS", "1001", 0)?;
    println!("\nResolved entity: {entity}");

    // 6. Clean up.
    engine.delete_record("CUSTOMERS", "1001", 0)?;
    println!("\nRecord deleted. Done.");

    Ok(())
}
