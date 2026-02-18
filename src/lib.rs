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

#[cfg(test)]
mod test_support {
    const GRPC_URL: &str = "http://localhost:8261";

    /// Runs exactly once before any test via the `ctor` crate.
    /// Registers CUSTOMERS, REFERENCE, and WATCHLIST data sources.
    #[ctor::ctor]
    fn setup_test_environment() {
        use sz_sdk::{SzAbstractFactory, SzConfigManager};
        let channel = crate::runtime::runtime()
            .block_on(async {
                tonic::transport::Channel::from_shared(GRPC_URL.to_string())
                    .unwrap()
                    .connect()
                    .await
            })
            .expect("failed to connect to gRPC server");
        let mut config_manager =
            crate::szconfigmanager::SzConfigManagerGrpc::new(channel.clone());
        let mut config = config_manager
            .create_config_from_template()
            .expect("failed to create config from template");
        for ds in &["CUSTOMERS", "REFERENCE", "WATCHLIST"] {
            let _ = config.register_data_source(ds);
        }
        let config_definition =
            config.export_config().expect("failed to export config");
        let config_id = config_manager
            .register_config(&config_definition, "rust test setup")
            .expect("failed to register config");
        config_manager
            .set_default_config_id(config_id)
            .expect("failed to set default config id");

        // Reinitialize the gRPC server's engine to pick up the new config.
        let mut factory =
            crate::szabstractfactory::SzAbstractFactoryGrpc::new_from_channel(channel);
        factory
            .reinitialize(config_id)
            .expect("failed to reinitialize with new config");
    }
}

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
