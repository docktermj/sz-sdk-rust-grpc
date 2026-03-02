#[cfg(test)]
mod tests;

use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szproduct::sz_product_client::SzProductClient;
use crate::pb_szproduct::{GetLicenseRequest, GetVersionRequest};

/// gRPC implementation of [`sz_sdk::SzProduct`].
#[derive(Debug, Clone)]
pub struct SzProductGrpc {
    client: SzProductClient<Channel>,
}

impl SzProductGrpc {
    /// Creates a new `SzProductGrpc` from a gRPC channel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzProductGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let product = SzProductGrpc::new(factory.channel());
    /// ```
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzProductClient::new(channel),
        }
    }
}

/// Parsed version information from the Senzing server.
///
/// Returned by [`SzProductGrpc::get_version_info`]. Fields that are absent
/// from the server response will be empty strings.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VersionInfo {
    /// Product version string (e.g., `"4.0.0"`).
    pub version: String,
    /// Build version string.
    pub build_version: String,
    /// Build number.
    pub build_number: String,
    /// Build date.
    pub build_date: String,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Senzing {} (build {})", self.version, self.build_number)
    }
}

/// Parsed license information from the Senzing server.
///
/// Returned by [`SzProductGrpc::get_license_info`]. Fields that are absent
/// from the server response will be empty strings.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LicenseInfo {
    /// Customer name.
    pub customer: String,
    /// License type (e.g., `"EVAL"`, `"PRODUCTION"`).
    pub license_type: String,
    /// License expiration date.
    pub expiration_date: String,
    /// Maximum number of records allowed.
    pub record_limit: i64,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl std::fmt::Display for LicenseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} license for {} (expires {})",
            self.license_type, self.customer, self.expiration_date
        )
    }
}

impl SzProductGrpc {
    /// Returns parsed version information from the server.
    ///
    /// This is a convenience wrapper around [`get_version()`](sz_sdk::SzProduct::get_version)
    /// that parses the JSON response into a [`VersionInfo`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzProductGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let product = SzProductGrpc::new(factory.channel());
    /// let info = product.get_version_info().unwrap();
    /// println!("Senzing version: {}", info.version);
    /// ```
    pub fn get_version_info(&self) -> Result<VersionInfo, SzError> {
        use sz_sdk::SzProduct;

        let raw_json = self.get_version()?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("version", e))?;

        let str_field = |key: &str| -> String {
            parsed
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        Ok(VersionInfo {
            version: str_field("VERSION"),
            build_version: str_field("BUILD_VERSION"),
            build_number: str_field("BUILD_NUMBER"),
            build_date: str_field("BUILD_DATE"),
            raw_json,
        })
    }

    /// Returns parsed license information from the server.
    ///
    /// This is a convenience wrapper around [`get_license()`](sz_sdk::SzProduct::get_license)
    /// that parses the JSON response into a [`LicenseInfo`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzProductGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let product = SzProductGrpc::new(factory.channel());
    /// let license = product.get_license_info().unwrap();
    /// println!("License type: {}", license.license_type);
    /// ```
    pub fn get_license_info(&self) -> Result<LicenseInfo, SzError> {
        use sz_sdk::SzProduct;

        let raw_json = self.get_license()?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("license", e))?;

        let str_field = |key: &str| -> String {
            parsed
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        let record_limit = parsed
            .get("recordLimit")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(LicenseInfo {
            customer: str_field("customer"),
            license_type: str_field("licenseType"),
            expiration_date: str_field("expireDate"),
            record_limit,
            raw_json,
        })
    }
}

impl std::fmt::Display for SzProductGrpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SzProductGrpc")
    }
}

impl sz_sdk::SzProduct for SzProductGrpc {
    /// No-op for gRPC — the server manages its own lifecycle.
    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    /// Returns license information as a JSON string.
    fn get_license(&self) -> Result<String, SzError> {
        Ok(grpc_call!(self, get_license(GetLicenseRequest {}))?
            .into_inner()
            .result)
    }

    /// Returns version information as a JSON string.
    fn get_version(&self) -> Result<String, SzError> {
        Ok(grpc_call!(self, get_version(GetVersionRequest {}))?
            .into_inner()
            .result)
    }
}
