//! Health monitor example: periodically check the gRPC server's health
//! and log status transitions.
//!
//! Demonstrates a production pattern for monitoring Senzing server
//! availability. The monitor runs in a background thread, reports state
//! transitions (healthy/unhealthy), and shuts down cleanly when signalled.
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
//! cargo run --example health_monitor
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

/// How often to check health.
const CHECK_INTERVAL: Duration = Duration::from_secs(2);

/// How long to run the monitor before shutting down (for this demo).
const DEMO_DURATION: Duration = Duration::from_secs(12);

/// Runs a health check loop until the shutdown flag is set.
fn health_monitor_loop(factory: SzAbstractFactoryGrpc, shutdown: Arc<AtomicBool>) {
    let mut was_healthy = None; // None = unknown initial state
    let mut consecutive_failures = 0u32;

    while !shutdown.load(Ordering::Relaxed) {
        let healthy = factory.check_health().is_ok();

        match (was_healthy, healthy) {
            (None, true) => {
                println!("[monitor] Initial check: server is HEALTHY");
                consecutive_failures = 0;
            }
            (None, false) => {
                println!("[monitor] Initial check: server is UNHEALTHY");
                consecutive_failures = 1;
            }
            (Some(true), false) => {
                consecutive_failures = 1;
                println!("[monitor] State change: HEALTHY -> UNHEALTHY");
            }
            (Some(false), true) => {
                consecutive_failures = 0;
                println!("[monitor] State change: UNHEALTHY -> HEALTHY (recovered)");
            }
            (Some(false), false) => {
                consecutive_failures += 1;
                if consecutive_failures % 3 == 0 {
                    println!(
                        "[monitor] Still UNHEALTHY ({consecutive_failures} consecutive failures)"
                    );
                }
            }
            (Some(true), true) => {
                // Still healthy — quiet unless verbose.
            }
        }

        was_healthy = Some(healthy);
        thread::sleep(CHECK_INTERVAL);
    }

    println!("[monitor] Shutdown signal received, exiting health monitor.");
}

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    println!("Connected to {grpc_url}");
    println!("Starting health monitor (will run for {DEMO_DURATION:?})...\n");

    // Shared shutdown flag.
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    // Spawn the monitor in a background thread.
    let monitor_factory = factory.clone();
    let monitor_handle = thread::spawn(move || {
        health_monitor_loop(monitor_factory, shutdown_clone);
    });

    // Simulate the application doing work while the monitor runs.
    let start = Instant::now();
    while start.elapsed() < DEMO_DURATION {
        thread::sleep(Duration::from_secs(1));
    }

    // Signal shutdown and wait for the monitor thread.
    println!("\nSignalling monitor to shut down...");
    shutdown.store(true, Ordering::Relaxed);
    monitor_handle.join().expect("monitor thread panicked");
    println!("Done.");
}
