#[cfg(test)]
mod tests;

use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szengine::sz_engine_client::SzEngineClient;
use crate::pb_szengine::*;
use crate::runtime::{grpc_to_sz_error, runtime};

/// gRPC implementation of [`sz_sdk::SzEngine`].
#[derive(Debug, Clone)]
pub struct SzEngineGrpc {
    client: SzEngineClient<Channel>,
}

impl SzEngineGrpc {
    /// Creates a new `SzEngineGrpc` from a gRPC channel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sz_sdk_rust_grpc::{SzAbstractFactoryGrpc, SzEngineGrpc};
    ///
    /// let factory = SzAbstractFactoryGrpc::new_from_url("http://localhost:8261").unwrap();
    /// let engine = SzEngineGrpc::new(factory.channel());
    /// ```
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzEngineClient::new(channel),
        }
    }
}

impl std::fmt::Display for SzEngineGrpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SzEngineGrpc")
    }
}

impl sz_sdk::SzEngine for SzEngineGrpc {
    /// Adds a record to the repository.
    ///
    /// Returns with-info JSON when `flags` includes `SZ_WITH_INFO`.
    /// Returns [`SzError::UnknownDataSource`] if the data source is not registered.
    /// Returns [`SzError::BadInput`] for malformed `record_definition`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk::SzEngine;
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &mut SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let record = r#"{"NAME_FULL": "Robert Smith", "ADDR_FULL": "123 Main St"}"#;
    /// engine.add_record("CUSTOMERS", "1001", record, 0)?;
    /// # Ok(())
    /// # }
    /// ```
    fn add_record(
        &mut self,
        data_source_code: &str,
        record_id: &str,
        record_definition: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            add_record(AddRecordRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                record_definition: record_definition.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn close_export_report(&mut self, export_handle: usize) -> Result<(), SzError> {
        grpc_call!(
            self,
            close_export_report(CloseExportReportRequest {
                export_handle: export_handle as i64,
            })
        )?;
        Ok(())
    }

    fn count_redo_records(&self) -> Result<i64, SzError> {
        Ok(
            grpc_call!(self, count_redo_records(CountRedoRecordsRequest {}))?
                .into_inner()
                .result,
        )
    }

    /// Deletes a record from the repository.
    ///
    /// Returns with-info JSON when `flags` includes `SZ_WITH_INFO`.
    fn delete_record(
        &mut self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            delete_record(DeleteRecordRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    /// No-op for gRPC — the server manages its own lifecycle.
    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    /// Opens a CSV export and returns a handle for [`fetch_next`](sz_sdk::SzEngine::fetch_next).
    ///
    /// Prefer [`stream_export_csv_entity_report`](SzEngineGrpc::stream_export_csv_entity_report)
    /// or [`export_csv_as_string`](SzEngineGrpc::export_csv_as_string) for simpler usage.
    fn export_csv_entity_report(
        &mut self,
        csv_column_list: &str,
        flags: i64,
    ) -> Result<usize, SzError> {
        Ok(grpc_call!(
            self,
            export_csv_entity_report(ExportCsvEntityReportRequest {
                csv_column_list: csv_column_list.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result as usize)
    }

    /// Opens a JSON export and returns a handle for [`fetch_next`](sz_sdk::SzEngine::fetch_next).
    ///
    /// Prefer [`stream_export_json_entity_report`](SzEngineGrpc::stream_export_json_entity_report)
    /// or [`export_json_as_string`](SzEngineGrpc::export_json_as_string) for simpler usage.
    fn export_json_entity_report(&mut self, flags: i64) -> Result<usize, SzError> {
        Ok(grpc_call!(
            self,
            export_json_entity_report(ExportJsonEntityReportRequest { flags })
        )?
        .into_inner()
        .result as usize)
    }

    fn fetch_next(&self, export_handle: usize) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            fetch_next(FetchNextRequest {
                export_handle: export_handle as i64,
            })
        )?
        .into_inner()
        .result)
    }

    fn find_interesting_entities_by_entity_id(
        &self,
        entity_id: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            find_interesting_entities_by_entity_id(FindInterestingEntitiesByEntityIdRequest {
                entity_id,
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn find_interesting_entities_by_record_id(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            find_interesting_entities_by_record_id(FindInterestingEntitiesByRecordIdRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn find_network_by_entity_id(
        &self,
        entity_ids: &str,
        max_degrees: i64,
        build_out_degrees: i64,
        build_out_max_entities: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            find_network_by_entity_id(FindNetworkByEntityIdRequest {
                entity_ids: entity_ids.to_string(),
                max_degrees,
                build_out_degrees,
                build_out_max_entities,
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn find_network_by_record_id(
        &self,
        record_keys: &str,
        max_degrees: i64,
        build_out_degrees: i64,
        build_out_max_entities: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            find_network_by_record_id(FindNetworkByRecordIdRequest {
                record_keys: record_keys.to_string(),
                max_degrees,
                build_out_degrees,
                build_out_max_entities,
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn find_path_by_entity_id(
        &self,
        start_entity_id: i64,
        end_entity_id: i64,
        max_degrees: i64,
        avoid_entity_ids: &str,
        required_data_sources: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            find_path_by_entity_id(FindPathByEntityIdRequest {
                start_entity_id,
                end_entity_id,
                max_degrees,
                avoid_entity_ids: avoid_entity_ids.to_string(),
                required_data_sources: required_data_sources.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    #[allow(clippy::too_many_arguments)]
    fn find_path_by_record_id(
        &self,
        start_data_source_code: &str,
        start_record_id: &str,
        end_data_source_code: &str,
        end_record_id: &str,
        max_degrees: i64,
        avoid_record_keys: &str,
        required_data_sources: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            find_path_by_record_id(FindPathByRecordIdRequest {
                start_data_source_code: start_data_source_code.to_string(),
                start_record_id: start_record_id.to_string(),
                end_data_source_code: end_data_source_code.to_string(),
                end_record_id: end_record_id.to_string(),
                max_degrees,
                avoid_record_keys: avoid_record_keys.to_string(),
                required_data_sources: required_data_sources.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn get_active_config_id(&self) -> Result<i64, SzError> {
        Ok(
            grpc_call!(self, get_active_config_id(GetActiveConfigIdRequest {}))?
                .into_inner()
                .result,
        )
    }

    /// Returns JSON for a resolved entity. Returns [`SzError::NotFound`]
    /// if the entity does not exist.
    fn get_entity_by_entity_id(&self, entity_id: i64, flags: i64) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            get_entity_by_entity_id(GetEntityByEntityIdRequest { entity_id, flags })
        )?
        .into_inner()
        .result)
    }

    fn get_entity_by_record_id(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            get_entity_by_record_id(GetEntityByRecordIdRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn get_record(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            get_record(GetRecordRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn get_record_preview(&self, record_definition: &str, flags: i64) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            get_record_preview(GetRecordPreviewRequest {
                record_definition: record_definition.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn get_redo_record(&self) -> Result<String, SzError> {
        Ok(grpc_call!(self, get_redo_record(GetRedoRecordRequest {}))?
            .into_inner()
            .result)
    }

    fn get_stats(&self) -> Result<String, SzError> {
        Ok(grpc_call!(self, get_stats(GetStatsRequest {}))?
            .into_inner()
            .result)
    }

    fn get_virtual_entity_by_record_id(
        &self,
        record_keys: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            get_virtual_entity_by_record_id(GetVirtualEntityByRecordIdRequest {
                record_keys: record_keys.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn how_entity_by_entity_id(&self, entity_id: i64, flags: i64) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            how_entity_by_entity_id(HowEntityByEntityIdRequest { entity_id, flags })
        )?
        .into_inner()
        .result)
    }

    fn prime_engine(&self) -> Result<(), SzError> {
        grpc_call!(self, prime_engine(PrimeEngineRequest {}))?;
        Ok(())
    }

    fn process_redo_record(&mut self, redo_record: &str, flags: i64) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            process_redo_record(ProcessRedoRecordRequest {
                redo_record: redo_record.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn reevaluate_entity(&mut self, entity_id: i64, flags: i64) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            reevaluate_entity(ReevaluateEntityRequest { entity_id, flags })
        )?
        .into_inner()
        .result)
    }

    fn reevaluate_record(
        &mut self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            reevaluate_record(ReevaluateRecordRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    /// Searches for entities matching the given attribute JSON.
    /// Returns a JSON array of matching entities.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk::SzEngine;
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "SMITH"}]}"#;
    /// let results = engine.search_by_attributes(attrs, "", 0)?;
    /// println!("{results}");
    /// # Ok(())
    /// # }
    /// ```
    fn search_by_attributes(
        &self,
        attributes: &str,
        search_profile: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            search_by_attributes(SearchByAttributesRequest {
                attributes: attributes.to_string(),
                search_profile: search_profile.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn why_entities(
        &self,
        entity_id1: i64,
        entity_id2: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            why_entities(WhyEntitiesRequest {
                entity_id_1: entity_id1,
                entity_id_2: entity_id2,
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn why_record_in_entity(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            why_record_in_entity(WhyRecordInEntityRequest {
                data_source_code: data_source_code.to_string(),
                record_id: record_id.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn why_records(
        &self,
        data_source_code1: &str,
        record_id1: &str,
        data_source_code2: &str,
        record_id2: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            why_records(WhyRecordsRequest {
                data_source_code_1: data_source_code1.to_string(),
                record_id_1: record_id1.to_string(),
                data_source_code_2: data_source_code2.to_string(),
                record_id_2: record_id2.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }

    fn why_search(
        &self,
        attributes: &str,
        entity_id: i64,
        search_profile: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        Ok(grpc_call!(
            self,
            why_search(WhySearchRequest {
                attributes: attributes.to_string(),
                entity_id,
                search_profile: search_profile.to_string(),
                flags,
            })
        )?
        .into_inner()
        .result)
    }
}

/// Pass this to CSV export methods to use the server's default column list.
///
/// Equivalent to `""`. Using this constant makes the intent explicit:
/// ```no_run
/// # use sz_sdk_rust_grpc::SzEngineGrpc;
/// # use sz_sdk_rust_grpc::szengine::SZ_CSV_DEFAULT_COLUMNS;
/// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
/// let csv = engine.export_csv_as_string(SZ_CSV_DEFAULT_COLUMNS, 0)?;
/// # Ok(())
/// # }
/// ```
pub const SZ_CSV_DEFAULT_COLUMNS: &str = "";

/// Parsed engine statistics from the Senzing server.
///
/// Returned by [`SzEngineGrpc::get_stats_parsed`]. Fields that are absent
/// from the server response will use default values.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EngineStats {
    /// Number of workloads processed by the engine.
    pub workload: i64,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl std::fmt::Display for EngineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Engine stats ({} workloads)", self.workload)
    }
}

/// Parsed entity data from a `get_entity_by_*` response.
///
/// Returned by [`SzEngineGrpc::get_entity_parsed`]. Fields that are absent
/// from the server response will use default values.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityData {
    /// The resolved entity ID.
    pub entity_id: i64,
    /// The entity name (best-name chosen by resolution).
    pub entity_name: String,
    /// Number of records that resolved to this entity.
    pub record_count: i64,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl std::fmt::Display for EntityData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity {} ({}, {} records)",
            self.entity_id, self.entity_name, self.record_count
        )
    }
}

/// A single entity match from a search result.
///
/// Contained within [`SearchResults`].
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchEntity {
    /// The resolved entity ID.
    pub entity_id: i64,
    /// The entity name (best-name chosen by resolution).
    pub entity_name: String,
    /// Match score (higher = better match).
    pub match_score: i64,
}

impl std::fmt::Display for SearchEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity {} ({}, score {})",
            self.entity_id, self.entity_name, self.match_score
        )
    }
}

/// Parsed search results from [`SzEngineGrpc::search_parsed`].
///
/// Contains matching entities sorted by match score (descending).
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchResults {
    /// Matching entities, ordered by match score (best first).
    pub entities: Vec<SearchEntity>,
    /// The raw JSON response for access to any additional fields.
    pub raw_json: String,
}

impl SearchResults {
    /// Returns `true` if no entities matched the search.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns the number of matching entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns the best match (highest score), if any.
    #[must_use]
    pub fn best_match(&self) -> Option<&SearchEntity> {
        self.entities.first()
    }

    /// Returns the top `n` entities by match score.
    ///
    /// If fewer than `n` entities matched, returns all of them.
    #[must_use]
    pub fn top_n(&self, n: usize) -> Vec<&SearchEntity> {
        self.entities.iter().take(n).collect()
    }

    /// Returns a new `SearchResults` containing only entities with a
    /// match score greater than or equal to `min_score`.
    pub fn filter_by_min_score(&self, min_score: i64) -> SearchResults {
        SearchResults {
            entities: self
                .entities
                .iter()
                .filter(|e| e.match_score >= min_score)
                .cloned()
                .collect(),
            raw_json: self.raw_json.clone(),
        }
    }

    /// Returns the entity IDs of all matching entities.
    #[must_use]
    pub fn entity_ids(&self) -> Vec<i64> {
        self.entities.iter().map(|e| e.entity_id).collect()
    }
}

impl std::fmt::Display for SearchResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} matching entities", self.entities.len())
    }
}

/// Summary of a redo processing run.
///
/// Returned by [`SzEngineGrpc::process_all_redo_records`].
#[must_use]
#[derive(Debug)]
pub struct RedoSummary {
    /// Number of redo records successfully processed.
    pub processed: u32,
    /// Number of redo records that failed to process.
    pub failed: u32,
    /// Errors from failed redo records.
    pub errors: Vec<SzError>,
}

impl RedoSummary {
    /// Returns `true` if all redo records were processed successfully.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the total number of redo records attempted.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.processed + self.failed
    }

    /// Returns `true` if any redo records failed to process.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the success rate as a value between 0.0 and 1.0.
    ///
    /// Returns 1.0 if no redo records were attempted (vacuously true).
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 1.0;
        }
        f64::from(self.processed) / f64::from(total)
    }
}

impl std::fmt::Display for RedoSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Redo: {} processed, {} failed",
            self.processed, self.failed
        )
    }
}

/// Result of a batch operation ([`add_records_batch`](SzEngineGrpc::add_records_batch)
/// or [`delete_records_batch`](SzEngineGrpc::delete_records_batch)).
///
/// All records in the batch are attempted regardless of individual failures.
/// Check `errors` for any that failed.
#[must_use]
#[derive(Debug)]
pub struct BatchResult {
    /// Number of records successfully processed.
    pub successes: u32,
    /// Records that failed, as `(record_id, error)` pairs.
    pub errors: Vec<(String, SzError)>,
}

impl BatchResult {
    /// Returns `true` if all records were processed successfully.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the total number of records attempted (successes + failures).
    #[must_use]
    pub fn total(&self) -> usize {
        self.successes as usize + self.errors.len()
    }

    /// Returns the success rate as a value between 0.0 and 1.0.
    ///
    /// Returns 1.0 if no records were attempted (vacuously true).
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 1.0;
        }
        self.successes as f64 / total as f64
    }

    /// Returns only the errors that are retryable (transient failures).
    ///
    /// Use this to build a retry loop: extract the retryable failures,
    /// re-submit those records, and merge the results.
    #[must_use]
    pub fn retryable_errors(&self) -> Vec<&(String, SzError)> {
        self.errors
            .iter()
            .filter(|(_, err)| crate::szerrortypes::is_retryable(err))
            .collect()
    }

    /// Returns the record IDs of all failed records.
    #[must_use]
    pub fn failed_record_ids(&self) -> Vec<&str> {
        self.errors.iter().map(|(id, _)| id.as_str()).collect()
    }
}

impl<'a> IntoIterator for &'a BatchResult {
    type Item = &'a (String, SzError);
    type IntoIter = std::slice::Iter<'a, (String, SzError)>;

    /// Iterates over the `(record_id, error)` pairs for failed records.
    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

impl Default for BatchResult {
    /// Returns an empty success result (0 successes, no errors).
    fn default() -> Self {
        Self {
            successes: 0,
            errors: Vec::new(),
        }
    }
}

impl std::fmt::Display for BatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} succeeded, {} failed",
            self.successes,
            self.errors.len()
        )
    }
}

// ---- Streaming and convenience methods (concrete type only) ----

impl SzEngineGrpc {
    /// Exports JSON entity report via server-side streaming.
    ///
    /// Returns an iterator that yields one chunk per streamed response.
    /// This is more efficient than the handle-based
    /// [`export_json_entity_report`](sz_sdk::SzEngine::export_json_entity_report) +
    /// [`fetch_next`](sz_sdk::SzEngine::fetch_next) loop because it avoids
    /// per-chunk round-trips.
    ///
    /// # Usage patterns
    ///
    /// Collect the full export into a string:
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let full = engine.export_json_as_string(0)?; // convenience helper
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Process chunks incrementally (constant memory):
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// for chunk in engine.stream_export_json_entity_report(0)? {
    ///     let data = chunk?;
    ///     // process `data` without buffering the entire export
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Take only the first N chunks (early termination is safe):
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let first_chunks: Vec<String> = engine
    ///     .stream_export_json_entity_report(0)?
    ///     .take(5)
    ///     .collect::<Result<Vec<_>, _>>()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// **Memory note**: Each chunk is a single protobuf message. For large
    /// repositories, use incremental processing rather than collecting
    /// everything into a `String`.
    pub fn stream_export_json_entity_report(
        &self,
        flags: i64,
    ) -> Result<StreamingExportIterator<StreamExportJsonEntityReportResponse>, SzError> {
        let mut client = self.client.clone();
        let stream = runtime()
            .block_on(async {
                client
                    .stream_export_json_entity_report(StreamExportJsonEntityReportRequest { flags })
                    .await
            })
            .map_err(|s| grpc_to_sz_error(s, "stream_export_json_entity_report"))?
            .into_inner();
        Ok(StreamingExportIterator { stream })
    }

    /// Exports CSV entity report via server-side streaming.
    ///
    /// Returns an iterator that yields one chunk per streamed response.
    /// This is more efficient than the handle-based
    /// [`export_csv_entity_report`](sz_sdk::SzEngine::export_csv_entity_report) +
    /// [`fetch_next`](sz_sdk::SzEngine::fetch_next) loop.
    ///
    /// See [`stream_export_json_entity_report`](Self::stream_export_json_entity_report)
    /// for usage patterns — the API is identical.
    pub fn stream_export_csv_entity_report(
        &self,
        csv_column_list: &str,
        flags: i64,
    ) -> Result<StreamingExportIterator<StreamExportCsvEntityReportResponse>, SzError> {
        let mut client = self.client.clone();
        let stream = runtime()
            .block_on(async {
                client
                    .stream_export_csv_entity_report(StreamExportCsvEntityReportRequest {
                        csv_column_list: csv_column_list.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(|s| grpc_to_sz_error(s, "stream_export_csv_entity_report"))?
            .into_inner();
        Ok(StreamingExportIterator { stream })
    }

    /// Convenience method: exports the full JSON entity report as a single string.
    ///
    /// Uses server-side streaming internally. Buffers the entire export in
    /// memory — for very large repositories, prefer
    /// [`stream_export_json_entity_report`](Self::stream_export_json_entity_report)
    /// for incremental processing.
    pub fn export_json_as_string(&self, flags: i64) -> Result<String, SzError> {
        let iter = self.stream_export_json_entity_report(flags)?;
        collect_export(iter)
    }

    /// Convenience method: exports the full CSV entity report as a single string.
    ///
    /// Uses server-side streaming internally. Buffers the entire export in
    /// memory — for very large repositories, prefer
    /// [`stream_export_csv_entity_report`](Self::stream_export_csv_entity_report)
    /// for incremental processing.
    pub fn export_csv_as_string(
        &self,
        csv_column_list: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let iter = self.stream_export_csv_entity_report(csv_column_list, flags)?;
        collect_export(iter)
    }

    /// Convenience method: exports the JSON entity report directly to a file.
    ///
    /// Uses server-side streaming internally and writes each chunk to the
    /// file as it arrives, so memory usage stays constant regardless of
    /// repository size.
    ///
    /// Creates the file if it does not exist, or truncates it if it does.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// engine.export_json_entity_report_to_file("entities.json", 0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn export_json_entity_report_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
        flags: i64,
    ) -> Result<(), SzError> {
        use std::io::Write;
        let iter = self.stream_export_json_entity_report(flags)?;
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| crate::runtime::io_error("create export file", e))?;
        for chunk in iter {
            file.write_all(chunk?.as_bytes())
                .map_err(|e| crate::runtime::io_error("write export chunk", e))?;
        }
        file.flush()
            .map_err(|e| crate::runtime::io_error("flush export file", e))?;
        Ok(())
    }

    /// Convenience method: exports the CSV entity report directly to a file.
    ///
    /// Uses server-side streaming internally and writes each chunk to the
    /// file as it arrives, so memory usage stays constant regardless of
    /// repository size.
    ///
    /// Creates the file if it does not exist, or truncates it if it does.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # use sz_sdk_rust_grpc::SZ_CSV_DEFAULT_COLUMNS;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// engine.export_csv_entity_report_to_file(SZ_CSV_DEFAULT_COLUMNS, "entities.csv", 0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn export_csv_entity_report_to_file(
        &self,
        csv_column_list: &str,
        path: impl AsRef<std::path::Path>,
        flags: i64,
    ) -> Result<(), SzError> {
        use std::io::Write;
        let iter = self.stream_export_csv_entity_report(csv_column_list, flags)?;
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|e| crate::runtime::io_error("create export file", e))?;
        for chunk in iter {
            file.write_all(chunk?.as_bytes())
                .map_err(|e| crate::runtime::io_error("write export chunk", e))?;
        }
        file.flush()
            .map_err(|e| crate::runtime::io_error("flush export file", e))?;
        Ok(())
    }

    /// Exports JSON entity report via streaming, invoking a callback after
    /// each chunk with the running total of chunks received.
    ///
    /// Useful for progress reporting during large exports.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let report = engine.stream_export_json_with_callback(0, |chunks| {
    ///     println!("Received {chunks} chunks so far...");
    /// })?;
    /// println!("Total length: {}", report.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_export_json_with_callback<F>(
        &self,
        flags: i64,
        mut on_progress: F,
    ) -> Result<String, SzError>
    where
        F: FnMut(u64),
    {
        let iter = self.stream_export_json_entity_report(flags)?;
        let mut result = String::new();
        let mut count = 0u64;
        for chunk in iter {
            result.push_str(&chunk?);
            count += 1;
            on_progress(count);
        }
        Ok(result)
    }

    /// Exports CSV entity report via streaming, invoking a callback after
    /// each chunk with the running total of chunks received.
    ///
    /// Useful for progress reporting during large exports.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let report = engine.stream_export_csv_with_callback("", 0, |chunks| {
    ///     println!("Received {chunks} chunks so far...");
    /// })?;
    /// println!("Total length: {}", report.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_export_csv_with_callback<F>(
        &self,
        csv_column_list: &str,
        flags: i64,
        mut on_progress: F,
    ) -> Result<String, SzError>
    where
        F: FnMut(u64),
    {
        let iter = self.stream_export_csv_entity_report(csv_column_list, flags)?;
        let mut result = String::new();
        let mut count = 0u64;
        for chunk in iter {
            result.push_str(&chunk?);
            count += 1;
            on_progress(count);
        }
        Ok(result)
    }

    /// Adds multiple records in a single call.
    ///
    /// Each record is a `(data_source, record_id, record_definition)` tuple.
    /// All records are attempted regardless of individual failures. Returns a
    /// result summary containing the number of successes and any errors with
    /// their associated record IDs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &mut SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let records = &[
    ///     ("CUSTOMERS", "1001", r#"{"NAME_FULL": "Alice"}"#),
    ///     ("CUSTOMERS", "1002", r#"{"NAME_FULL": "Bob"}"#),
    /// ];
    /// let result = engine.add_records_batch(records, 0);
    /// println!("{} succeeded, {} failed", result.successes, result.errors.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_records_batch(&mut self, records: &[(&str, &str, &str)], flags: i64) -> BatchResult {
        use sz_sdk::SzEngine;
        let mut successes = 0u32;
        let mut errors = Vec::new();
        for &(data_source, record_id, record_definition) in records {
            match self.add_record(data_source, record_id, record_definition, flags) {
                Ok(_) => successes += 1,
                Err(e) => errors.push((record_id.to_string(), e)),
            }
        }
        BatchResult { successes, errors }
    }

    /// Deletes multiple records in a single call.
    ///
    /// Each record is a `(data_source, record_id)` tuple. All records are
    /// attempted regardless of individual failures. Returns a result summary
    /// containing the number of successes and any errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &mut SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let records = &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")];
    /// let result = engine.delete_records_batch(records, 0);
    /// println!("{} succeeded, {} failed", result.successes, result.errors.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_records_batch(&mut self, records: &[(&str, &str)], flags: i64) -> BatchResult {
        use sz_sdk::SzEngine;
        let mut successes = 0u32;
        let mut errors = Vec::new();
        for &(data_source, record_id) in records {
            match self.delete_record(data_source, record_id, flags) {
                Ok(_) => successes += 1,
                Err(e) => errors.push((record_id.to_string(), e)),
            }
        }
        BatchResult { successes, errors }
    }

    /// Returns the entity ID for a record identified by data source and record ID.
    ///
    /// This is a convenience wrapper that calls
    /// [`get_entity_by_record_id`](sz_sdk::SzEngine::get_entity_by_record_id),
    /// parses the JSON response, and extracts the `ENTITY_ID` field.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let entity_id = engine.lookup_entity_id("CUSTOMERS", "1001", 0)?;
    /// println!("Entity ID: {entity_id}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn lookup_entity_id(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<i64, SzError> {
        use sz_sdk::SzEngine;
        let response = self.get_entity_by_record_id(data_source_code, record_id, flags)?;
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| crate::runtime::json_parse_error("entity", e))?;
        json.get("RESOLVED_ENTITY")
            .and_then(|re| re.get("ENTITY_ID"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| SzError::General {
                code: 0,
                message: "ENTITY_ID not found in response".to_string(),
            })
    }

    /// Searches for entities matching the given attributes and returns
    /// the full entity detail for the top match.
    ///
    /// This is a convenience method that combines
    /// [`search_by_attributes`](sz_sdk::SzEngine::search_by_attributes) with
    /// [`get_entity_by_entity_id`](sz_sdk::SzEngine::get_entity_by_entity_id).
    /// Returns `Ok(None)` if no matching entity is found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "SMITH"}]}"#;
    /// if let Some(entity_json) = engine.search_and_resolve(attrs, "", 0)? {
    ///     println!("Top match: {entity_json}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_and_resolve(
        &self,
        attributes: &str,
        search_profile: &str,
        flags: i64,
    ) -> Result<Option<String>, SzError> {
        use sz_sdk::SzEngine;
        let search_result = self.search_by_attributes(attributes, search_profile, flags)?;
        let parsed: serde_json::Value = serde_json::from_str(&search_result)
            .map_err(|e| crate::runtime::json_parse_error("search result", e))?;
        let entities = parsed.get("RESOLVED_ENTITIES").and_then(|v| v.as_array());
        let entity_id = entities
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("ENTITY"))
            .and_then(|e| e.get("RESOLVED_ENTITY"))
            .and_then(|re| re.get("ENTITY_ID"))
            .and_then(|v| v.as_i64());
        match entity_id {
            Some(id) => Ok(Some(self.get_entity_by_entity_id(id, flags)?)),
            None => Ok(None),
        }
    }

    /// Returns parsed engine statistics from the server.
    ///
    /// This is a convenience wrapper around
    /// [`get_stats()`](sz_sdk::SzEngine::get_stats) that parses the JSON
    /// response into an [`EngineStats`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let stats = engine.get_stats_parsed()?;
    /// println!("{stats}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats_parsed(&self) -> Result<EngineStats, SzError> {
        use sz_sdk::SzEngine;
        let raw_json = self.get_stats()?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("stats", e))?;
        let workload = parsed
            .get("workload")
            .and_then(|w| w.get("apiVersion"))
            .and_then(|v| v.as_i64())
            .or_else(|| {
                // Try top-level workload count
                parsed
                    .get("workload")
                    .and_then(|w| w.get("loadedRecords"))
                    .and_then(|v| v.as_i64())
            })
            .unwrap_or(0);
        Ok(EngineStats { workload, raw_json })
    }

    /// Returns parsed entity data for the given entity ID.
    ///
    /// This is a convenience wrapper around
    /// [`get_entity_by_entity_id()`](sz_sdk::SzEngine::get_entity_by_entity_id)
    /// that parses the JSON response into an [`EntityData`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let entity = engine.get_entity_parsed(1, 0)?;
    /// println!("{entity}"); // "Entity 1 (Bob Smith, 3 records)"
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_entity_parsed(&self, entity_id: i64, flags: i64) -> Result<EntityData, SzError> {
        use sz_sdk::SzEngine;
        let raw_json = self.get_entity_by_entity_id(entity_id, flags)?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("entity", e))?;
        let resolved = parsed.get("RESOLVED_ENTITY");
        let eid = resolved
            .and_then(|re| re.get("ENTITY_ID"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| SzError::General {
                code: 0,
                message: "ENTITY_ID not found in entity response".to_string(),
            })?;
        let entity_name = resolved
            .and_then(|re| re.get("ENTITY_NAME"))
            .and_then(|v| v.as_str())
            .unwrap_or("") // optional field
            .to_string();
        let record_count = resolved
            .and_then(|re| re.get("RECORD_SUMMARY"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r.get("RECORD_COUNT").and_then(|v| v.as_i64()))
                    .sum()
            })
            .unwrap_or(0);
        Ok(EntityData {
            entity_id: eid,
            entity_name,
            record_count,
            raw_json,
        })
    }

    /// Looks up multiple entities by their entity IDs.
    ///
    /// Returns a `Vec` of entity JSON strings, one per input ID. If any
    /// lookup fails, the error is returned immediately and remaining lookups
    /// are skipped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let entity_ids = &[1, 2, 3];
    /// let entities = engine.get_entities_by_entity_ids(entity_ids, 0)?;
    /// for entity_json in &entities {
    ///     println!("{entity_json}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_entities_by_entity_ids(
        &self,
        entity_ids: &[i64],
        flags: i64,
    ) -> Result<Vec<String>, SzError> {
        use sz_sdk::SzEngine;
        let mut results = Vec::with_capacity(entity_ids.len());
        for &entity_id in entity_ids {
            results.push(self.get_entity_by_entity_id(entity_id, flags)?);
        }
        Ok(results)
    }

