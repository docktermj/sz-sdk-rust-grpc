use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szconfig::sz_config_client::SzConfigClient;
use crate::pb_szconfig::{
    GetDataSourceRegistryRequest, RegisterDataSourceRequest, UnregisterDataSourceRequest,
};
use crate::runtime::runtime;

/// gRPC implementation of [`sz_sdk::SzConfig`].
///
/// Holds a `config_definition` string that is passed to each RPC call.
/// The proto API is stateless (config is a parameter), but the Rust trait
/// is stateful (config is held in the struct and mutated in place).
pub struct SzConfigGrpc {
    client: SzConfigClient<Channel>,
    config_definition: String,
}

impl SzConfigGrpc {
    /// Creates a new `SzConfigGrpc` with the given config definition.
    pub fn new(channel: Channel, config_definition: String) -> Self {
        Self {
            client: SzConfigClient::new(channel),
            config_definition,
        }
    }
}

impl sz_sdk::SzConfig for SzConfigGrpc {
    fn export_config(&self) -> Result<String, SzError> {
        Ok(self.config_definition.clone())
    }

    fn get_data_source_registry(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let config_definition = self.config_definition.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_data_source_registry(GetDataSourceRegistryRequest { config_definition })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn register_data_source(&mut self, data_source_code: &str) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let config_definition = self.config_definition.clone();
        let response = runtime()
            .block_on(async {
                client
                    .register_data_source(RegisterDataSourceRequest {
                        config_definition,
                        data_source_code: data_source_code.to_string(),
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        let inner = response.into_inner();
        self.config_definition = inner.config_definition;
        Ok(inner.result)
    }

    fn unregister_data_source(&mut self, data_source_code: &str) -> Result<(), SzError> {
        let mut client = self.client.clone();
        let config_definition = self.config_definition.clone();
        let response = runtime()
            .block_on(async {
                client
                    .unregister_data_source(UnregisterDataSourceRequest {
                        config_definition,
                        data_source_code: data_source_code.to_string(),
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        self.config_definition = response.into_inner().config_definition;
        Ok(())
    }
}

fn grpc_to_sz_error(status: tonic::Status) -> SzError {
    SzError::General {
        code: status.code() as i32,
        message: status.message().to_string(),
    }
}
