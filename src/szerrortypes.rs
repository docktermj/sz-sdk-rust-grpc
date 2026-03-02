//! Maps Senzing error codes to specific [`SzError`] variants.
//!
//! Ported from the auto-generated Go SDK file
//! [`sz-sdk-go/szerror/szerrortypes.go`](https://github.com/senzing-garage/sz-sdk-go/blob/main/szerror/szerrortypes.go).
//! For each error code, the most specific error type from the Go SDK's
//! type hierarchy (first non-generic entry in `SzErrorTypes`) is selected.
//!
//! # Updating
//!
//! When new error codes are added upstream, update the `match` arms in
//! [`sz_error_from_code`] to include them. Refer to the Go SDK's
//! `szerrortypes.go` for the authoritative code-to-type mapping.

use sz_sdk::SzError;

/// Maps a Senzing error code to the appropriate `SzError` variant.
///
/// Returns `SzError::General` for unrecognized codes or codes that only
/// map to the generic `SzError` type in the Go SDK.
pub(crate) fn sz_error_from_code(code: i32, message: String) -> SzError {
    match code {
        // BadInput — invalid parameters, malformed JSON, conflicting flags, etc.
        // e.g., 23 = "Conflicting DATA_SOURCE values", 7 = "Invalid JSON in input"
        2 | 7 | 22 | 23 | 24 | 25 | 26 | 27 | 51 | 65 | 66 | 88 | 2057 | 3121 | 3122 | 3123
        | 3131 | 7303 | 7305 | 7313 | 7314 | 7426 | 8000 | 9115 | 9226 | 9414 => {
            SzError::BadInput { code, message }
        }

        // NotFound — entity or record does not exist.
        // e.g., 33 = "Unknown resolved entity", 37 = "Unknown record"
        33 | 37 => SzError::NotFound { code, message },

        // NotInitialized — engine not initialized or already destroyed.
        // e.g., 48 = "Engine not initialized", 53 = "Engine already destroyed"
        48 | 49 | 50 | 53 => SzError::NotInitialized { code, message },

        // Database — general database errors (query failures, schema issues).
        // e.g., 1000 = "SQL error", 54 = "Database table missing"
        54 | 1000 | 1001 | 1002 | 1003 | 1004 | 1005 | 1009 | 1010 | 1011 | 1012 | 1013 | 1014
        | 1015 | 1016 | 1017 | 1018 => SzError::Database { code, message },

        // DatabaseConnectionLost — the database connection dropped unexpectedly.
        // e.g., 1006 = "Lost connection to database", 1007 = "Connection reset"
        1006 | 1007 => SzError::DatabaseConnectionLost { code, message },

        // DatabaseTransient — temporary database failure, safe to retry.
        // e.g., 1008 = "Database deadlock detected"
        1008 => SzError::DatabaseTransient { code, message },

        // RetryTimeoutExceeded — optimistic locking retry limit reached.
        // e.g., 10 = "Exceeded retry timeout for concurrent modification"
        10 => SzError::RetryTimeoutExceeded { code, message },

        // License — license expired, invalid, or exceeds record limit.
        // e.g., 999 = "License expired", 9000 = "License record limit exceeded"
        999 | 9000 => SzError::License { code, message },

        // Unhandled — an unexpected internal error.
        // e.g., 87 = "Unexpected error in engine processing"
        87 => SzError::Unhandled { code, message },

        // ReplaceConflict — concurrent config replacement detected (CAS failure).
        // e.g., 7245 = "Default config ID changed since it was read"
        7245 => SzError::ReplaceConflict { code, message },

        // UnknownDataSource — referenced data source does not exist in config.
        // e.g., 2207 = "Data source code [BOGUS] does not exist"
        2207 => SzError::UnknownDataSource { code, message },

        // Configuration — invalid or corrupt engine configuration.
        // e.g., 14 = "Missing config key", 2001 = "Invalid config format"
        14 | 19 | 20 | 21 | 28 | 30 | 34 | 35 | 36 | 40 | 60 | 61 | 62 | 64 | 67 | 89 | 90
        | 1019 | 2001 | 2012 | 2015 | 2029 | 2034 | 2036 | 2037 | 2038 | 2041 | 2045 | 2047
        | 2048 | 2049 | 2050 | 2051 | 2061 | 2062 | 2065 | 2066 | 2067 | 2069 | 2070 | 2071
        | 2075 | 2076 | 2079 | 2080 | 2081 | 2082 | 2083 | 2084 | 2088 | 2089 | 2090 | 2091
        | 2092 | 2093 | 2094 | 2095 | 2099 | 2101 | 2102 | 2103 | 2104 | 2105 | 2106 | 2107
        | 2108 | 2109 | 2110 | 2111 | 2112 | 2113 | 2114 | 2117 | 2118 | 2120 | 2121 | 2123
        | 2131 | 2135 | 2136 | 2137 | 2138 | 2139 | 2205 | 2206 | 2209 | 2210 | 2211 | 2212
        | 2213 | 2214 | 2215 | 2216 | 2217 | 2218 | 2219 | 2220 | 2221 | 2222 | 2223 | 2224
        | 2225 | 2226 | 2227 | 2228 | 2230 | 2231 | 2232 | 2233 | 2234 | 2235 | 2236 | 2237
        | 2238 | 2239 | 2240 | 2241 | 2242 | 2243 | 2244 | 2245 | 2246 | 2247 | 2248 | 2249
        | 2250 | 2251 | 2252 | 2253 | 2254 | 2255 | 2256 | 2257 | 2258 | 2259 | 2260 | 2261
        | 2262 | 2263 | 2264 | 2266 | 2267 | 2268 | 2269 | 2270 | 2271 | 2272 | 2273 | 2274
        | 2275 | 2276 | 2277 | 2278 | 2279 | 2280 | 2281 | 2282 | 2283 | 2289 | 2290 | 2291
        | 7209 | 7211 | 7212 | 7213 | 7216 | 7217 | 7218 | 7220 | 7221 | 7222 | 7223 | 7224
        | 7226 | 7227 | 7228 | 7230 | 7232 | 7233 | 7234 | 7235 | 7236 | 7237 | 7239 | 7240
        | 7241 | 7243 | 7244 | 7246 | 7247 | 7317 | 7344 | 8501 | 8516 | 8517 | 8522 | 8525
        | 8526 | 8527 | 8528 | 8529 | 8536 | 8538 | 8540 | 8543 | 8544 | 8545 | 8556 | 8557
        | 8599 | 8601 | 8602 | 8604 | 8605 | 8606 | 8607 | 8608 | 8701 | 8702 | 9107 | 9110
        | 9111 | 9112 | 9113 | 9116 | 9117 | 9118 | 9119 | 9120 | 9210 | 9220 | 9222 | 9224
        | 9225 | 9228 | 9240 | 9241 | 9250 | 9251 | 9252 | 9253 | 9254 | 9255 | 9256 | 9257
        | 9258 | 9259 | 9260 | 9261 | 9264 | 9265 | 9266 | 9269 | 9270 | 9284 | 9285 | 9286
        | 9292 | 9293 | 9295 | 9296 | 9297 | 9298 | 9300 | 9301 | 9308 | 9309 | 9310 | 9408
        | 9409 | 9413 | 9500 | 9802 | 9803 => SzError::Configuration { code, message },

        // General (catchall for unrecognized codes and codes that only map to SzError)
        _ => SzError::General { code, message },
    }
}

