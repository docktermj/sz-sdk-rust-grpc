#[cfg(test)]
mod tests;

use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szproduct::sz_product_client::SzProductClient;
use crate::pb_szproduct::{GetLicenseRequest, GetVersionRequest};
use crate::runtime::runtime;

/// gRPC implementation of [`sz_sdk::SzProduct`].
pub struct SzProductGrpc {
    client: SzProductClient<Channel>,
}

impl SzProductGrpc {
    /// Creates a new `SzProductGrpc` from a gRPC channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzProductClient::new(channel),
        }
    }
}

impl sz_sdk::SzProduct for SzProductGrpc {
    fn destroy(&mut self) -> Result<(), SzError> {
        // No-op: the gRPC client does not hold server-side state.
        Ok(())
    }

    fn get_license(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_license(GetLicenseRequest {}).await })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_version(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_version(GetVersionRequest {}).await })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }
}

/// Converts a gRPC `tonic::Status` into an `SzError`.
fn grpc_to_sz_error(status: tonic::Status) -> SzError {
    SzError::General {
        code: status.code() as i32,
        message: status.message().to_string(),
    }
}
