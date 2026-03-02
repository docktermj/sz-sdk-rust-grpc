//! Rust gRPC implementation of the Senzing SDK traits.
//!
//! This crate provides gRPC-based implementations of the traits defined in
//! [`sz_sdk`]. It communicates with a remote Senzing gRPC server
//! (e.g., `senzing/serve-grpc`) over the network.
//!
//! # Usage
//!
//! Use [`SzAbstractFactoryGrpc`] to create instances of the various SDK
//! components. All components share a single gRPC channel (connection).
//!
//! # Async compatibility
//!
//! All trait methods in this crate are **synchronous**. Internally they use
//! [`tokio::runtime::Runtime::block_on`] on a shared background runtime to
//! execute async gRPC calls. This means:
//!
//! - **Do not call these methods from within a tokio async context** (e.g.,
//!   inside a `#[tokio::main]` function or a spawned task). Doing so will
//!   panic with *"Cannot block the current thread from within a runtime"*
//!   because `block_on` cannot be nested.
//! - If you need to use this crate from async code, run the SDK calls on a
//!   blocking thread via [`tokio::task::spawn_blocking`].
//!
//! # Connection behavior
//!
//! The underlying [`tonic::transport::Channel`] manages a pool of HTTP/2
//! connections and will lazily reconnect if the server restarts or a
//! connection drops. This means:
//!
//! - Construction via [`SzAbstractFactoryGrpc::new_from_url`] eagerly
//!   establishes a connection, but subsequent RPCs may transparently
//!   reconnect if that connection is lost.
//! - Use [`SzAbstractFactoryGrpc::check_health`] after construction to
//!   verify that the server is reachable before performing real work.
//! - The `connect_timeout` in [`GrpcConnectionConfig`] applies to the
//!   initial connection and each reconnection attempt. The `rpc_timeout`
//!   applies to each individual RPC call.
//!
//! # Performance
//!
//! - **Channel reuse**: All service clients created from a single factory share
//!   one [`tonic::transport::Channel`], which multiplexes RPCs over a pool of
//!   HTTP/2 connections. Creating additional clients (via `create_engine()`,
//!   `create_product()`, etc.) is cheap — it just clones the channel handle.
//! - **Cloning is cheap**: All gRPC types (`SzEngineGrpc`, `SzProductGrpc`, etc.)
//!   can be cloned in O(1). The clone shares the same underlying connection pool.
//! - **Serialization overhead**: Each RPC serializes a protobuf request and
//!   deserializes the response. For large payloads (e.g., entity exports),
//!   prefer the streaming methods on [`SzEngineGrpc`] to avoid buffering the
//!   entire response in memory.
//! - **Runtime overhead**: A shared tokio runtime (created once via `OnceLock`)
//!   drives all async I/O. The runtime uses a multi-threaded scheduler. Sync
//!   trait methods call `block_on()`, which parks the calling thread while
//!   waiting for the RPC — consider using multiple threads for concurrent RPCs.
//!
//! # Feature flags
//!
//! | Feature    | Default | Description |
//! |------------|---------|-------------|
//! | `serde`    | off     | Adds `Serialize`/`Deserialize` to [`GrpcConnectionConfig`] and [`GrpcUrl`] |
//! | `tracing`  | off     | Emits [`tracing`](https://docs.rs/tracing) spans for every gRPC call |
//!
//! Enable via Cargo:
//!
//! ```toml
//! [dependencies]
//! sz-sdk-rust-grpc = { version = "0.1", features = ["tracing"] }
//! ```
//!
//! When enabled, each RPC method emits an `INFO`-level span named after the
//! gRPC method (e.g., `add_record`, `get_version`). On completion, the span
//! records `otel.status_code` ("OK" or "ERROR"). Failed RPCs also emit an
//! `ERROR`-level event with `error.message`. Pair with a subscriber like
//! [`tracing-subscriber`](https://docs.rs/tracing-subscriber) to see timing
//! and context in logs. The overhead is negligible when no subscriber is
//! installed.
//!
//! # Custom middleware / interceptors
//!
//! To inject custom headers (e.g., authentication tokens) or apply other
//! middleware, build a [`tonic::transport::Channel`] yourself and pass it
//! to [`SzAbstractFactoryGrpc::new_from_channel`]:
//!
//! ```no_run
//! use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let channel = tonic::transport::Channel::from_static("http://localhost:8261")
//!     .connect()
//!     .await?;
//! let factory = SzAbstractFactoryGrpc::new_from_channel(channel);
//! # Ok(())
//! # }
//! ```
//!
//! This gives you full control over the channel configuration, including
//! TLS settings, load balancing, and tower middleware layers.
//!
//! # Thread safety
//!
//! All gRPC client types (`SzEngineGrpc`, `SzConfigGrpc`, `SzConfigManagerGrpc`,
//! `SzDiagnosticGrpc`, `SzProductGrpc`, `SzAbstractFactoryGrpc`) implement
//! `Clone`, `Send`, and `Sync`. They can be safely shared across threads
//! (e.g., via `Arc`) or cloned cheaply — the underlying `tonic::transport::Channel`
//! is a handle to a shared HTTP/2 connection pool.
//!
//! ```no_run
//! use sz_sdk_rust_grpc::SzAbstractFactoryGrpc;
//! use sz_sdk::{SzAbstractFactory, SzProduct};
//!
//! let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
//!
//! // From async code, use spawn_blocking to call sync SDK methods:
//! # async fn example(factory: SzAbstractFactoryGrpc) {
//! let result = tokio::task::spawn_blocking(move || {
//!     let product = factory.create_product().unwrap();
//!     product.get_version()
//! })
//! .await
//! .unwrap();
//! # }
//! ```
//!
//! # Error recovery
//!
//! When an RPC fails, the returned [`sz_sdk::SzError`] variant indicates
//! whether retrying is appropriate:
//!
//! | Variant | Retryable? | Notes |
//! |---------|-----------|-------|
//! | `DatabaseTransient` | **Yes** | Deadlock or lock timeout — retry with backoff |
//! | `DatabaseConnectionLost` | **Yes** | Connection dropped — tonic will reconnect automatically, retry the RPC |
//! | `RetryTimeoutExceeded` | **Yes** | Optimistic locking contention — retry with backoff |
//! | `Database` | Maybe | Persistent DB error — retry only if the cause is transient (e.g., maintenance) |
//! | `BadInput` | **No** | Invalid parameters — fix the input before retrying |
//! | `NotFound` | **No** | Entity/record does not exist |
//! | `UnknownDataSource` | **No** | Data source not registered in config |
//! | `NotInitialized` | **No** | Engine not initialized or factory closed |
//! | `Configuration` | **No** | Config is invalid or corrupt |
//! | `License` | **No** | License expired or record limit exceeded |
//!
//! **Idempotency**: `add_record` with the same `(data_source, record_id)` is
//! idempotent — retrying a failed add is safe. `delete_record` is also safe
//! to retry. Read-only methods (`get_entity_by_*`, `search_by_attributes`,
//! exports, etc.) are inherently safe to retry.
//!
//! **Recommended retry pattern**: use exponential backoff with jitter for
//! retryable errors. See `examples/error_handling.rs` for a working example.
//!
//! # Shutdown
//!
//! Call [`close()`](sz_sdk::SzAbstractFactory::close) on the factory when
//! you are done. After `close()`:
//!
//! - New `create_*()` calls return `SzError::NotInitialized`.
//! - Already-created service clients (engines, products, etc.) remain usable
//!   — they hold their own channel clone and are not affected by `close()`.
//! - `close()` is idempotent and can be called multiple times safely.
//! - `close()` does **not** cancel in-flight RPCs. If you need to wait for
//!   pending work, finish those calls before closing.
//! - Cloned factories are independent — closing one does not close others.
//!
//! **Recommended shutdown sequence:**
//!
//! 1. Stop submitting new work.
//! 2. Wait for in-flight RPCs to complete.
//! 3. Call `factory.close()`.
//! 4. Drop all service clients and the factory.