/// Returns `true` if the error is likely transient and the operation
/// can be retried.
///
/// Retryable variants:
/// - [`SzError::DatabaseTransient`] — deadlock or lock timeout
/// - [`SzError::DatabaseConnectionLost`] — connection dropped, tonic will reconnect
/// - [`SzError::RetryTimeoutExceeded`] — optimistic locking contention
///
/// All other variants return `false`. For `Database` errors, retrying may
/// help if the underlying cause is transient (e.g., maintenance), but this
/// function conservatively returns `false` — callers should inspect the
/// error message if they want to retry `Database` errors.
///
/// # Example
///
/// ```
/// use sz_sdk::SzError;
/// use sz_sdk_rust_grpc::is_retryable;
///
/// let err = SzError::DatabaseTransient { code: 1008, message: "deadlock".into() };
/// assert!(is_retryable(&err));
///
/// let err = SzError::BadInput { code: 23, message: "bad input".into() };
/// assert!(!is_retryable(&err));
/// ```
#[must_use]
pub fn is_retryable(err: &SzError) -> bool {
    matches!(
        err,
        SzError::DatabaseTransient { .. }
            | SzError::DatabaseConnectionLost { .. }
            | SzError::RetryTimeoutExceeded { .. }
    )
}

/// Executes a fallible operation with automatic retry for transient errors.
///
/// When the operation returns a retryable error (as determined by
/// [`is_retryable`]), it is retried up to `max_attempts - 1` additional
/// times with exponential backoff and jitter. Non-retryable errors are
/// returned immediately.
///
/// # Arguments
///
/// * `max_attempts` — Total number of attempts (including the first). Must be ≥ 1.
/// * `base_delay` — Initial delay between retries. Doubles on each subsequent attempt.
/// * `f` — The operation to attempt. Called repeatedly until it succeeds, fails
///   with a non-retryable error, or exhausts all attempts.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use sz_sdk::SzError;
/// use sz_sdk_rust_grpc::with_retry;
///
/// let mut calls = 0;
/// let result = with_retry(3, Duration::from_millis(1), || {
///     calls += 1;
///     if calls < 3 {
///         Err(SzError::DatabaseTransient { code: 1008, message: "deadlock".into() })
///     } else {
///         Ok("done")
///     }
/// });
/// assert_eq!(result.unwrap(), "done");
/// assert_eq!(calls, 3);
/// ```
pub fn with_retry<F, T>(
    max_attempts: u32,
    base_delay: std::time::Duration,
    mut f: F,
) -> Result<T, SzError>
where
    F: FnMut() -> Result<T, SzError>,
{
    let max_delay = std::time::Duration::from_secs(30);
    let max_retries = max_attempts.saturating_sub(1);

    for attempt in 0..max_attempts {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) if is_retryable(&err) && attempt < max_retries => {
                let backoff = base_delay
                    .saturating_mul(1 << attempt.min(10))
                    .min(max_delay);
                // Add jitter: up to 25% of the backoff.
                let jitter_ms = (backoff.as_millis() as u64 / 4).max(1);
                let jitter = std::time::Duration::from_millis(simple_jitter_nanos() % jitter_ms);
                std::thread::sleep(backoff + jitter);
            }
            Err(err) => return Err(err),
        }
    }
    // Unreachable: the loop always returns.
    unreachable!()
}

