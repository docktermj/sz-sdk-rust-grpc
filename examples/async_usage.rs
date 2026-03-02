//! Demonstrates using sz-sdk-rust-grpc from async (tokio) code.
//!
//! All SDK methods in this crate are synchronous and internally call
//! `block_on()`. Calling them directly from an async context will panic.
//! Use `tokio::task::spawn_blocking` to bridge safely.
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
//! cargo run --example async_usage
//! ```

use sz_sdk::SzAbstractFactory;
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

#[tokio::main]
async fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // Create the factory on a blocking thread since construction connects
    // to the server (which internally uses block_on).
    let factory = tokio::task::spawn_blocking(move || {
        SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect")
    })
    .await
    .expect("spawn_blocking panicked");

    // Clone the factory for use in another blocking task.
    let f = factory.clone();
    let version = tokio::task::spawn_blocking(move || {
        let product = f.create_product().expect("failed to create product");
        product.get_version().expect("failed to get version")
    })
    .await
    .expect("spawn_blocking panicked");

    println!("Version (from async): {version}");

    // You can run multiple SDK calls concurrently using separate spawn_blocking tasks.
    let f1 = factory.clone();
    let f2 = factory.clone();

    let (license, health) = tokio::join!(
        tokio::task::spawn_blocking(move || {
            let product = f1.create_product().expect("failed to create product");
            product.get_license().expect("failed to get license")
        }),
        tokio::task::spawn_blocking(move || f2.check_health()),
    );

    println!("License: {}", license.expect("spawn_blocking panicked"));
    health
        .expect("spawn_blocking panicked")
        .expect("health check failed");
    println!("Health check passed!");
}