    /// Looks up multiple entities by their `(data_source, record_id)` pairs.
    ///
    /// Returns a `Vec` of entity JSON strings, one per input pair. If any
    /// lookup fails, the error is returned immediately and remaining lookups
    /// are skipped. For partial-failure tolerance, call
    /// [`get_entity_by_record_id`](sz_sdk::SzEngine::get_entity_by_record_id)
    /// in a loop instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let records = &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")];
    /// let entities = engine.get_entities_by_record_ids(records, 0)?;
    /// for entity_json in &entities {
    ///     println!("{entity_json}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_entities_by_record_ids(
        &self,
        record_ids: &[(&str, &str)],
        flags: i64,
    ) -> Result<Vec<String>, SzError> {
        use sz_sdk::SzEngine;
        let mut results = Vec::with_capacity(record_ids.len());
        for &(data_source, record_id) in record_ids {
            results.push(self.get_entity_by_record_id(data_source, record_id, flags)?);
        }
        Ok(results)
    }

    /// Returns parsed search results for the given attributes.
    ///
    /// This is a convenience wrapper around
    /// [`search_by_attributes()`](sz_sdk::SzEngine::search_by_attributes)
    /// that parses the JSON response into a [`SearchResults`] struct.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "SMITH"}]}"#;
    /// let results = engine.search_parsed(attrs, "", 0)?;
    /// if let Some(best) = results.best_match() {
    ///     println!("Best match: {best}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_parsed(
        &self,
        attributes: &str,
        search_profile: &str,
        flags: i64,
    ) -> Result<SearchResults, SzError> {
        use sz_sdk::SzEngine;
        let raw_json = self.search_by_attributes(attributes, search_profile, flags)?;
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| crate::runtime::json_parse_error("search results", e))?;
        let entities = parsed
            .get("RESOLVED_ENTITIES")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let entity = entry.get("ENTITY")?.get("RESOLVED_ENTITY")?;
                        let entity_id = entity.get("ENTITY_ID")?.as_i64()?;
                        let entity_name = entity
                            .get("ENTITY_NAME")
                            .and_then(|v| v.as_str())
                            .unwrap_or("") // optional field
                            .to_string();
                        let match_info = entry.get("MATCH_INFO");
                        let match_score = match_info
                            .and_then(|mi| mi.get("MATCH_SCORE"))
                            .and_then(|v| v.as_i64())
                            .or_else(|| {
                                match_info
                                    .and_then(|mi| mi.get("ERRULE_CODE"))
                                    .and_then(|v| v.as_i64())
                            })
                            .unwrap_or(0); // optional field
                        Some(SearchEntity {
                            entity_id,
                            entity_name,
                            match_score,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(SearchResults { entities, raw_json })
    }

    /// Drains the redo queue, processing all pending redo records.
    ///
    /// Returns a [`RedoSummary`] with counts of processed and failed records.
    /// Errors from individual redo records are collected (not short-circuited).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sz_sdk_rust_grpc::SzEngineGrpc;
    /// # fn example(engine: &mut SzEngineGrpc) -> Result<(), sz_sdk::SzError> {
    /// let summary = engine.process_all_redo_records(0)?;
    /// println!("{summary}"); // "Redo: 42 processed, 0 failed"
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_all_redo_records(&mut self, flags: i64) -> Result<RedoSummary, SzError> {
        use sz_sdk::SzEngine;
        let mut processed = 0u32;
        let mut failed = 0u32;
        let mut errors = Vec::new();

        loop {
            let count = self.count_redo_records()?;
            if count == 0 {
                break;
            }
            let redo_record = self.get_redo_record()?;
            if redo_record.is_empty() {
                break;
            }
            match self.process_redo_record(&redo_record, flags) {
                Ok(_) => processed += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(e);
                }
            }
        }

        Ok(RedoSummary {
            processed,
            failed,
            errors,
        })
    }
}

