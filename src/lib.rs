//! Rust gRPC implementation of the Senzing SDK traits.
//!
//! This crate provides gRPC-based implementations of the traits defined in
//! [`sz_sdk`]. It communicates with a remote Senzing gRPC server
//! (e.g., `senzing/serve-grpc`) over the network.
//!
//! # Usage
//!
//! Use [`SzAbstractFactoryGrpc`] to create instances of the various SDK
//! components. All components share a single gRPC channel (connection).

pub mod szabstractfactory;
pub mod szconfig;
pub mod szconfigmanager;
pub mod szdiagnostic;
pub mod szengine;
pub mod szproduct;

mod runtime;

pub use szabstractfactory::SzAbstractFactoryGrpc;
pub use szconfig::SzConfigGrpc;
pub use szconfigmanager::SzConfigManagerGrpc;
pub use szdiagnostic::SzDiagnosticGrpc;
pub use szengine::SzEngineGrpc;
pub use szproduct::SzProductGrpc;

/// Generated gRPC client code for the SzConfig service.
pub mod pb_szconfig {
    tonic::include_proto!("szconfig");
}

/// Generated gRPC client code for the SzConfigManager service.
pub mod pb_szconfigmanager {
    tonic::include_proto!("szconfigmanager");
}

/// Generated gRPC client code for the SzDiagnostic service.
pub mod pb_szdiagnostic {
    tonic::include_proto!("szdiagnostic");
}

/// Generated gRPC client code for the SzEngine service.
pub mod pb_szengine {
    tonic::include_proto!("szengine");
}

/// Generated gRPC client code for the SzProduct service.
pub mod pb_szproduct {
    tonic::include_proto!("szproduct");
}
