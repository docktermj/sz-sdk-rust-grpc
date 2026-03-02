use std::sync::OnceLock;
use sz_sdk::SzError;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

/// Initializes (if needed) and returns a reference to the shared tokio runtime.
///
/// Returns `Err` if the runtime could not be created (e.g., OS thread limits).
/// The error is cached — once initialization fails, all subsequent calls
/// return the same error without retrying.
///
/// # Panics
///
/// Calling `runtime.block_on(...)` from within an existing tokio runtime
/// (e.g., inside `#[tokio::main]` or a spawned task) will panic. Use
/// [`tokio::task::spawn_blocking`] to bridge from async to sync code.
pub(crate) fn try_runtime() -> Result<&'static Runtime, SzError> {
    let result = RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("failed to create tokio runtime: {e}"))
    });
    match result {
        Ok(rt) => Ok(rt),
        Err(msg) => Err(SzError::General {
            code: 0,
            message: msg.clone(),
        }),
    }
}

/// Returns a reference to the shared tokio runtime.
///
/// Convenience wrapper around [`try_runtime`] that panics on failure.
/// Used by the `grpc_call!` macro and other internal code paths where
/// runtime creation failure is unrecoverable.
///
/// # Panics
///
/// Panics if the tokio runtime cannot be created.
pub(crate) fn runtime() -> &'static Runtime {
    try_runtime().expect(
        "failed to initialize tokio runtime — \
         possible causes: OS thread/resource limits, or calling from within \
         an existing tokio runtime (use tokio::task::spawn_blocking instead)",
    )
}

/// Converts a gRPC `tonic::Status` into an `SzError`.
///
/// The Senzing gRPC server encodes error details in the status message as
/// JSON containing a `"reason"` field with format `SENZ####E|message text`.
/// This function attempts to extract the Senzing error code and map it to
/// the most specific `SzError` variant. Falls back to `SzError::General`
/// if the message cannot be parsed.
///
/// The `method` parameter is the RPC method name (e.g., `"add_record"`),
/// included in fallback error messages for easier debugging.
pub(crate) fn grpc_to_sz_error(status: tonic::Status, method: &str) -> SzError {
    let message = status.message().to_string();
    if let Some(code) = parse_senzing_error_code(&message) {
        return crate::szerrortypes::sz_error_from_code(code, message);
    }
    // Fallback: gRPC error without a Senzing-specific error code.
    #[cfg(feature = "tracing")]
    tracing::warn!(
        method,
        grpc_code = %status.code(),
        raw_message = %message,
        "could not parse Senzing error code from gRPC status"
    );
    SzError::General {
        code: 0,
        message: format!("gRPC {} failed ({}): {}", method, status.code(), message),
    }
}

/// Attempts to extract a Senzing error code from a gRPC error message.
///
/// The message is expected to contain JSON with a `"reason"` field in
/// format `SENZ####E|...`. Characters at positions 4..8 of the reason
/// string are the 4-digit numeric error code.
fn parse_senzing_error_code(message: &str) -> Option<i32> {
    // Find the start of JSON in the message.
    let json_start = message.find('{')?;
    let json_str = &message[json_start..];

    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let reason = extract_reason(&value)?;

    // Reason format: "SENZ####E|message text"
    // Characters 4..8 are the numeric error code.
    if reason.len() < 8 {
        return None;
    }
    reason[4..8].parse::<i32>().ok()
}

/// Recursively searches a JSON value for a `"reason"` field.
fn extract_reason(value: &serde_json::Value) -> Option<&str> {
    let obj = value.as_object()?;

    if let Some(reason) = obj.get("reason").and_then(|v| v.as_str()) {
        return Some(reason);
    }

    // Recurse into nested "error" objects (Go SDK wraps errors).
    if let Some(inner) = obj.get("error") {
        return extract_reason(inner);
    }

    None
}

