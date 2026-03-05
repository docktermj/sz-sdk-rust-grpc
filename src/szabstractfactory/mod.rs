#[cfg(test)]
mod tests;

use std::time::Duration;

use sz_sdk::{SzConfigManager, SzDiagnostic, SzEngine, SzError, SzProduct};
use tonic::transport::{Channel, ClientTlsConfig};

use crate::pb_szdiagnostic::sz_diagnostic_client::SzDiagnosticClient;
use crate::pb_szdiagnostic::ReinitializeRequest;
use crate::pb_szengine::sz_engine_client::SzEngineClient;
use crate::runtime::{grpc_to_sz_error, runtime, try_runtime};
use crate::szconfigmanager::SzConfigManagerGrpc;
use crate::szdiagnostic::SzDiagnosticGrpc;
use crate::szengine::SzEngineGrpc;
use crate::szproduct::SzProductGrpc;

/// Configuration for gRPC connections created by [`SzAbstractFactoryGrpc`].
///
/// Use [`Default::default()`] for balanced settings, or the preset constructors
/// [`for_development`](Self::for_development) and
/// [`for_production`](Self::for_production) for common scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GrpcConnectionConfig {
    /// Timeout for establishing the initial TCP connection. Default: 10 seconds.
    pub connect_timeout: Duration,
    /// Per-RPC timeout applied to every gRPC call on the channel. Default: 120 seconds.
    pub rpc_timeout: Duration,
    /// Interval between HTTP/2 keepalive pings. `None` disables keepalive.
    /// Default: `None`.
    pub keepalive_interval: Option<Duration>,
    /// How long to wait for a keepalive response before considering the
    /// connection dead. Only meaningful when `keepalive_interval` is `Some`.
    /// Default: 20 seconds.
    pub keepalive_timeout: Duration,
}

impl std::fmt::Display for GrpcConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GrpcConnectionConfig {{ connect_timeout: {:?}, rpc_timeout: {:?}, \
             keepalive_interval: {:?}, keepalive_timeout: {:?} }}",
            self.connect_timeout, self.rpc_timeout, self.keepalive_interval, self.keepalive_timeout
        )
    }
}

impl Default for GrpcConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            rpc_timeout: Duration::from_secs(120),
            keepalive_interval: None,
            keepalive_timeout: Duration::from_secs(20),
        }
    }
}

impl GrpcConnectionConfig {
    /// Preset for local development: short timeouts, no keepalive.
    ///
    /// - `connect_timeout`: 3 seconds
    /// - `rpc_timeout`: 30 seconds
    /// - `keepalive_interval`: None
    #[must_use]
    pub fn for_development() -> Self {
        Self {
            connect_timeout: Duration::from_secs(3),
            rpc_timeout: Duration::from_secs(30),
            keepalive_interval: None,
            keepalive_timeout: Duration::from_secs(20),
        }
    }

    /// Preset for production: generous timeouts with keepalive enabled.
    ///
    /// - `connect_timeout`: 30 seconds
    /// - `rpc_timeout`: 300 seconds (5 minutes)
    /// - `keepalive_interval`: 60 seconds
    /// - `keepalive_timeout`: 20 seconds
    #[must_use]
    pub fn for_production() -> Self {
        Self {
            connect_timeout: Duration::from_secs(30),
            rpc_timeout: Duration::from_secs(300),
            keepalive_interval: Some(Duration::from_secs(60)),
            keepalive_timeout: Duration::from_secs(20),
        }
    }

    /// Validates the configuration, returning an error for nonsensical values.
    ///
    /// Checked conditions:
    /// - `connect_timeout` must be > 0
    /// - `rpc_timeout` must be > 0
    /// - `rpc_timeout` must be >= `connect_timeout` (an RPC that times out
    ///   before the connection is established would always fail)
    /// - `keepalive_timeout` must be > 0 when keepalive is enabled
    /// - `keepalive_interval` must be > `keepalive_timeout` (otherwise every
    ///   ping would time out before the next one fires)
    pub fn validate(&self) -> Result<(), SzError> {
        if self.connect_timeout.is_zero() {
            return Err(SzError::bad_input(
                "connect_timeout must be greater than zero",
            ));
        }
        if self.rpc_timeout.is_zero() {
            return Err(SzError::bad_input(
                "rpc_timeout must be greater than zero",
            ));
        }
        if self.rpc_timeout < self.connect_timeout {
            return Err(SzError::bad_input(format!(
                "rpc_timeout ({:?}) must be >= connect_timeout ({:?})",
                self.rpc_timeout, self.connect_timeout
            )));
        }
        if let Some(interval) = self.keepalive_interval {
            if self.keepalive_timeout.is_zero() {
                return Err(SzError::bad_input(
                    "keepalive_timeout must be greater than zero when keepalive is enabled",
                ));
            }
            if interval <= self.keepalive_timeout {
                return Err(SzError::bad_input(format!(
                    "keepalive_interval ({interval:?}) must be greater than \
                     keepalive_timeout ({:?})",
                    self.keepalive_timeout
                )));
            }
        }
        Ok(())
    }
}