/// Clones the gRPC client, executes an async RPC via the shared runtime,
/// and maps any gRPC error to `SzError`.
///
/// When the `tracing` feature is enabled, each call emits a `tracing` span
/// named after the RPC method. The span records `otel.status_code` ("OK" or
/// "ERROR") and, on failure, `error.message` with the gRPC status details.
macro_rules! grpc_call {
    ($self:expr, $method:ident($request:expr)) => {{
        #[cfg(feature = "tracing")]
        let _span = tracing::info_span!(stringify!($method)).entered();
        let mut client = $self.client.clone();
        let result = $crate::runtime::runtime()
            .block_on(async { client.$method($request).await });
        #[cfg(feature = "tracing")]
        match &result {
            Ok(_) => {
                tracing::Span::current().record("otel.status_code", "OK");
            }
            Err(status) => {
                tracing::Span::current().record("otel.status_code", "ERROR");
                tracing::event!(tracing::Level::ERROR, error.message = %status.message());
            }
        }
        result.map_err(|status| $crate::runtime::grpc_to_sz_error(status, stringify!($method)))
    }};
}

pub mod szabstractfactory;
pub mod szconfig;
pub mod szconfigmanager;
pub mod szdiagnostic;
pub mod szengine;
pub mod szproduct;

mod runtime;
mod szerrortypes;

