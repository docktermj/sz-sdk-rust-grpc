//! Diagnostics example: system health dashboard.
//!
//! Demonstrates using parsed convenience types to inspect server version,
//! license, repository info, performance, and engine statistics.
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
//! cargo run --example diagnostics
//! ```

use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzDiagnosticGrpc, SzEngineGrpc, SzProductGrpc};

fn main() {
    let grpc_url =
        std::env::var("SENZING_GRPC_URL").unwrap_or_else(|_| "http://localhost:8261".to_string());

    let factory = SzAbstractFactoryGrpc::new_from_url(&grpc_url).expect("failed to connect");
    factory.check_health().expect("server not reachable");
    println!("Connected to {grpc_url}\n");

    // --- 1. Version info ---
    println!("=== Version Info ===");
    let product = SzProductGrpc::new(factory.channel());
    let version = product.get_version_info().expect("failed to get version");
    println!("  Version:       {}", version.version);
    println!("  Build version: {}", version.build_version);
    println!("  Build number:  {}", version.build_number);
    println!("  Build date:    {}", version.build_date);
    println!("  Display:       {version}");
    println!();

    // --- 2. License info ---
    println!("=== License Info ===");
    let license = product.get_license_info().expect("failed to get license");
    println!("  Customer:     {}", license.customer);
    println!("  License type: {}", license.license_type);
    println!("  Expiration:   {}", license.expiration_date);
    println!("  Record limit: {}", license.record_limit);
    println!("  Display:      {license}");
    println!();

    // --- 3. Repository info ---
    println!("=== Repository Info ===");
    let diagnostic = SzDiagnosticGrpc::new(factory.channel());
    let repo_info = diagnostic
        .get_repository_info_parsed()
        .expect("failed to get repo info");
    println!("  Record count: {}", repo_info.record_count);
    println!("  Display:      {repo_info}");
    println!();

    // --- 4. Performance report ---
    println!("=== Performance Report (1 second) ===");
    let perf = diagnostic
        .get_performance_report(1)
        .expect("failed to get performance report");
    println!("  Inserts/sec: {}", perf.inserts_per_second);
    println!("  Display:     {perf}");
    println!();

    // --- 5. Engine stats ---
    println!("=== Engine Stats ===");
    let engine = SzEngineGrpc::new(factory.channel());
    let stats = engine.get_stats_parsed().expect("failed to get stats");
    println!("  Workload: {}", stats.workload);
    println!("  Display:  {stats}");
    println!();

    println!("Done.");
}