/// A validated gRPC URL.
///
/// Ensures the URL uses `http://` or `https://` scheme. Use [`TryFrom`] or
/// [`parse`](str::parse) to construct.
///
/// # Example
///
/// ```
/// use sz_sdk_rust_grpc::GrpcUrl;
///
/// let url: GrpcUrl = "http://localhost:8261".parse().unwrap();
/// assert_eq!(url.as_str(), "http://localhost:8261");
///
/// let bad = "ftp://localhost:8261".parse::<GrpcUrl>();
/// assert!(bad.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GrpcUrl(String);

impl GrpcUrl {
    /// Returns the URL as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GrpcUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<&str> for GrpcUrl {
    type Error = SzError;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(SzError::bad_input(format!(
                "gRPC URL must use http:// or https:// scheme (got: {url})"
            )));
        }
        Ok(Self(url.to_string()))
    }
}

impl TryFrom<String> for GrpcUrl {
    type Error = SzError;

    fn try_from(url: String) -> Result<Self, Self::Error> {
        GrpcUrl::try_from(url.as_str())
    }
}

impl std::str::FromStr for GrpcUrl {
    type Err = SzError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GrpcUrl::try_from(s)
    }
}

impl std::ops::Deref for GrpcUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GrpcUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<GrpcUrl> for String {
    fn from(url: GrpcUrl) -> Self {
        url.0
    }
}

/// gRPC implementation of [`sz_sdk::SzAbstractFactory`].
///
/// Holds a shared [`tonic::transport::Channel`] and creates gRPC clients
/// from it on demand. The channel is cheaply cloneable (it represents a
/// pool of HTTP/2 connections).
///
/// # Resource cleanup
///
/// No explicit `Drop` is needed. The underlying [`tonic::transport::Channel`]
/// manages HTTP/2 connection cleanup automatically when the last reference
/// is dropped. Calling [`close()`](sz_sdk::SzAbstractFactory::close) is
/// optional — it prevents new `create_*()` calls but does not close the
/// underlying connection (already-created clients continue working).
///
/// # Async compatibility
///
/// All methods on this type and the components it creates are synchronous
/// and internally call [`tokio::runtime::Runtime::block_on`]. **Do not call
/// them from within an async context** — this will panic. Use
/// [`tokio::task::spawn_blocking`] to bridge from async code.
#[derive(Debug, Clone)]
pub struct SzAbstractFactoryGrpc {
    channel: Channel,
    closed: bool,
}

impl SzAbstractFactoryGrpc {
    /// Creates a new factory by connecting to the given gRPC URL
    /// using default timeout settings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// ```
    pub fn new_from_url(grpc_url: &str) -> Result<Self, SzError> {
        Self::new_from_url_with_config(grpc_url, GrpcConnectionConfig::default())
    }

    /// Creates a new factory by connecting to the given gRPC URL
    /// with custom timeout configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, GrpcConnectionConfig};
    ///
    /// let config = GrpcConnectionConfig::for_development();
    /// let factory = SzAbstractFactoryGrpc::new_from_url_with_config(
    ///     "http://localhost:8261",
    ///     config,
    /// ).unwrap();
    /// ```
    pub fn new_from_url_with_config(
        grpc_url: &str,
        config: GrpcConnectionConfig,
    ) -> Result<Self, SzError> {
        let validated = GrpcUrl::try_from(grpc_url)?;
        Self::connect(validated, config, None)
    }

    /// Creates a new factory from a pre-validated [`GrpcUrl`].
    pub fn new_from_grpc_url(url: &GrpcUrl) -> Result<Self, SzError> {
        Self::connect(url.clone(), GrpcConnectionConfig::default(), None)
    }

    /// Creates a new factory from a pre-validated [`GrpcUrl`] with custom config.
    pub fn new_from_grpc_url_with_config(
        url: &GrpcUrl,
        config: GrpcConnectionConfig,
    ) -> Result<Self, SzError> {
        Self::connect(url.clone(), config, None)
    }

