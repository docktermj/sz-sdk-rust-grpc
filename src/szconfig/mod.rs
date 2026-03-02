#[cfg(test)]
mod tests;

use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szconfig::sz_config_client::SzConfigClient;
use crate::pb_szconfig::{
    GetDataSourceRegistryRequest, RegisterDataSourceRequest, UnregisterDataSourceRequest,
    VerifyConfigRequest,
};
use crate::runtime::{grpc_to_sz_error, runtime};

/// gRPC implementation of [`sz_sdk::SzConfig`].
///
/// Holds a `config_definition` string that is passed to each RPC call.
/// The proto API is stateless (config is a parameter), but the Rust trait
/// is stateful (config is held in the struct and mutated in place).
///
/// # Cloning
///
/// Unlike the other gRPC client types (which are thin handles over a shared
/// connection pool), cloning an `SzConfigGrpc` **copies the current config
/// state**. After cloning, the original and the clone are independent —
/// mutations on one do not affect the other.
#[derive(Debug, Clone)]
pub struct SzConfigGrpc {
    client: SzConfigClient<Channel>,
    config_definition: String,
}

impl SzConfigGrpc {
    /// Creates a new `SzConfigGrpc` with the given config definition.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::SzConfigManager;
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let config_manager = SzConfigManagerGrpc::new(factory.channel());
    /// let config = config_manager.create_config_from_template().unwrap();
    /// let definition = config.export_config().unwrap();
    /// let config = SzConfigGrpc::new(factory.channel(), definition);
    /// ```
    #[must_use]
    pub fn new(channel: Channel, config_definition: String) -> Self {
        Self {
            client: SzConfigClient::new(channel),
            config_definition,
        }
    }

    /// Verifies that the current config definition is valid.
    ///
    /// Makes a gRPC call to the server to validate the config. Returns `true`
    /// if valid, `false` otherwise. This method is not part of the `SzConfig`
    /// trait — it is a gRPC-specific extension.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::SzConfigManager;
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let config_manager = SzConfigManagerGrpc::new(factory.channel());
    /// let config = config_manager.create_config_from_template().unwrap();
    /// let definition = config.export_config().unwrap();
    /// let config = SzConfigGrpc::new(factory.channel(), definition);
    /// assert!(config.verify_config().unwrap());
    /// ```
    pub fn verify_config(&self) -> Result<bool, SzError> {
        let config_definition = self.config_definition.clone();
        Ok(grpc_call!(
            self,
            verify_config(VerifyConfigRequest { config_definition })
        )?
        .into_inner()
        .result)
    }
}

impl SzConfigGrpc {
    /// Returns the list of registered data source names.
    ///
    /// This is a convenience wrapper around
    /// [`get_data_source_registry()`](sz_sdk::SzConfig::get_data_source_registry)
    /// that parses the JSON response and extracts the `DSRC_CODE` values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::SzConfigManager;
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let config_manager = SzConfigManagerGrpc::new(factory.channel());
    /// let config = config_manager.create_config_from_template().unwrap();
    /// let definition = config.export_config().unwrap();
    /// let config = SzConfigGrpc::new(factory.channel(), definition);
    /// let sources = config.get_data_sources().unwrap();
    /// for name in &sources {
    ///     println!("Data source: {name}");
    /// }
    /// ```
    pub fn get_data_sources(&self) -> Result<Vec<String>, SzError> {
        use sz_sdk::SzConfig;
        let registry_json = self.get_data_source_registry()?;
        let parsed: serde_json::Value = serde_json::from_str(&registry_json)
            .map_err(|e| crate::runtime::json_parse_error("data source registry", e))?;
        let sources = parsed
            .get("DATA_SOURCES")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| entry.get("DSRC_CODE").and_then(|v| v.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(sources)
    }

    /// Returns `true` if the given data source is registered in this config.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::SzConfigManager;
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let cm = SzConfigManagerGrpc::new(factory.channel());
    /// let config = cm.create_config_from_template().unwrap();
    /// let def = config.export_config().unwrap();
    /// let config = SzConfigGrpc::new(factory.channel(), def);
    /// assert!(config.contains_data_source("TEST").unwrap_or(false));
    /// ```
    pub fn contains_data_source(&self, name: &str) -> Result<bool, SzError> {
        Ok(self.get_data_sources()?.iter().any(|s| s == name))
    }

    /// Registers multiple data sources, ignoring "already exists" errors.
    ///
    /// Returns the number of data sources that were newly added (i.e., not
    /// already present). This is useful for idempotent setup code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::SzConfigManager;
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let cm = SzConfigManagerGrpc::new(factory.channel());
    /// let config = cm.create_config_from_template().unwrap();
    /// let def = config.export_config().unwrap();
    /// let mut config = SzConfigGrpc::new(factory.channel(), def);
    /// let added = config.add_data_sources(&["CUSTOMERS", "WATCHLIST"]).unwrap();
    /// println!("{added} new data sources registered");
    /// ```
    pub fn add_data_sources(&mut self, sources: &[&str]) -> Result<usize, SzError> {
        use sz_sdk::SzConfig;
        let existing: std::collections::HashSet<String> =
            self.get_data_sources()?.into_iter().collect();
        let mut added = 0usize;
        for &name in sources {
            if !existing.contains(name) {
                self.register_data_source(name)?;
                added += 1;
            }
        }
        Ok(added)
    }

    /// Returns the number of registered data sources.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::SzConfigManager;
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let cm = SzConfigManagerGrpc::new(factory.channel());
    /// let config = cm.create_config_from_template().unwrap();
    /// let def = config.export_config().unwrap();
    /// let config = SzConfigGrpc::new(factory.channel(), def);
    /// println!("{} data sources", config.data_source_count().unwrap());
    /// ```
    pub fn data_source_count(&self) -> Result<usize, SzError> {
        Ok(self.get_data_sources()?.len())
    }
}

/// Differences between two config data source registries.
///
/// Returned by [`SzConfigGrpc::diff_data_sources`]. Contains the lists of
/// data sources that were added or removed relative to another config.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataSourceDiff {
    /// Data sources present in `self` but not in `other`.
    pub added: Vec<String>,
    /// Data sources present in `other` but not in `self`.
    pub removed: Vec<String>,
}

