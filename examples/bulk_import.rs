//! Bulk import example: load records from JSONL data with batching,
//! progress reporting, and retry for transient errors.
//!
//! Demonstrates the common real-world pattern of ingesting a dataset
//! into Senzing using [`add_records_batch`] with [`with_retry`].
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
//! cargo run --example bulk_import
//! ```

use std::time::{Duration, Instant};

use sz_sdk::SzEngine;
use sz_sdk_rust_grpc::{with_retry, SzAbstractFactoryGrpc, SzEngineGrpc};

/// Sample records simulating a JSONL file. In production you would read
/// these from a file line by line.
const SAMPLE_RECORDS: &[(&str, &str, &str)] = &[
    (
        "CUSTOMERS",
        "BULK-1001",
        r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "BULK-1001", "PRIMARY_NAME_LAST": "Anderson", "PRIMARY_NAME_FIRST": "Robert", "DATE_OF_BIRTH": "1985-03-15", "ADDR_LINE1": "100 Oak St", "ADDR_CITY": "Portland", "ADDR_STATE": "OR"}"#,
    ),
    (
        "CUSTOMERS",
        "BULK-1002",
        r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "BULK-1002", "PRIMARY_NAME_LAST": "Anderson", "PRIMARY_NAME_FIRST": "Bob", "DATE_OF_BIRTH": "3/15/1985", "PHONE_NUMBER": "503-555-0101"}"#,
    ),
    (
        "CUSTOMERS",
        "BULK-1003",
        r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "BULK-1003", "PRIMARY_NAME_LAST": "Chen", "PRIMARY_NAME_FIRST": "Wei", "SSN_NUMBER": "123-45-6789", "EMAIL_ADDRESS": "wchen@example.com"}"#,
    ),
    (
        "CUSTOMERS",
        "BULK-1004",
        r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "BULK-1004", "PRIMARY_NAME_LAST": "Chen", "PRIMARY_NAME_FIRST": "William", "SSN_NUMBER": "123-45-6789", "ADDR_LINE1": "200 Pine Ave", "ADDR_CITY": "Seattle", "ADDR_STATE": "WA"}"#,
    ),
    (
        "CUSTOMERS",
        "BULK-1005",
        r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "BULK-1005", "PRIMARY_NAME_LAST": "Garcia", "PRIMARY_NAME_FIRST": "Maria", "DATE_OF_BIRTH": "1992-07-22", "DRIVERS_LICENSE_NUMBER": "G1234567", "DRIVERS_LICENSE_STATE": "CA"}"#,
    ),
    (
        "CUSTOMERS",
        "BULK-1006",
        r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "BULK-1006", "PRIMARY_NAME_LAST": "Garcia", "PRIMARY_NAME_FIRST": "M", "DRIVERS_LICENSE_NUMBER": "G1234567", "DRIVERS_LICENSE_STATE": "CA", "PHONE_NUMBER": "415-555-0202"}"#,
    ),
];

/// Batch size for add_records_batch calls.
const BATCH_SIZE: usize = 3;

/// Retry configuration.
const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_BASE_DELAY: Duration = Duration::from_millis(200);

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}");
    println!(
        "Importing {} records in batches of {BATCH_SIZE}...\n",
        SAMPLE_RECORDS.len()
    );

    let mut engine = SzEngineGrpc::new(factory.channel());
    let start = Instant::now();
    let mut total_success = 0u32;
    let mut total_failed = 0u32;

    // Process records in batches.
    for (batch_num, chunk) in SAMPLE_RECORDS.chunks(BATCH_SIZE).enumerate() {
        let batch_start = Instant::now();

        // Use with_retry to handle transient errors at the batch level.
        let result = with_retry(MAX_RETRY_ATTEMPTS, RETRY_BASE_DELAY, || {
            Ok(engine.add_records_batch(chunk, 0))
        })
        .expect("retry logic should not fail");

        let elapsed = batch_start.elapsed();
        total_success += result.successes;
        total_failed += result.errors.len() as u32;

        print!(
            "  Batch {}: {} succeeded, {} failed ({:.1?})",
            batch_num + 1,
            result.successes,
            result.errors.len(),
            elapsed,
        );

        // Report any errors.
        if !result.is_ok() {
            println!();
            for (record_id, err) in &result {
                println!("    ERROR {record_id}: {err}");
            }
        } else {
            println!(" OK");
        }
    }

    let total_elapsed = start.elapsed();
    println!("\nImport complete:");
    println!("  Total records: {}", SAMPLE_RECORDS.len());
    println!("  Succeeded: {total_success}");
    println!("  Failed: {total_failed}");
    println!("  Time: {total_elapsed:.1?}");
    if total_success > 0 {
        let rate = total_success as f64 / total_elapsed.as_secs_f64();
        println!("  Rate: {rate:.1} records/sec");
    }

    // --- Cleanup ---
    println!("\nCleaning up...");
    for (ds, rid, _) in SAMPLE_RECORDS {
        let _ = engine.delete_record(ds, rid, 0);
    }
    println!("Done.");
}
