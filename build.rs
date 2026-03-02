// Proto files in proto/ are sourced from the Senzing gRPC proto definitions:
//
//   Repository: https://github.com/senzing-garage/sz-sdk-proto
//   Directory:  sz-sdk-proto/szconfig.proto, szconfigmanager.proto, etc.
//
// To update:
//   1. Clone or pull the latest from senzing-garage/sz-sdk-proto.
//   2. Copy the .proto files into this project's proto/ directory.
//   3. Run `cargo build` — tonic-build will regenerate the Rust code.
//   4. Run `cargo test` to verify the generated code matches the server.
//
// The generated Rust code is included via tonic::include_proto!() in src/lib.rs
// as pb_szconfig, pb_szconfigmanager, pb_szdiagnostic, pb_szengine, pb_szproduct.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Disable server codegen; this crate only implements gRPC clients.
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "proto/szconfig.proto",
                "proto/szconfigmanager.proto",
                "proto/szdiagnostic.proto",
                "proto/szengine.proto",
                "proto/szproduct.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