/// Converts a `serde_json::Error` into an `SzError::General` with a
/// descriptive message.
///
/// This is a convenience helper for the many convenience methods that parse
/// JSON responses. Instead of writing:
///
/// ```ignore
/// serde_json::from_str(&json).map_err(|e| SzError::General {
///     code: 0,
///     message: format!("failed to parse ... JSON: {e}"),
/// })?;
/// ```
///
/// You can write:
///
/// ```ignore
/// serde_json::from_str(&json).map_err(|e| json_parse_error("entity", e))?;
/// ```
pub(crate) fn json_parse_error(context: &str, err: serde_json::Error) -> SzError {
    SzError::General {
        code: 0,
        message: format!("failed to parse {context} JSON: {err}"),
    }
}

/// Converts a [`std::io::Error`] into an `SzError::General` with a
/// descriptive message.
///
/// This is a convenience helper for file I/O operations in convenience
/// methods (e.g., export-to-file). The `context` describes the operation
/// that failed.
///
/// ```ignore
/// File::create(path).map_err(|e| io_error("create export file", e))?;
/// ```
pub(crate) fn io_error(context: &str, err: std::io::Error) -> SzError {
    SzError::General {
        code: 0,
        message: format!("failed to {context}: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_reason() {
        let msg = r#"{"id":"SZSDK60044001","reason":"SENZ0023E|Conflicting DATA_SOURCE values"}"#;
        assert_eq!(parse_senzing_error_code(msg), Some(23));
    }

    #[test]
    fn test_parse_with_prefix() {
        let msg =
            r#"rpc error: code = Unknown desc = {"id":"X","reason":"SENZ1006E|Connection lost"}"#;
        assert_eq!(parse_senzing_error_code(msg), Some(1006));
    }

    #[test]
    fn test_parse_nested_error() {
        let msg = r#"{"id":"outer","error":{"id":"inner","reason":"SENZ0033E|Unknown record"}}"#;
        assert_eq!(parse_senzing_error_code(msg), Some(33));
    }

    #[test]
    fn test_parse_no_json() {
        assert_eq!(parse_senzing_error_code("plain text error"), None);
    }

    #[test]
    fn test_parse_no_reason_field() {
        let msg = r#"{"id":"X","message":"something"}"#;
        assert_eq!(parse_senzing_error_code(msg), None);
    }

    #[test]
    fn test_parse_short_reason() {
        let msg = r#"{"reason":"SEN"}"#;
        assert_eq!(parse_senzing_error_code(msg), None);
    }

    #[test]
    fn test_parse_non_numeric_code() {
        let msg = r#"{"reason":"SENZxxxxE|bad"}"#;
        assert_eq!(parse_senzing_error_code(msg), None);
    }

    #[test]
    fn test_grpc_to_sz_error_with_senzing_code() {
        let status = tonic::Status::unknown(
            r#"{"id":"test","reason":"SENZ2207E|Data source code [BOGUS] does not exist."}"#,
        );
        let err = grpc_to_sz_error(status, "add_record");
        assert!(matches!(err, SzError::UnknownDataSource { .. }));
    }

    #[test]
    fn test_json_parse_error() {
        let err: serde_json::Error = serde_json::from_str::<serde_json::Value>("}{").unwrap_err();
        let sz_err = json_parse_error("entity", err);
        match &sz_err {
            SzError::General { code, message } => {
                assert_eq!(*code, 0);
                assert!(message.contains("entity"));
                assert!(message.contains("JSON"));
            }
            _ => panic!("expected SzError::General, got {sz_err:?}"),
        }
    }

    #[test]
    fn test_grpc_to_sz_error_fallback() {
        let status = tonic::Status::not_found("no such thing");
        let err = grpc_to_sz_error(status, "get_entity");
        match &err {
            SzError::General { code, message } => {
                assert_eq!(*code, 0);
                assert!(message.contains("get_entity"));
                assert!(message.contains("no such thing"));
            }
            _ => panic!("expected SzError::General, got {err:?}"),
        }
    }
}