/// Simple jitter source using system clock nanoseconds (no external deps).
fn simple_jitter_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_input_code() {
        let err = sz_error_from_code(23, "test".to_string());
        assert!(matches!(err, SzError::BadInput { .. }));
    }

    #[test]
    fn test_not_found_code() {
        let err = sz_error_from_code(33, "test".to_string());
        assert!(matches!(err, SzError::NotFound { .. }));
    }

    #[test]
    fn test_database_code() {
        let err = sz_error_from_code(1000, "test".to_string());
        assert!(matches!(err, SzError::Database { .. }));
    }

    #[test]
    fn test_database_connection_lost_code() {
        let err = sz_error_from_code(1006, "test".to_string());
        assert!(matches!(err, SzError::DatabaseConnectionLost { .. }));
    }

    #[test]
    fn test_database_transient_code() {
        let err = sz_error_from_code(1008, "test".to_string());
        assert!(matches!(err, SzError::DatabaseTransient { .. }));
    }

    #[test]
    fn test_not_initialized_code() {
        let err = sz_error_from_code(48, "test".to_string());
        assert!(matches!(err, SzError::NotInitialized { .. }));
    }

    #[test]
    fn test_license_code() {
        let err = sz_error_from_code(999, "test".to_string());
        assert!(matches!(err, SzError::License { .. }));
    }

    #[test]
    fn test_unhandled_code() {
        let err = sz_error_from_code(87, "test".to_string());
        assert!(matches!(err, SzError::Unhandled { .. }));
    }

    #[test]
    fn test_replace_conflict_code() {
        let err = sz_error_from_code(7245, "test".to_string());
        assert!(matches!(err, SzError::ReplaceConflict { .. }));
    }

    #[test]
    fn test_unknown_data_source_code() {
        let err = sz_error_from_code(2207, "test".to_string());
        assert!(matches!(err, SzError::UnknownDataSource { .. }));
    }

    #[test]
    fn test_configuration_code() {
        let err = sz_error_from_code(14, "test".to_string());
        assert!(matches!(err, SzError::Configuration { .. }));
    }

    #[test]
    fn test_retry_timeout_exceeded_code() {
        let err = sz_error_from_code(10, "test".to_string());
        assert!(matches!(err, SzError::RetryTimeoutExceeded { .. }));
    }

    #[test]
    fn test_unknown_code_falls_back_to_general() {
        let err = sz_error_from_code(99999, "test".to_string());
        assert!(matches!(err, SzError::General { .. }));
    }

    // is_retryable tests

    #[test]
    fn test_is_retryable_database_transient() {
        let err = SzError::DatabaseTransient {
            code: 1008,
            message: "deadlock".to_string(),
        };
        assert!(super::is_retryable(&err));
    }

    #[test]
    fn test_is_retryable_database_connection_lost() {
        let err = SzError::DatabaseConnectionLost {
            code: 1006,
            message: "lost".to_string(),
        };
        assert!(super::is_retryable(&err));
    }

    #[test]
    fn test_is_retryable_retry_timeout() {
        let err = SzError::RetryTimeoutExceeded {
            code: 10,
            message: "timeout".to_string(),
        };
        assert!(super::is_retryable(&err));
    }

    #[test]
    fn test_is_not_retryable_bad_input() {
        let err = SzError::BadInput {
            code: 23,
            message: "bad".to_string(),
        };
        assert!(!super::is_retryable(&err));
    }

    #[test]
    fn test_is_not_retryable_not_found() {
        let err = SzError::NotFound {
            code: 33,
            message: "missing".to_string(),
        };
        assert!(!super::is_retryable(&err));
    }

    #[test]
    fn test_is_not_retryable_general() {
        let err = SzError::General {
            code: 0,
            message: "general".to_string(),
        };
        assert!(!super::is_retryable(&err));
    }

    // with_retry tests

    #[test]
    fn test_with_retry_succeeds_first_attempt() {
        let mut calls = 0;
        let result = super::with_retry(3, std::time::Duration::from_millis(1), || {
            calls += 1;
            Ok::<_, SzError>("ok")
        });
        assert_eq!(result.unwrap(), "ok");
        assert_eq!(calls, 1);
    }

    #[test]
    fn test_with_retry_succeeds_after_transient_failures() {
        let mut calls = 0;
        let result = super::with_retry(5, std::time::Duration::from_millis(1), || {
            calls += 1;
            if calls < 3 {
                Err(SzError::DatabaseTransient {
                    code: 1008,
                    message: "deadlock".into(),
                })
            } else {
                Ok("recovered")
            }
        });
        assert_eq!(result.unwrap(), "recovered");
        assert_eq!(calls, 3);
    }

    #[test]
    fn test_with_retry_permanent_error_no_retry() {
        let mut calls = 0;
        let result = super::with_retry(5, std::time::Duration::from_millis(1), || {
            calls += 1;
            Err::<(), _>(SzError::BadInput {
                code: 23,
                message: "bad".into(),
            })
        });
        assert!(result.is_err());
        assert_eq!(calls, 1, "permanent errors should not be retried");
    }

    #[test]
    fn test_with_retry_exhausts_attempts() {
        let mut calls = 0;
        let result = super::with_retry(3, std::time::Duration::from_millis(1), || {
            calls += 1;
            Err::<(), _>(SzError::DatabaseTransient {
                code: 1008,
                message: "deadlock".into(),
            })
        });
        assert!(result.is_err());
        assert_eq!(calls, 3, "should try exactly max_attempts times");
    }

    #[test]
    fn test_with_retry_single_attempt() {
        let mut calls = 0;
        let result = super::with_retry(1, std::time::Duration::from_millis(1), || {
            calls += 1;
            Err::<(), _>(SzError::DatabaseTransient {
                code: 1008,
                message: "deadlock".into(),
            })
        });
        assert!(result.is_err());
        assert_eq!(calls, 1, "max_attempts=1 means no retries");
    }
}
