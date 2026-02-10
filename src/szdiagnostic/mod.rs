#[cfg(test)]
mod tests;

use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szdiagnostic::sz_diagnostic_client::SzDiagnosticClient;
use crate::pb_szdiagnostic::{
    CheckRepositoryPerformanceRequest, GetFeatureRequest, GetRepositoryInfoRequest,
    PurgeRepositoryRequest,
};
use crate::runtime::runtime;

/// gRPC implementation of [`sz_sdk::SzDiagnostic`].
pub struct SzDiagnosticGrpc {
    client: SzDiagnosticClient<Channel>,
}

impl SzDiagnosticGrpc {
    /// Creates a new `SzDiagnosticGrpc` from a gRPC channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzDiagnosticClient::new(channel),
        }
    }
}

impl sz_sdk::SzDiagnostic for SzDiagnosticGrpc {
    fn check_repository_performance(&self, seconds_to_run: i32) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .check_repository_performance(CheckRepositoryPerformanceRequest {
                        seconds_to_run,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    fn get_feature(&self, feature_id: i64) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_feature(GetFeatureRequest { feature_id }).await })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_repository_info(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_repository_info(GetRepositoryInfoRequest {})
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn purge_repository(&mut self) -> Result<(), SzError> {
        let mut client = self.client.clone();
        runtime()
            .block_on(async { client.purge_repository(PurgeRepositoryRequest {}).await })
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
