use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Returns a reference to the shared tokio runtime.
///
/// The runtime is created lazily on first access and reused for all
/// subsequent calls. This allows synchronous trait methods to call
/// async gRPC operations via `runtime().block_on(...)`.
pub(crate) fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime")
    })
}