    fn connect(
        url: GrpcUrl,
        config: GrpcConnectionConfig,
        tls: Option<ClientTlsConfig>,
    ) -> Result<Self, SzError> {
        config.validate()?;
        let channel = try_runtime()?.block_on(async {
            let mut endpoint = Channel::from_shared(url.0)
                .map_err(|e| SzError::general(format!("invalid gRPC URL: {e}")))?
                .connect_timeout(config.connect_timeout)
                .timeout(config.rpc_timeout);
            if let Some(interval) = config.keepalive_interval {
                endpoint = endpoint
                    .keep_alive_while_idle(true)
                    .http2_keep_alive_interval(interval)
                    .keep_alive_timeout(config.keepalive_timeout);
            }
            if let Some(tls_config) = tls {
                endpoint = endpoint
                    .tls_config(tls_config)
                    .map_err(|e| SzError::general(format!("invalid TLS configuration: {e}")))?;
            }
            endpoint
                .connect()
                .await
                .map_err(|e| SzError::general(format!("failed to connect to gRPC server: {e}")))
        })?;
        Ok(Self {
            channel,
            closed: false,
        })
    }

    /// Creates a new factory from an existing channel.
    #[must_use]
    pub fn new_from_channel(channel: Channel) -> Self {
        Self {
            channel,
            closed: false,
        }
    }

    /// Returns a clone of the underlying [`tonic::transport::Channel`].
    ///
    /// Use this to construct concrete service clients (e.g., [`SzEngineGrpc`])
    /// directly when you need access to methods not on the trait, such as
    /// streaming exports.
    ///
    /// Cloning a channel is cheap — it shares the same HTTP/2 connection pool.
    #[must_use]
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// Returns a builder for fluent construction.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
    ///
    /// let factory = SzAbstractFactoryGrpc::builder()
    ///     .url("http://localhost:8261")
    ///     .connect_timeout(Duration::from_secs(5))
    ///     .rpc_timeout(Duration::from_secs(60))
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn builder() -> SzAbstractFactoryBuilder {
        SzAbstractFactoryBuilder::default()
    }

    /// Verifies that the gRPC server is reachable by making a lightweight
    /// `get_version` RPC call.
    ///
    /// Call this after construction to confirm the server is up before
    /// performing real work.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// factory.check_health().expect("gRPC server is not reachable");
    /// ```
    pub fn check_health(&self) -> Result<(), SzError> {
        self.check_closed()?;
        let mut client =
            crate::pb_szproduct::sz_product_client::SzProductClient::new(self.channel.clone());
        runtime()
            .block_on(async {
                client
                    .get_version(crate::pb_szproduct::GetVersionRequest {})
                    .await
            })
            .map_err(|s| grpc_to_sz_error(s, "check_health"))?;
        Ok(())
    }

    /// Returns `true` if [`close`](sz_sdk::SzAbstractFactory::close) has been called.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

impl From<Channel> for SzAbstractFactoryGrpc {
    fn from(channel: Channel) -> Self {
        Self::new_from_channel(channel)
    }
}

impl SzAbstractFactoryGrpc {
    fn check_closed(&self) -> Result<(), SzError> {
        if self.closed {
            return Err(SzError::not_initialized(
                "SzAbstractFactory has been closed",
            ));
        }
        Ok(())
    }
}

/// Builder for [`SzAbstractFactoryGrpc`].
///
/// Created via [`SzAbstractFactoryGrpc::builder()`]. The only required
/// field is the gRPC URL (set via [`url`](Self::url)).
#[derive(Debug, Clone, Default)]
pub struct SzAbstractFactoryBuilder {
    url: Option<Result<GrpcUrl, String>>,
    config: GrpcConnectionConfig,
    tls: Option<ClientTlsConfig>,
}

impl SzAbstractFactoryBuilder {
    /// Creates a new builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the gRPC server URL (required). Must use `http://` or `https://`.
    ///
    /// The URL is validated immediately. If the scheme is invalid, the error
    /// is deferred until [`build`](Self::build) is called.
    #[must_use]
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(GrpcUrl::try_from(url).map_err(|e| format!("{e}")));
        self
    }