/// Collects all chunks from a streaming export iterator into a single string.
fn collect_export(iter: impl Iterator<Item = Result<String, SzError>>) -> Result<String, SzError> {
    let mut result = String::new();
    for chunk in iter {
        result.push_str(&chunk?);
    }
    Ok(result)
}

/// Iterator adapter over a tonic streaming response.
///
/// Each call to `next()` blocks the calling thread on the shared tokio
/// runtime to receive the next message from the gRPC stream. Do not call
/// from within an async context — use [`tokio::task::spawn_blocking`].
///
/// # Limitations
///
/// - **Single-use**: streams cannot be cloned or rewound. Once consumed
///   (or dropped), the data is gone. Start a new export to re-read.
/// - **Early termination is safe**: dropping the iterator mid-stream
///   abandons the client-side read. The server may continue sending for
///   a short time, but the connection cleans up normally.
/// - **Not `Send`**: the underlying `tonic::Streaming<T>` is not `Send`,
///   so this iterator must be consumed on the thread that created it.
pub struct StreamingExportIterator<T> {
    stream: tonic::Streaming<T>,
}

impl<T> std::fmt::Debug for StreamingExportIterator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingExportIterator")
            .finish_non_exhaustive()
    }
}

impl<T: StreamExportChunk> Iterator for StreamingExportIterator<T> {
    type Item = Result<String, SzError>;

