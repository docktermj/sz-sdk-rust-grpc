//! Entity comparison example: analyze how records resolve into entities.
//!
//! Demonstrates adding overlapping records, inspecting resolved entities
//! with parsed types, comparing entity attributes, and using `why_entities`
//! to explain relationships.
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
//! cargo run --example entity_comparison
//! ```

use sz_sdk::flags::*;
use sz_sdk::parameters::*;
use sz_sdk::SzEngine;
use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigManagerGrpc, SzEngineGrpc};

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}\n");

    // Ensure CUSTOMERS data source exists.
    let mut cm = SzConfigManagerGrpc::new(factory.channel());
    cm.ensure_data_sources(&["CUSTOMERS"], "entity_comparison example")
        .expect("failed to ensure data sources");

    let mut engine = SzEngineGrpc::new(factory.channel());

    // --- 1. Add records with overlapping attributes ---
    println!("1. Adding records with overlapping attributes...");
    let records = &[
        ("CUSTOMERS", "CMP_1001", r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "CMP_1001", "PRIMARY_NAME_LAST": "Smith", "PRIMARY_NAME_FIRST": "Robert", "DATE_OF_BIRTH": "12/11/1978", "PHONE_NUMBER": "702-919-1300"}"#),
        ("CUSTOMERS", "CMP_1002", r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "CMP_1002", "PRIMARY_NAME_LAST": "Smith", "PRIMARY_NAME_FIRST": "Bob", "DATE_OF_BIRTH": "11/12/1978", "PHONE_NUMBER": "702-919-1300"}"#),
        ("CUSTOMERS", "CMP_1003", r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "CMP_1003", "PRIMARY_NAME_LAST": "Jones", "PRIMARY_NAME_FIRST": "Alice", "DATE_OF_BIRTH": "05/15/1985", "EMAIL_ADDRESS": "alice@example.com"}"#),
    ];
    for &(ds, rid, json) in records {
        engine
            .add_record(ds, rid, json, SZ_WITHOUT_INFO)
            .unwrap_or_else(|e| panic!("failed to add {rid}: {e}"));
    }
    println!("   Added {} records.\n", records.len());

    // --- 2. Get parsed entity data ---
    println!("2. Inspecting resolved entities...");
    let entity_a = engine
        .get_entity_parsed(
            engine
                .lookup_entity_id("CUSTOMERS", "CMP_1001", SZ_NO_FLAGS)
                .expect("lookup failed"),
            SZ_ENTITY_DEFAULT_FLAGS,
        )
        .expect("get_entity_parsed failed");
    let entity_b = engine
        .get_entity_parsed(
            engine
                .lookup_entity_id("CUSTOMERS", "CMP_1003", SZ_NO_FLAGS)
                .expect("lookup failed"),
            SZ_ENTITY_DEFAULT_FLAGS,
        )
        .expect("get_entity_parsed failed");

    println!("   Entity A: {entity_a}");
    println!("   Entity B: {entity_b}");
    println!();

    // --- 3. Compare entities ---
    println!("3. Comparing entities...");
    if entity_a.entity_id == entity_b.entity_id {
        println!("   Same entity! All records resolved together.");
    } else {
        println!("   Different entities.");
        println!(
            "   Entity A has {} records, Entity B has {} records.",
            entity_a.record_count, entity_b.record_count
        );
    }
    println!();

    // --- 4. Why entities ---
    println!("4. Analyzing why entities are (or aren't) the same...");
    let why_result = engine
        .why_entities(entity_a.entity_id, entity_b.entity_id, SZ_NO_FLAGS)
        .expect("why_entities failed");
    let parsed: serde_json::Value =
        serde_json::from_str(&why_result).expect("invalid JSON from why_entities");
    if let Some(results) = parsed["WHY_RESULTS"].as_array() {
        for (i, result) in results.iter().enumerate() {
            let match_info = &result["MATCH_INFO"];
            let match_level = match_info["MATCH_LEVEL"]
                .as_i64()
                .unwrap_or(-1);
            let match_key = match_info["MATCH_KEY"]
                .as_str()
                .unwrap_or("(none)");
            println!("   Result {}: match_level={match_level}, match_key={match_key}", i + 1);
        }
    }
    println!();

    // --- 5. Search for an entity by attributes ---
    println!("5. Searching by attributes...");
    let search_attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "Smith", "NAME_FIRST": "Robert"}], "PHONE_NUMBER": "702-919-1300"}"#;
    let results = engine
        .search_parsed(search_attrs, SZ_NO_SEARCH_PROFILE, SZ_SEARCH_BY_ATTRIBUTES_DEFAULT_FLAGS)
        .expect("search_parsed failed");
    println!("   Found {} matching entities.", results.len());
    if let Some(best) = results.best_match() {
        println!("   Best match: {best}");
    }
    let ids = results.entity_ids();
    println!("   Entity IDs: {ids:?}");
    println!();

    // --- 6. Cleanup ---
    println!("6. Cleaning up...");
    for &(ds, rid, _) in records {
        let _ = engine.delete_record(ds, rid, SZ_WITHOUT_INFO);
    }
    println!("   Done.");
}
