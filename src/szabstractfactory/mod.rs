#[cfg(test)]
mod tests;

use sz_sdk::{SzConfigManager, SzDiagnostic, SzEngine, SzError, SzProduct};
use tonic::transport::Channel;

use crate::pb_szdiagnostic::sz_diagnostic_client::SzDiagnosticClient;
use crate::pb_szdiagnostic::ReinitializeRequest;
use crate::pb_szengine::sz_engine_client::SzEngineClient;
use crate::runtime::runtime;
use crate::szconfigmanager::SzConfigManagerGrpc;
use crate::szdiagnostic::SzDiagnosticGrpc;
use crate::szengine::SzEngineGrpc;
use crate::szproduct::SzProductGrpc;

/// gRPC implementation of [`sz_sdk::SzAbstractFactory`].
///
/// Holds a shared [`tonic::transport::Channel`] and creates gRPC clients
/// from it on demand. The channel is cheaply cloneable (it represents a
/// pool of HTTP/2 connections).
pub struct SzAbstractFactoryGrpc {
    channel: Channel,
}

impl SzAbstractFactoryGrpc {
    /// Creates a new factory by connecting to the given gRPC URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
    ///
    /// let factory = SzAbstractFactoryGrpc::new("http://localhost:8261").unwrap();
    /// ```
    pub fn new(grpc_url: &str) -> Result<Self, SzError> {
        let channel = runtime().block_on(async {
            Channel::from_shared(grpc_url.to_string())
                .map_err(|e| SzError::General {
                    code: 0,
                    message: format!("invalid gRPC URL: {e}"),
                })?
                .connect()
                .await
                .map_err(|e| SzError::General {
                    code: 0,
                    message: format!("failed to connect to gRPC server: {e}"),
                })
        })?;
        Ok(Self { channel })
    }

    /// Creates a new factory from an existing channel.
    pub fn from_channel(channel: Channel) -> Self {
        Self { channel }
    }
}

impl sz_sdk::SzAbstractFactory for SzAbstractFactoryGrpc {
    fn close(&mut self) -> Result<(), SzError> {
        // No-op: the gRPC channel is reference-counted and will be
        // dropped when the last client goes out of scope.
        Ok(())
    }

    fn create_config_manager(&self) -> Result<Box<dyn SzConfigManager>, SzError> {
        Ok(Box::new(SzConfigManagerGrpc::new(self.channel.clone())))
    }

    fn create_diagnostic(&self) -> Result<Box<dyn SzDiagnostic>, SzError> {
        Ok(Box::new(SzDiagnosticGrpc::new(self.channel.clone())))
    }

    fn create_engine(&self) -> Result<Box<dyn SzEngine>, SzError> {
        Ok(Box::new(SzEngineGrpc::new(self.channel.clone())))
    }

    fn create_product(&self) -> Result<Box<dyn SzProduct>, SzError> {
        Ok(Box::new(SzProductGrpc::new(self.channel.clone())))
    }

    fn reinitialize(&mut self, config_id: i64) -> Result<(), SzError> {
        let mut diag_client = SzDiagnosticClient::new(self.channel.clone());
        runtime()
            .block_on(async {
                diag_client
                    .reinitialize(ReinitializeRequest { config_id })
                    .await
            })
            .map_err(|status| SzError::General {
                code: status.code() as i32,
                message: status.message().to_string(),
            })?;

        let mut engine_client = SzEngineClient::new(self.channel.clone());
        runtime()
            .block_on(async {
                engine_client
                    .reinitialize(crate::pb_szengine::ReinitializeRequest { config_id })
                    .await
            })
            .map_err(|status| SzError::General {
                code: status.code() as i32,
                message: status.message().to_string(),
            })?;

        Ok(())
    }
}
