//! Retry pattern example: demonstrate automatic retry with exponential
//! backoff for transient Senzing errors.
//!
//! Uses [`sz_sdk_rust_grpc::is_retryable`] to decide whether a failed
//! operation should be retried. Transient errors (deadlocks, lost
//! connections, retry timeouts) are retried with exponential backoff
//! and jitter, while permanent errors (bad input, not found) fail
//! immediately.
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
//! cargo run --example retry_pattern
//! ```

use std::thread;
use std::time::Duration;

use sz_sdk::SzEngine;
use sz_sdk_rust_grpc::{is_retryable, SzAbstractFactoryGrpc, SzEngineGrpc};

/// Retry configuration.
const MAX_RETRIES: u32 = 5;
const BASE_DELAY: Duration = Duration::from_millis(100);
const MAX_DELAY: Duration = Duration::from_secs(10);

/// Executes a fallible operation with exponential backoff and jitter.
///
/// Retries only if `is_retryable` returns `true` for the error. Permanent
/// errors are returned immediately.
fn with_retry<T, F>(operation_name: &str, mut f: F) -> Result<T, sz_sdk::SzError>
where
    F: FnMut() -> Result<T, sz_sdk::SzError>,
{
    let mut attempt = 0u32;
    loop {
        match f() {
            Ok(value) => {
                if attempt > 0 {
                    println!("  {operation_name}: succeeded after {attempt} retries");
                }
                return Ok(value);
            }
            Err(err) if is_retryable(&err) && attempt < MAX_RETRIES => {
                attempt += 1;
                // Exponential backoff: base * 2^attempt, capped at MAX_DELAY.
                let backoff = BASE_DELAY
                    .saturating_mul(1 << attempt.min(10))
                    .min(MAX_DELAY);
                // Add jitter: random portion of the backoff (simple approach).
                let jitter_ms = (backoff.as_millis() as u64 / 4).max(1);
                let delay = backoff + Duration::from_millis(simple_random() % jitter_ms);

                println!(
                    "  {operation_name}: attempt {attempt}/{MAX_RETRIES} failed \
                     (retryable), waiting {delay:?}..."
                );
                thread::sleep(delay);
            }
            Err(err) => {
                if !is_retryable(&err) {
                    println!("  {operation_name}: permanent error (not retryable): {err}");
                } else {
                    println!("  {operation_name}: giving up after {attempt} retries: {err}");
                }
                return Err(err);
            }
        }
    }
}

/// Simple pseudo-random number (no external dependency needed for an example).
fn simple_random() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
}

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}\n");

    let mut engine = SzEngineGrpc::new(factory.channel());

    // --- Example 1: Retryable operation (add_record) ---
    println!("1. Adding a record with retry wrapper:");
    let record = r#"{"NAME_FULL": "Retry Test Person", "DATE_OF_BIRTH": "1990-01-01"}"#;
    with_retry("add_record", || {
        engine.add_record("CUSTOMERS", "RETRY-1001", record, 0)
    })
    .expect("add_record failed after retries");
    println!("   Record added successfully.\n");

    // --- Example 2: Non-retryable error (bad data source) ---
    println!("2. Attempting operation with bad data source (permanent error):");
    let result = with_retry("add_record[bad_source]", || {
        engine.add_record("NONEXISTENT_SOURCE", "X", record, 0)
    });
    match &result {
        Ok(_) => println!("   Unexpectedly succeeded."),
        Err(err) => println!(
            "   Correctly failed immediately: {} (retryable={})\n",
            err,
            is_retryable(err)
        ),
    }

    // --- Example 3: Read operation with retry ---
    println!("3. Reading entity with retry wrapper:");
    let entity_id = engine
        .lookup_entity_id("CUSTOMERS", "RETRY-1001", 0)
        .expect("lookup failed");
    let entity = with_retry("get_entity", || {
        engine.get_entity_by_entity_id(entity_id, 0)
    })
    .expect("get_entity failed");
    println!("   Got entity ({}  bytes).\n", entity.len());

    // --- Cleanup ---
    let _ = engine.delete_record("CUSTOMERS", "RETRY-1001", 0);
    println!("Cleaned up.");
}
