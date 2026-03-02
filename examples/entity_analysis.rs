//! Entity analysis example: add records, resolve entities, and explore
//! relationships using why/how/path analysis.
//!
//! This example demonstrates a complete analysis workflow:
//! 1. Add records from two data sources with overlapping identities
//! 2. Look up the resolved entity IDs
//! 3. Use `why_records` to understand *why* two records resolved together
//! 4. Use `how_entity_by_entity_id` to see *how* the entity was built
//! 5. Use `find_path_by_record_id` to find relationship paths
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
//! cargo run --example entity_analysis
//! ```

use sz_sdk::SzEngine;
use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzEngineGrpc};

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // Connect and verify health.
    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}\n");

    // Use SzEngineGrpc directly to access convenience methods like lookup_entity_id.
    let mut engine = SzEngineGrpc::new(factory.channel());

    // --- Step 1: Add records with overlapping identity data ---
    let record_a = r#"{
        "NAME_FULL": "Robert Smith",
        "DATE_OF_BIRTH": "1985-02-15",
        "ADDR_FULL": "123 Main St, Springfield, IL 62701",
        "SSN_NUMBER": "111-22-3333"
    }"#;

    let record_b = r#"{
        "NAME_FULL": "Bob Smith",
        "DATE_OF_BIRTH": "1985-02-15",
        "PHONE_NUMBER": "555-867-5309",
        "SSN_NUMBER": "111-22-3333"
    }"#;

    let record_c = r#"{
        "NAME_FULL": "Jane Doe",
        "DATE_OF_BIRTH": "1990-07-04",
        "ADDR_FULL": "456 Oak Ave, Springfield, IL 62702"
    }"#;

    println!("Adding records...");
    engine
        .add_record("CUSTOMERS", "ANALYSIS-1001", record_a, 0)
        .expect("failed to add record A");
    engine
        .add_record("CUSTOMERS", "ANALYSIS-1002", record_b, 0)
        .expect("failed to add record B");
    engine
        .add_record("CUSTOMERS", "ANALYSIS-1003", record_c, 0)
        .expect("failed to add record C");
    println!("Added 3 records.\n");

    // --- Step 2: Look up resolved entity IDs ---
    let entity_id_ab = engine
        .lookup_entity_id("CUSTOMERS", "ANALYSIS-1001", 0)
        .expect("failed to look up entity for record A");
    let entity_id_ab2 = engine
        .lookup_entity_id("CUSTOMERS", "ANALYSIS-1002", 0)
        .expect("failed to look up entity for record B");
    let entity_id_c = engine
        .lookup_entity_id("CUSTOMERS", "ANALYSIS-1003", 0)
        .expect("failed to look up entity for record C");

    println!("Record A (ANALYSIS-1001) -> Entity {entity_id_ab}");
    println!("Record B (ANALYSIS-1002) -> Entity {entity_id_ab2}");
    println!("Record C (ANALYSIS-1003) -> Entity {entity_id_c}");

    if entity_id_ab == entity_id_ab2 {
        println!("Records A and B resolved to the SAME entity (shared SSN + DOB).\n");
    } else {
        println!("Records A and B resolved to DIFFERENT entities.\n");
    }

    // --- Step 3: Why did records A and B resolve together? ---
    println!("=== WHY RECORDS ===");
    let why_result = engine
        .why_records(
            "CUSTOMERS",
            "ANALYSIS-1001",
            "CUSTOMERS",
            "ANALYSIS-1002",
            0,
        )
        .expect("why_records failed");
    println!("{}\n", pretty_json(&why_result));

    // --- Step 4: How was the entity built? ---
    println!("=== HOW ENTITY ===");
    let how_result = engine
        .how_entity_by_entity_id(entity_id_ab, 0)
        .expect("how_entity failed");
    println!("{}\n", pretty_json(&how_result));

    // --- Step 5: Find path between entities ---
    if entity_id_ab != entity_id_c {
        println!("=== FIND PATH (record A -> record C) ===");
        let path_result = engine
            .find_path_by_record_id(
                "CUSTOMERS",
                "ANALYSIS-1001",
                "CUSTOMERS",
                "ANALYSIS-1003",
                5, // max degrees of separation
                "",
                "",
                0,
            )
            .expect("find_path failed");
        println!("{}\n", pretty_json(&path_result));
    }

    // --- Cleanup ---
    engine
        .delete_record("CUSTOMERS", "ANALYSIS-1001", 0)
        .expect("cleanup failed");
    engine
        .delete_record("CUSTOMERS", "ANALYSIS-1002", 0)
        .expect("cleanup failed");
    engine
        .delete_record("CUSTOMERS", "ANALYSIS-1003", 0)
        .expect("cleanup failed");
    println!("Cleaned up records.");
}

/// Pretty-print a JSON string, falling back to the raw string on parse failure.
fn pretty_json(s: &str) -> String {
    serde_json::from_str::<serde_json::Value>(s)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| s.to_string())
}