pub use szabstractfactory::{
    GrpcConnectionConfig, GrpcUrl, SzAbstractFactoryBuilder, SzAbstractFactoryGrpc,
};
pub use szconfig::DataSourceDiff;
pub use szconfig::SzConfigGrpc;
pub use szconfigmanager::SzConfigManagerGrpc;
pub use szdiagnostic::PerformanceReport;
pub use szdiagnostic::RepositoryInfo;
pub use szdiagnostic::SzDiagnosticGrpc;
pub use szengine::BatchResult;
pub use szengine::EngineStats;
pub use szengine::EntityData;
pub use szengine::RedoSummary;
pub use szengine::SearchEntity;
pub use szengine::SearchResults;
pub use szengine::StreamExportChunk;
pub use szengine::StreamingExportIterator;
pub use szengine::SzEngineGrpc;
pub use szengine::SZ_CSV_DEFAULT_COLUMNS;
pub use szerrortypes::is_retryable;
pub use szerrortypes::with_retry;
pub use szproduct::LicenseInfo;
pub use szproduct::SzProductGrpc;
pub use szproduct::VersionInfo;

#[cfg(test)]
mod assert_traits {
    use static_assertions::assert_impl_all;

    assert_impl_all!(crate::SzAbstractFactoryGrpc: Clone, Send, Sync, std::fmt::Debug);
    assert_impl_all!(crate::SzConfigGrpc: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display);
    assert_impl_all!(crate::SzConfigManagerGrpc: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display);
    assert_impl_all!(crate::SzDiagnosticGrpc: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display);
    assert_impl_all!(crate::SzEngineGrpc: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display);
    assert_impl_all!(crate::SzProductGrpc: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display);
    assert_impl_all!(crate::GrpcConnectionConfig: Clone, Copy, Send, Sync, std::fmt::Debug, std::hash::Hash);
    assert_impl_all!(crate::GrpcUrl: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord);
    assert_impl_all!(crate::SzAbstractFactoryBuilder: Clone, Send, Sync, std::fmt::Debug, Default);
    assert_impl_all!(crate::VersionInfo: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::LicenseInfo: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::BatchResult: std::fmt::Debug, std::fmt::Display, Default);
    assert_impl_all!(crate::RepositoryInfo: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::PerformanceReport: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::EngineStats: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::EntityData: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::DataSourceDiff: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::SearchResults: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::SearchEntity: Clone, Send, Sync, std::fmt::Debug, std::fmt::Display, Ord, std::hash::Hash);
    assert_impl_all!(crate::RedoSummary: Send, Sync, std::fmt::Debug, std::fmt::Display);
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const GRPC_URL: &str = "http://localhost:8261";

    pub fn is_valid_json(s: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(s).is_ok()
    }

    pub fn connect_channel() -> tonic::transport::Channel {
        let config = crate::szabstractfactory::GrpcConnectionConfig::default();
        crate::runtime::runtime()
            .block_on(async {
                tonic::transport::Channel::from_shared(GRPC_URL.to_string())
                    .unwrap()
                    .connect_timeout(config.connect_timeout)
                    .timeout(config.rpc_timeout)
                    .connect()
                    .await
            })
            .expect("failed to connect to gRPC server — is 'make setup' running?")
    }

    /// Runs exactly once before any test via the `ctor` crate.
    /// Registers CUSTOMERS, REFERENCE, and WATCHLIST data sources.
    #[ctor::ctor]
    fn setup_test_environment() {
        use sz_sdk::{SzAbstractFactory, SzConfigManager};
        let channel = connect_channel();
        let mut config_manager = crate::szconfigmanager::SzConfigManagerGrpc::new(channel.clone());
        let mut config = config_manager
            .create_config_from_template()
            .expect("failed to create config from template");
        for ds in &["CUSTOMERS", "INTEGRATION_TEST", "REFERENCE", "WATCHLIST"] {
            let _ = config.register_data_source(ds);
        }
        let config_definition = config.export_config().expect("failed to export config");
        let config_id = config_manager
            .register_config(&config_definition, "rust test setup")
            .expect("failed to register config");
        config_manager
            .set_default_config_id(config_id)
            .expect("failed to set default config id");

        // Reinitialize the gRPC server's engine to pick up the new config.
        let mut factory =
            crate::szabstractfactory::SzAbstractFactoryGrpc::new_from_channel(channel);
        factory
            .reinitialize(config_id)
            .expect("failed to reinitialize with new config");
    }
}

/// Generated gRPC client code for the SzConfig service.
pub mod pb_szconfig {
    tonic::include_proto!("szconfig");
}

/// Generated gRPC client code for the SzConfigManager service.
pub mod pb_szconfigmanager {
    tonic::include_proto!("szconfigmanager");
}

/// Generated gRPC client code for the SzDiagnostic service.
pub mod pb_szdiagnostic {
    tonic::include_proto!("szdiagnostic");
}

/// Generated gRPC client code for the SzEngine service.
pub mod pb_szengine {
    tonic::include_proto!("szengine");
}

/// Generated gRPC client code for the SzProduct service.
pub mod pb_szproduct {
    tonic::include_proto!("szproduct");
}
