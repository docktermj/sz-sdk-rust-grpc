#[cfg(test)]
mod tests;

use sz_sdk::{SzConfig, SzError};
use tonic::transport::Channel;

use crate::pb_szconfigmanager::sz_config_manager_client::SzConfigManagerClient;
use crate::pb_szconfigmanager::*;
use crate::runtime::runtime;
use crate::szconfig::SzConfigGrpc;

/// gRPC implementation of [`sz_sdk::SzConfigManager`].
///
/// Also holds a reference to the config service channel so it can create
/// `SzConfigGrpc` instances from the `create_config_from_*` methods.
pub struct SzConfigManagerGrpc {
    client: SzConfigManagerClient<Channel>,
    channel: Channel,
}

impl SzConfigManagerGrpc {
    /// Creates a new `SzConfigManagerGrpc` from a gRPC channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzConfigManagerClient::new(channel.clone()),
            channel,
        }
    }
}

impl sz_sdk::SzConfigManager for SzConfigManagerGrpc {
    fn create_config_from_config_id(&self, config_id: i64) -> Result<Box<dyn SzConfig>, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_config(GetConfigRequest { config_id }).await })
            .map_err(grpc_to_sz_error)?;
        let config_definition = response.into_inner().result;
        Ok(Box::new(SzConfigGrpc::new(
            self.channel.clone(),
            config_definition,
        )))
    }

    fn create_config_from_string(
        &self,
        config_definition: &str,
    ) -> Result<Box<dyn SzConfig>, SzError> {
        Ok(Box::new(SzConfigGrpc::new(
            self.channel.clone(),
            config_definition.to_string(),
        )))
    }

    fn create_config_from_template(&self) -> Result<Box<dyn SzConfig>, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_template_config(GetTemplateConfigRequest {})
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        let config_definition = response.into_inner().result;
        Ok(Box::new(SzConfigGrpc::new(
            self.channel.clone(),
            config_definition,
        )))
    }

    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    fn get_config_registry(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_config_registry(GetConfigRegistryRequest {})
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_default_config_id(&self) -> Result<i64, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_default_config_id(GetDefaultConfigIdRequest {})
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn register_config(
        &mut self,
        config_definition: &str,
        config_comment: &str,
    ) -> Result<i64, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .register_config(RegisterConfigRequest {
                        config_definition: config_definition.to_string(),
                        config_comment: config_comment.to_string(),
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn replace_default_config_id(
        &mut self,
        current_default_config_id: i64,
        new_default_config_id: i64,
    ) -> Result<(), SzError> {
        let mut client = self.client.clone();
        runtime()
            .block_on(async {
                client
                    .replace_default_config_id(ReplaceDefaultConfigIdRequest {
                        current_default_config_id,
                        new_default_config_id,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(())
    }

    fn set_default_config(
        &mut self,
        config_definition: &str,
        config_comment: &str,
    ) -> Result<i64, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .set_default_config(SetDefaultConfigRequest {
                        config_definition: config_definition.to_string(),
                        config_comment: config_comment.to_string(),
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn set_default_config_id(&mut self, config_id: i64) -> Result<(), SzError> {
        let mut client = self.client.clone();
        runtime()
            .block_on(async {
                client
                    .set_default_config_id(SetDefaultConfigIdRequest { config_id })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(())
    }
}

fn grpc_to_sz_error(status: tonic::Status) -> SzError {
    SzError::General {
        code: status.code() as i32,
        message: status.message().to_string(),
    }
}
