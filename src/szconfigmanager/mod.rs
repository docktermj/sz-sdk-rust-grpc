#[cfg(test)]
mod tests;

use sz_sdk::{SzConfig, SzError};
use tonic::transport::Channel;

use crate::pb_szconfigmanager::sz_config_manager_client::SzConfigManagerClient;
use crate::pb_szconfigmanager::*;
use crate::runtime::{grpc_to_sz_error, runtime};
use crate::szconfig::SzConfigGrpc;

/// gRPC implementation of [`sz_sdk::SzConfigManager`].
///
/// Also holds a reference to the config service channel so it can create
/// `SzConfigGrpc` instances from the `create_config_from_*` methods.
#[derive(Debug, Clone)]
pub struct SzConfigManagerGrpc {
    client: SzConfigManagerClient<Channel>,
    channel: Channel,
}

impl SzConfigManagerGrpc {
    /// Creates a new `SzConfigManagerGrpc` from a gRPC channel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let config_manager = SzConfigManagerGrpc::new(factory.channel());
    /// ```
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzConfigManagerClient::new(channel.clone()),
            channel,
        }
    }
}

impl SzConfigManagerGrpc {
    /// Ensures the given data sources exist in the default config.
    ///
    /// Creates a config from the current template, idempotently registers
    /// each data source (ignoring "already exists" errors), and sets the
    /// result as the new default config. Returns the new config ID.
    ///
    /// This is a convenience wrapper around the common setup pattern of
    /// `create_config_from_template` → `register_data_source` (×N) →
    /// `set_default_config`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let mut cm = SzConfigManagerGrpc::new(factory.channel());
    /// let config_id = cm.ensure_data_sources(
    ///     &["CUSTOMERS", "WATCHLIST"],
    ///     "register app data sources",
    /// ).unwrap();
    /// ```
    pub fn ensure_data_sources(
        &mut self,
        data_sources: &[&str],
        comment: &str,
    ) -> Result<i64, SzError> {
        use sz_sdk::SzConfigManager;

        let mut config = self.create_config_from_template()?;
        for ds in data_sources {
            // Ignore errors from duplicate registration.
            let _ = config.register_data_source(ds);
        }
        let config_definition = config.export_config()?;
        self.set_default_config(&config_definition, comment)
    }
}

impl SzConfigManagerGrpc {
    /// Returns the list of registered config IDs.
    ///
    /// This is a convenience wrapper around
    /// [`get_config_registry()`](sz_sdk::SzConfigManager::get_config_registry)
    /// that parses the JSON response and extracts the config IDs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzConfigManagerGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let config_manager = SzConfigManagerGrpc::new(factory.channel());
    /// let ids = config_manager.get_config_ids().unwrap();
    /// for id in &ids {
    ///     println!("Config ID: {id}");
    /// }
    /// ```
    pub fn get_config_ids(&self) -> Result<Vec<i64>, SzError> {
        use sz_sdk::SzConfigManager;
        let registry_json = self.get_config_registry()?;
        let parsed: serde_json::Value = serde_json::from_str(&registry_json)
            .map_err(|e| crate::runtime::json_parse_error("config registry", e))?;
        let ids = parsed
            .get("CONFIGS")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| entry.get("CONFIG_ID").and_then(|v| v.as_i64()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(ids)
    }
}

impl std::fmt::Display for SzConfigManagerGrpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SzConfigManagerGrpc")
    }
}

impl sz_sdk::SzConfigManager for SzConfigManagerGrpc {
    /// Retrieves a previously registered config by its ID and returns it
    /// as a mutable [`SzConfig`] instance.
    fn create_config_from_config_id(&self, config_id: i64) -> Result<Box<dyn SzConfig>, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_config(GetConfigRequest { config_id }).await })
            .map_err(|s| grpc_to_sz_error(s, "get_config"))?;
        let config_definition = response.into_inner().result;
        Ok(Box::new(SzConfigGrpc::new(
            self.channel.clone(),
            config_definition,
        )))
    }

    /// Creates an [`SzConfig`] from a JSON config definition string.
    ///
    /// No RPC is performed — the string is used directly. Use this to
    /// re-import a config previously obtained via [`SzConfig::export_config`].
    fn create_config_from_string(
        &self,
        config_definition: &str,
    ) -> Result<Box<dyn SzConfig>, SzError> {
        Ok(Box::new(SzConfigGrpc::new(
            self.channel.clone(),
            config_definition.to_string(),
        )))
    }

    /// Creates an [`SzConfig`] from the server's built-in template.
    ///
    /// The template provides a baseline config with no custom data sources.
    fn create_config_from_template(&self) -> Result<Box<dyn SzConfig>, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_template_config(GetTemplateConfigRequest {})
                    .await
            })
            .map_err(|s| grpc_to_sz_error(s, "get_template_config"))?;
        let config_definition = response.into_inner().result;
        Ok(Box::new(SzConfigGrpc::new(
            self.channel.clone(),
            config_definition,
        )))
    }

    /// No-op for gRPC — the server manages its own lifecycle.
    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    /// Returns a JSON listing of all registered configs with their IDs,
    /// comments, and creation timestamps.
    fn get_config_registry(&self) -> Result<String, SzError> {
        Ok(
            grpc_call!(self, get_config_registry(GetConfigRegistryRequest {}))?
                .into_inner()
                .result,
        )
    }

    /// Returns the config ID currently set as the default.
    fn get_default_config_id(&self) -> Result<i64, SzError> {
        Ok(
            grpc_call!(self, get_default_config_id(GetDefaultConfigIdRequest {}))?
                .into_inner()
                .result,
        )
    }

    /// Stores a config definition in the repository and returns its new ID.
    fn register_config(
        &mut self,
        config_definition: &str,
        config_comment: &str,
    ) -> Result<i64, SzError> {
        Ok(grpc_call!(
            self,
            register_config(RegisterConfigRequest {
                config_definition: config_definition.to_string(),
                config_comment: config_comment.to_string(),
            })
        )?
        .into_inner()
        .result)
    }

    /// Atomically replaces the default config ID using compare-and-swap.
    ///
    /// Returns [`SzError::ReplaceConflict`] if `current_default_config_id`
    /// no longer matches the actual default (concurrent modification).
    fn replace_default_config_id(
        &mut self,
        current_default_config_id: i64,
        new_default_config_id: i64,
    ) -> Result<(), SzError> {
        grpc_call!(
            self,
            replace_default_config_id(ReplaceDefaultConfigIdRequest {
                current_default_config_id,
                new_default_config_id,
            })
        )?;
        Ok(())
    }

    /// Registers a config and sets it as the default in one operation.
    /// Returns the new config ID.
    fn set_default_config(
        &mut self,
        config_definition: &str,
        config_comment: &str,
    ) -> Result<i64, SzError> {
        Ok(grpc_call!(
            self,
            set_default_config(SetDefaultConfigRequest {
                config_definition: config_definition.to_string(),
                config_comment: config_comment.to_string(),
            })
        )?
        .into_inner()
        .result)
    }

    /// Sets the default config ID unconditionally (no compare-and-swap).
    fn set_default_config_id(&mut self, config_id: i64) -> Result<(), SzError> {
        grpc_call!(
            self,
            set_default_config_id(SetDefaultConfigIdRequest { config_id })
        )?;
        Ok(())
    }
}
