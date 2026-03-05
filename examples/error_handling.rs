//! Error handling patterns for the Senzing gRPC SDK.
//!
//! Demonstrates idiomatic Rust error handling with `SzError`:
//! matching specific error kinds, retry logic for transient failures,
//! and graceful degradation.
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
//! cargo run --example error_handling
//! ```

use sz_sdk::{SzAbstractFactory, SzError};
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // --- Connection error handling ---
    println!("1. Connection error handling");
    let factory = match SzAbstractFactoryGrpc::new_from_url(&grpc_url) {
        Ok(f) => f,
        Err(ref e) if e.is_bad_input() => {
            eprintln!("Invalid URL: {e}");
            return;
        }
        Err(e) => {
            eprintln!("Connection failed: {e}");
            return;
        }
    };
    println!("   Connected to {grpc_url}");

    // --- Handling unknown data source ---
    println!("\n2. Handling unknown data source");
    let mut engine = factory.create_engine().expect("failed to create engine");
    let record = r#"{"DATA_SOURCE": "BOGUS_SOURCE", "RECORD_ID": "1"}"#;
    match engine.add_record("BOGUS_SOURCE", "1", record, 0) {
        Ok(_) => println!("   Record added (unexpected)"),
        Err(ref e) if e.is_unknown_data_source() => {
            println!("   Caught UnknownDataSource: {e}");
        }
        Err(ref e) if e.is_bad_input() => {
            println!("   Caught BadInput: {e}");
        }
        Err(e) => {
            println!("   Unexpected error: {e}");
        }
    }

    // --- Handling entity not found ---
    println!("\n3. Handling entity not found");
    match engine.get_entity_by_entity_id(-999, 0) {
        Ok(json) => println!("   Found entity: {json}"),
        Err(ref e) if e.is_not_found() => {
            println!("   Entity not found (expected): {e}");
        }
        Err(e) => {
            println!("   Other error: {e}");
        }
    }

    // --- Retry logic for transient errors ---
    println!("\n4. Retry pattern for transient errors");
    let result = retry_with_backoff(3, || engine.get_entity_by_entity_id(-999, 0));
    match result {
        Ok(_) => println!("   Succeeded after retry"),
        Err(e) => println!("   Final error after retries: {e}"),
    }

    // --- Using factory after close ---
    println!("\n5. Handling closed factory");
    let mut factory2 = SzAbstractFactoryGrpc::new_from_url(&grpc_url).unwrap();
    factory2.close().unwrap();
    match factory2.create_engine() {
        Ok(_) => println!("   Created engine (unexpected)"),
        Err(ref e) if e.is_not_initialized() => {
            println!("   Factory closed (expected): {e}");
        }
        Err(e) => {
            println!("   Unexpected error: {e}");
        }
    }

    println!("\nDone.");
}

/// Retries a fallible operation with exponential backoff.
///
/// Only retries on retryable errors (DatabaseTransient, DatabaseConnectionLost,
/// RetryTimeoutExceeded). All other errors are returned immediately.
fn retry_with_backoff<T, F>(max_attempts: u32, mut operation: F) -> Result<T, SzError>
where
    F: FnMut() -> Result<T, SzError>,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) if e.is_retryable() && attempt < max_attempts => {
                let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt - 1));
                println!(
                    "   Attempt {attempt}/{max_attempts} failed (retryable), waiting {delay:?}"
                );
                std::thread::sleep(delay);
            }
            Err(e) => return Err(e),
        }
    }
}