impl DataSourceDiff {
    /// Returns `true` if there are no differences.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

impl std::fmt::Display for DataSourceDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("no differences");
        }
        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("added: {}", self.added.join(", ")));
        }
        if !self.removed.is_empty() {
            parts.push(format!("removed: {}", self.removed.join(", ")));
        }
        write!(f, "{}", parts.join("; "))
    }
}

impl SzConfigGrpc {
    /// Compares data sources between this config and another.
    ///
    /// Returns a [`DataSourceDiff`] showing which data sources were added
    /// (present in `self` but not `other`) and removed (present in `other`
    /// but not `self`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk::{SzConfig, SzConfigManager};
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let cm = SzConfigManagerGrpc::new(factory.channel());
    /// let config_a = cm.create_config_from_template().unwrap();
    /// let def_a = config_a.export_config().unwrap();
    /// let mut config_b = cm.create_config_from_string(&def_a).unwrap();
    /// config_b.register_data_source("NEW_SOURCE").unwrap();
    /// let def_b = config_b.export_config().unwrap();
    ///
    /// use sz_sdk_rust_grpc::SzConfigGrpc;
    /// let cfg_a = SzConfigGrpc::new(factory.channel(), def_a);
    /// let cfg_b = SzConfigGrpc::new(factory.channel(), def_b);
    /// let diff = cfg_b.diff_data_sources(&cfg_a).unwrap();
    /// assert!(diff.added.contains(&"NEW_SOURCE".to_string()));
    /// ```
    pub fn diff_data_sources(&self, other: &SzConfigGrpc) -> Result<DataSourceDiff, SzError> {
        let self_sources: std::collections::HashSet<String> =
            self.get_data_sources()?.into_iter().collect();
        let other_sources: std::collections::HashSet<String> =
            other.get_data_sources()?.into_iter().collect();

        let mut added: Vec<String> = self_sources.difference(&other_sources).cloned().collect();
        let mut removed: Vec<String> = other_sources.difference(&self_sources).cloned().collect();
        added.sort();
        removed.sort();

        Ok(DataSourceDiff { added, removed })
    }
}

impl std::fmt::Display for SzConfigGrpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SzConfigGrpc")
    }
}

impl sz_sdk::SzConfig for SzConfigGrpc {
    /// Returns the current config definition as a JSON string.
    ///
    /// This is a local operation — no RPC is performed.
    fn export_config(&self) -> Result<String, SzError> {
        Ok(self.config_definition.clone())
    }

    /// Returns the data source registry as a JSON string.
    ///
    /// Makes a gRPC call with the current config definition.
    fn get_data_source_registry(&self) -> Result<String, SzError> {
        let config_definition = self.config_definition.clone();
        Ok(grpc_call!(
            self,
            get_data_source_registry(GetDataSourceRegistryRequest { config_definition })
        )?
        .into_inner()
        .result)
    }

    /// Registers a new data source in the config.
    ///
    /// The internal config definition is only updated on success.
    /// If the RPC fails, the previous config is preserved.
    ///
    /// Note: the config definition is cloned for the RPC request because
    /// tonic takes ownership. On failure, the original is retained.
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
            .map_err(|s| grpc_to_sz_error(s, "register_data_source"))?;
        let inner = response.into_inner();
        self.config_definition = inner.config_definition;
        Ok(inner.result)
    }

    /// Removes a data source from the config.
    ///
    /// The internal config definition is only updated on success.
    /// If the RPC fails, the previous config is preserved.
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
            .map_err(|s| grpc_to_sz_error(s, "unregister_data_source"))?;
        self.config_definition = response.into_inner().config_definition;
        Ok(())
    }
}
