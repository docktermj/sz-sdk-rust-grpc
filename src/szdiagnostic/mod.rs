#[cfg(test)]
mod tests;

use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szdiagnostic::sz_diagnostic_client::SzDiagnosticClient;
use crate::pb_szdiagnostic::{
    CheckRepositoryPerformanceRequest, GetFeatureRequest, GetRepositoryInfoRequest,
    PurgeRepositoryRequest,
};

/// gRPC implementation of [`sz_sdk::SzDiagnostic`].
#[derive(Debug, Clone)]
pub struct SzDiagnosticGrpc {
    client: SzDiagnosticClient<Channel>,
}

impl SzDiagnosticGrpc {
    /// Creates a new `SzDiagnosticGrpc` from a gRPC channel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzDiagnosticGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let diagnostic = SzDiagnosticGrpc::new(factory.channel());
    /// ```
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzDiagnosticClient::new(channel),
        }
    }
}

impl std::fmt::Display for SzDiagnosticGrpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SzDiagnosticGrpc")
    }
}

/// Parsed repository information from the Senzing server.
///
/// Returned by [`SzDiagnosticGrpc::get_repository_info_parsed`]. Fields that
/// are absent from the server response will use default values.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RepositoryInfo {
    /// Number of records in the repository.
    pub record_count: i64,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl std::fmt::Display for RepositoryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Repository ({} records)", self.record_count)
    }
}

/// Parsed performance report from a repository benchmark.
///
/// Returned by [`SzDiagnosticGrpc::get_performance_report`].
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PerformanceReport {
    /// Number of insert operations per second during the benchmark.
    pub inserts_per_second: i64,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} inserts/sec", self.inserts_per_second)
    }
}

impl SzDiagnosticGrpc {
    /// Returns parsed repository information from the server.
    ///
    /// This is a convenience wrapper around
    /// [`get_repository_info()`](sz_sdk::SzDiagnostic::get_repository_info)
    /// that parses the JSON response into a [`RepositoryInfo`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzDiagnosticGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let diagnostic = SzDiagnosticGrpc::new(factory.channel());
    /// let info = diagnostic.get_repository_info_parsed().unwrap();
    /// println!("Records: {}", info.record_count);
    /// ```
    pub fn get_repository_info_parsed(&self) -> Result<RepositoryInfo, SzError> {
        use sz_sdk::SzDiagnostic;
        let raw_json = self.get_repository_info()?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("repository info", e))?;
        let record_count = parsed
            .get("dataStores")
            .and_then(|ds| ds.as_array())
            .and_then(|arr| arr.first())
            .and_then(|store| store.get("recordCount"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        Ok(RepositoryInfo {
            record_count,
            raw_json,
        })
    }

    /// Runs a performance benchmark and returns a parsed report.
    ///
    /// This is a convenience wrapper around
    /// [`check_repository_performance()`](sz_sdk::SzDiagnostic::check_repository_performance)
    /// that parses the JSON response into a [`PerformanceReport`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzDiagnosticGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let diagnostic = SzDiagnosticGrpc::new(factory.channel());
    /// let report = diagnostic.get_performance_report(3).unwrap();
    /// println!("{report}");
    /// ```
    pub fn get_performance_report(
        &self,
        seconds_to_run: i32,
    ) -> Result<PerformanceReport, SzError> {
        use sz_sdk::SzDiagnostic;
        let raw_json = self.check_repository_performance(seconds_to_run)?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("performance report", e))?;
        let inserts_per_second = parsed
            .get("numRecordsInsertedPerSecond")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        Ok(PerformanceReport {
            inserts_per_second,
            raw_json,
        })
    }
}

impl sz_sdk::SzDiagnostic for SzDiagnosticGrpc {
    /// Benchmarks the repository for the given duration and returns
    /// a JSON report with throughput metrics.
    fn check_repository_performance(&self, seconds_to_run: i32) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            check_repository_performance(CheckRepositoryPerformanceRequest { seconds_to_run })
        )?
        .into_inner()
        .result)
    }

    /// No-op for gRPC — the server manages its own lifecycle.
    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    /// Returns JSON details about a specific internal feature by ID.
    fn get_feature(&self, feature_id: i64) -> Result<String, SzError> {
        Ok(
            grpc_call!(self, get_feature(GetFeatureRequest { feature_id }))?
                .into_inner()
                .result,
        )
    }

    /// Returns a JSON summary of the repository (record counts, etc.).
    fn get_repository_info(&self) -> Result<String, SzError> {
        Ok(
            grpc_call!(self, get_repository_info(GetRepositoryInfoRequest {}))?
                .into_inner()
                .result,
        )
    }

    /// Deletes all records from the repository. Use with caution.
    fn purge_repository(&mut self) -> Result<(), SzError> {
        grpc_call!(self, purge_repository(PurgeRepositoryRequest {}))?;
        Ok(())
    }
}
