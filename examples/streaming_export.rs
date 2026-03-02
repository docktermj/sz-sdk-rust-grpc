//! Streaming export example: export entities to a file with constant memory
//! usage and a progress counter.
//!
//! Demonstrates the streaming export API vs the all-in-memory
//! `export_json_as_string()` approach. For large repositories, streaming
//! avoids buffering the entire export in memory.
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
//! cargo run --example streaming_export
//! ```

use std::io::Write;

use sz_sdk::SzEngine;
use sz_sdk_rust_grpc::SzEngineGrpc;

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = sz_sdk_rust_grpc::SzAbstractFactoryGrpc::new_from_url(&grpc_url)
        .expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}\n");

    let mut engine = SzEngineGrpc::new(factory.channel());

    // Add some sample records so there's data to export.
    println!("Adding sample records...");
    let records = [
        (
            "CUSTOMERS",
            "EXPORT-1",
            r#"{"NAME_FULL": "Alice Johnson", "DATE_OF_BIRTH": "1990-01-15"}"#,
        ),
        (
            "CUSTOMERS",
            "EXPORT-2",
            r#"{"NAME_FULL": "Bob Smith", "PHONE_NUMBER": "555-0100"}"#,
        ),
        (
            "CUSTOMERS",
            "EXPORT-3",
            r#"{"NAME_FULL": "Carol Davis", "ADDR_FULL": "100 Main St"}"#,
        ),
    ];
    for (ds, rid, json) in &records {
        engine
            .add_record(ds, rid, json, 0)
            .expect("failed to add record");
    }
    println!("Added {} records.\n", records.len());

    // --- Approach 1: Streaming export to file (constant memory) ---
    let out_path = std::env::temp_dir().join("sz_streaming_export.jsonl");
    println!("Streaming JSON export to {}...", out_path.display());

    let iter = engine
        .stream_export_json_entity_report(0)
        .expect("failed to start streaming export");

    let mut file = std::fs::File::create(&out_path).expect("failed to create output file");
    let mut chunk_count = 0u64;
    let mut total_bytes = 0u64;

    for chunk_result in iter {
        let chunk = chunk_result.expect("error during streaming");
        file.write_all(chunk.as_bytes())
            .expect("failed to write chunk");
        total_bytes += chunk.len() as u64;
        chunk_count += 1;
        print!("\r  Chunks: {chunk_count}, Bytes: {total_bytes}");
    }
    file.flush().expect("failed to flush");
    println!("\n  Done! Wrote {total_bytes} bytes in {chunk_count} chunks.\n");

    // --- Approach 2: All-in-memory (for comparison) ---
    println!("In-memory JSON export...");
    let full = engine.export_json_as_string(0).expect("full export failed");
    println!("  Got {} bytes in a single String.\n", full.len());

    // --- Approach 3: Convenience file export ---
    let csv_path = std::env::temp_dir().join("sz_streaming_export.csv");
    println!("CSV export to file via convenience method...");
    engine
        .export_csv_entity_report_to_file("", &csv_path, 0)
        .expect("CSV file export failed");
    let csv_size = std::fs::metadata(&csv_path).map(|m| m.len()).unwrap_or(0);
    println!("  Wrote {csv_size} bytes to {}.\n", csv_path.display());

    // Cleanup.
    for (ds, rid, _) in &records {
        let _ = engine.delete_record(ds, rid, 0);
    }
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&csv_path);
    println!("Cleaned up records and temp files.");
}
