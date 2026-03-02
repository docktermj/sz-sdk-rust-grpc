//! Multi-threaded batch record ingestion example.
//!
//! Demonstrates a production-style pipeline: a batch of records is split
//! across worker threads, each with its own cloned engine. Errors are
//! collected per-thread and summarized at the end.
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
//! cargo run --example batch_pipeline
//! ```

use std::sync::Arc;
use sz_sdk::SzEngine;
use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzEngineGrpc};

/// A record to ingest.
struct Record {
    data_source: &'static str,
    record_id: &'static str,
    json: &'static str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // 1. Connect and verify.
    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url)?;
    factory.check_health()?;
    println!("Connected to {grpc_url}");

    // 2. Define a batch of records.
    let records = Arc::new(vec![
        Record {
            data_source: "CUSTOMERS",
            record_id: "BATCH001",
            json: r#"{"NAME_FULL": "Alice Johnson", "ADDR_FULL": "100 Main St, Springfield IL"}"#,
        },
        Record {
            data_source: "CUSTOMERS",
            record_id: "BATCH002",
            json: r#"{"NAME_FULL": "Bob Smith", "ADDR_FULL": "200 Oak Ave, Springfield IL"}"#,
        },
        Record {
            data_source: "CUSTOMERS",
            record_id: "BATCH003",
            json: r#"{"NAME_FULL": "Carol Davis", "PHONE_NUMBER": "555-0100"}"#,
        },
        Record {
            data_source: "CUSTOMERS",
            record_id: "BATCH004",
            json: r#"{"NAME_FULL": "Dave Wilson", "EMAIL_ADDRESS": "dave@example.com"}"#,
        },
    ]);

    // 3. Spawn worker threads, each with a cloned engine.
    //    Use the concrete SzEngineGrpc type (which is Send + Sync) instead
    //    of Box<dyn SzEngine> (which is not Send).
    let num_workers = 2;
    let chunk_size = (records.len() + num_workers - 1) / num_workers;

    let mut handles = Vec::new();
    for worker_id in 0..num_workers {
        let engine = SzEngineGrpc::new(factory.channel());
        let records = Arc::clone(&records);
        let start = worker_id * chunk_size;

        handles.push(std::thread::spawn(move || {
            let mut engine = engine;
            let mut successes = 0u32;
            let mut failures = Vec::new();

            let end = (start + chunk_size).min(records.len());
            for record in &records[start..end] {
                match engine.add_record(record.data_source, record.record_id, record.json, 0) {
                    Ok(_) => {
                        successes += 1;
                        println!(
                            "[worker {worker_id}] Added {}/{}",
                            record.data_source, record.record_id
                        );
                    }
                    Err(e) => {
                        failures.push((record.record_id, e.to_string()));
                    }
                }
            }

            (worker_id, successes, failures)
        }));
    }

    // 4. Collect results.
    let mut total_ok = 0u32;
    let mut total_err = 0u32;
    for handle in handles {
        let (worker_id, ok, failures) = handle.join().expect("worker panicked");
        total_ok += ok;
        total_err += failures.len() as u32;
        for (rid, msg) in &failures {
            eprintln!("[worker {worker_id}] FAILED {rid}: {msg}");
        }
    }
    println!("\nBatch complete: {total_ok} succeeded, {total_err} failed");

    // 5. Cleanup.
    let mut engine = SzEngineGrpc::new(factory.channel());
    for record in records.iter() {
        let _ = engine.delete_record(record.data_source, record.record_id, 0);
    }
    println!("Cleanup done.");

    Ok(())
}