    /// Reads the gRPC server URL from the environment variable `var_name`.
    ///
    /// If the variable is not set, falls back to `fallback`. This eliminates
    /// the common boilerplate of `std::env::var(...).unwrap_or_else(...)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
    ///
    /// let factory = SzAbstractFactoryGrpc::builder()
    ///     .url_from_env("SENZING_GRPC_URL", "http://localhost:8261")
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn url_from_env(self, var_name: &str, fallback: &str) -> Self {
        let value = std::env::var(var_name).unwrap_or_else(|_| fallback.to_string());
        self.url(&value)
    }

    /// Sets the TCP connection timeout. Default: 10 seconds.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Sets the per-RPC timeout. Default: 120 seconds.
    #[must_use]
    pub fn rpc_timeout(mut self, timeout: Duration) -> Self {
        self.config.rpc_timeout = timeout;
        self
    }

    /// Enables HTTP/2 keepalive pings at the given interval.
    #[must_use]
    pub fn keepalive_interval(mut self, interval: Duration) -> Self {
        self.config.keepalive_interval = Some(interval);
        self
    }

    /// Sets the keepalive response timeout. Default: 20 seconds.
    #[must_use]
    pub fn keepalive_timeout(mut self, timeout: Duration) -> Self {
        self.config.keepalive_timeout = timeout;
        self
    }

    /// Applies all settings from a [`GrpcConnectionConfig`] at once.
    ///
    /// This is a convenience for applying a preset or pre-built configuration
    /// instead of setting each timeout individually.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, GrpcConnectionConfig};
    ///
    /// let factory = SzAbstractFactoryGrpc::builder()
    ///     .url("http://localhost:8261")
    ///     .from_config(GrpcConnectionConfig::for_production())
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn from_config(mut self, config: GrpcConnectionConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables TLS for the connection.
    ///
    /// Use [`tonic::transport::ClientTlsConfig`] to configure server CA
    /// certificates, client identity (for mutual TLS), and domain name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tonic::transport::ClientTlsConfig;
    /// use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
    ///
    /// let tls = ClientTlsConfig::new().domain_name("senzing.example.com");
    /// let factory = SzAbstractFactoryGrpc::builder()
    ///     .url("https://senzing.example.com:8261")
    ///     .tls_config(tls)
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn tls_config(mut self, tls: ClientTlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Builds the factory by connecting to the gRPC server.
    ///
    /// # Errors
    ///
    /// Returns an `SzError` with kind `BadInput` if no URL was set, and kind
    /// `General` if the connection fails.
    pub fn build(self) -> Result<SzAbstractFactoryGrpc, SzError> {
        let validated = self
            .url
            .ok_or_else(|| SzError::bad_input("gRPC URL is required — call .url() on the builder"))?
            .map_err(SzError::bad_input)?;
        SzAbstractFactoryGrpc::connect(validated, self.config, self.tls)
    }
}

impl sz_sdk::SzAbstractFactory for SzAbstractFactoryGrpc {
    fn close(&mut self) -> Result<(), SzError> {
        self.closed = true;
        Ok(())
    }

    fn create_config_manager(&self) -> Result<Box<dyn SzConfigManager>, SzError> {
        self.check_closed()?;
        Ok(Box::new(SzConfigManagerGrpc::new(self.channel.clone())))
    }

    fn create_diagnostic(&self) -> Result<Box<dyn SzDiagnostic>, SzError> {
        self.check_closed()?;
        Ok(Box::new(SzDiagnosticGrpc::new(self.channel.clone())))
    }

    fn create_engine(&self) -> Result<Box<dyn SzEngine>, SzError> {
        self.check_closed()?;
        Ok(Box::new(SzEngineGrpc::new(self.channel.clone())))
    }

    fn create_product(&self) -> Result<Box<dyn SzProduct>, SzError> {
        self.check_closed()?;
        Ok(Box::new(SzProductGrpc::new(self.channel.clone())))
    }

    fn reinitialize(&mut self, config_id: i64) -> Result<(), SzError> {
        self.check_closed()?;
        let mut diag_client = SzDiagnosticClient::new(self.channel.clone());
        runtime()
            .block_on(async {
                diag_client
                    .reinitialize(ReinitializeRequest { config_id })
                    .await
            })
            .map_err(|s| grpc_to_sz_error(s, "reinitialize"))?;

        let mut engine_client = SzEngineClient::new(self.channel.clone());
        runtime()
            .block_on(async {
                engine_client
                    .reinitialize(crate::pb_szengine::ReinitializeRequest { config_id })
                    .await
            })
            .map_err(|s| grpc_to_sz_error(s, "reinitialize"))?;

        Ok(())
    }
}