    fn next(&mut self) -> Option<Self::Item> {
        match runtime().block_on(self.stream.message()) {
            Ok(Some(msg)) => Some(Ok(msg.into_result())),
            Ok(None) => None,
            Err(status) => Some(Err(grpc_to_sz_error(status, "stream_export"))),
        }
    }

    /// The number of remaining chunks is unknown (streaming), so the lower
    /// bound is 0 and there is no known upper bound.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<T: StreamExportChunk> std::iter::FusedIterator for StreamingExportIterator<T> {}

/// Extracts the `result` field from a streaming export gRPC response.
///
/// This trait is implemented for the generated protobuf response types
/// (`StreamExportJsonEntityReportResponse` and `StreamExportCsvEntityReportResponse`).
/// It is used internally by [`StreamingExportIterator`] to unify JSON and CSV
/// streaming into a single generic iterator.
///
/// **You do not need to implement this trait.** It is only public because it
/// appears as a bound on the public `StreamingExportIterator<T>` type.
pub trait StreamExportChunk {
    /// Consumes the response and returns the payload string.
    fn into_result(self) -> String;
}

impl StreamExportChunk for StreamExportJsonEntityReportResponse {
    fn into_result(self) -> String {
        self.result
    }
}

impl StreamExportChunk for StreamExportCsvEntityReportResponse {
    fn into_result(self) -> String {
        self.result
    }
}

#[cfg(test)]
mod assert_stream_traits {
    use super::*;
    use static_assertions::assert_not_impl_any;

    // StreamingExportIterator must NOT be Clone — verify at compile time.
    // (Send cannot be checked via assert_not_impl_any! due to auto-trait
    // inference limitations; the !Send constraint is documented instead.)
    assert_not_impl_any!(StreamingExportIterator<StreamExportJsonEntityReportResponse>: Clone);
    assert_not_impl_any!(StreamingExportIterator<StreamExportCsvEntityReportResponse>: Clone);
}
