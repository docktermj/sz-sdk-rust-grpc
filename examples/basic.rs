//! Basic example: connect to a Senzing gRPC server and print version info.
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
//! cargo run --example basic
//! ```

use sz_sdk::SzAbstractFactory;
use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    // Connect to the gRPC server.
    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");

    // Verify the server is reachable.
    factory.check_health().expect("server not reachable");
    println!("Connected to Senzing gRPC server at {grpc_url}");

    // Get product info.
    let product = factory.create_product().expect("failed to create product");

    let version = product.get_version().expect("failed to get version");
    println!("\nVersion:\n{version}");

    let license = product.get_license().expect("failed to get license");
    println!("\nLicense:\n{license}");
}
