use sz_sdk::SzError;
use tonic::transport::Channel;

use crate::pb_szengine::sz_engine_client::SzEngineClient;
use crate::pb_szengine::*;
use crate::runtime::runtime;

/// gRPC implementation of [`sz_sdk::SzEngine`].
pub struct SzEngineGrpc {
    client: SzEngineClient<Channel>,
}

impl SzEngineGrpc {
    /// Creates a new `SzEngineGrpc` from a gRPC channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            client: SzEngineClient::new(channel),
        }
    }
}

impl sz_sdk::SzEngine for SzEngineGrpc {
    fn add_record(
        &mut self,
        data_source_code: &str,
        record_id: &str,
        record_definition: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .add_record(AddRecordRequest {
                        data_source_code: data_source_code.to_string(),
                        record_id: record_id.to_string(),
                        record_definition: record_definition.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn close_export_report(&mut self, export_handle: usize) -> Result<(), SzError> {
        let mut client = self.client.clone();
        runtime()
            .block_on(async {
                client
                    .close_export_report(CloseExportReportRequest {
                        export_handle: export_handle as i64,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(())
    }

    fn count_redo_records(&self) -> Result<i64, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.count_redo_records(CountRedoRecordsRequest {}).await })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn delete_record(
        &mut self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .delete_record(DeleteRecordRequest {
                        data_source_code: data_source_code.to_string(),
                        record_id: record_id.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn destroy(&mut self) -> Result<(), SzError> {
        Ok(())
    }

    fn export_csv_entity_report(
        &mut self,
        csv_column_list: &str,
        flags: i64,
    ) -> Result<usize, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .export_csv_entity_report(ExportCsvEntityReportRequest {
                        csv_column_list: csv_column_list.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result as usize)
    }

    fn export_json_entity_report(&mut self, flags: i64) -> Result<usize, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .export_json_entity_report(ExportJsonEntityReportRequest { flags })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result as usize)
    }

    fn fetch_next(&self, export_handle: usize) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .fetch_next(FetchNextRequest {
                        export_handle: export_handle as i64,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn find_interesting_entities_by_entity_id(
        &self,
        entity_id: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .find_interesting_entities_by_entity_id(
                        FindInterestingEntitiesByEntityIdRequest { entity_id, flags },
                    )
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn find_interesting_entities_by_record_id(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .find_interesting_entities_by_record_id(
                        FindInterestingEntitiesByRecordIdRequest {
                            data_source_code: data_source_code.to_string(),
                            record_id: record_id.to_string(),
                            flags,
                        },
                    )
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn find_network_by_entity_id(
        &self,
        entity_ids: &str,
        max_degrees: i64,
        build_out_degrees: i64,
        build_out_max_entities: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .find_network_by_entity_id(FindNetworkByEntityIdRequest {
                        entity_ids: entity_ids.to_string(),
                        max_degrees,
                        build_out_degrees,
                        build_out_max_entities,
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn find_network_by_record_id(
        &self,
        record_keys: &str,
        max_degrees: i64,
        build_out_degrees: i64,
        build_out_max_entities: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .find_network_by_record_id(FindNetworkByRecordIdRequest {
                        record_keys: record_keys.to_string(),
                        max_degrees,
                        build_out_degrees,
                        build_out_max_entities,
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
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
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .find_path_by_entity_id(FindPathByEntityIdRequest {
                        start_entity_id,
                        end_entity_id,
                        max_degrees,
                        avoid_entity_ids: avoid_entity_ids.to_string(),
                        required_data_sources: required_data_sources.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
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
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .find_path_by_record_id(FindPathByRecordIdRequest {
                        start_data_source_code: start_data_source_code.to_string(),
                        start_record_id: start_record_id.to_string(),
                        end_data_source_code: end_data_source_code.to_string(),
                        end_record_id: end_record_id.to_string(),
                        max_degrees,
                        avoid_record_keys: avoid_record_keys.to_string(),
                        required_data_sources: required_data_sources.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_active_config_id(&self) -> Result<i64, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_active_config_id(GetActiveConfigIdRequest {})
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_entity_by_entity_id(&self, entity_id: i64, flags: i64) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_entity_by_entity_id(GetEntityByEntityIdRequest { entity_id, flags })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_entity_by_record_id(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_entity_by_record_id(GetEntityByRecordIdRequest {
                        data_source_code: data_source_code.to_string(),
                        record_id: record_id.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_record(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_record(GetRecordRequest {
                        data_source_code: data_source_code.to_string(),
                        record_id: record_id.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_record_preview(&self, record_definition: &str, flags: i64) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_record_preview(GetRecordPreviewRequest {
                        record_definition: record_definition.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_redo_record(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_redo_record(GetRedoRecordRequest {}).await })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_stats(&self) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async { client.get_stats(GetStatsRequest {}).await })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn get_virtual_entity_by_record_id(
        &self,
        record_keys: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .get_virtual_entity_by_record_id(GetVirtualEntityByRecordIdRequest {
                        record_keys: record_keys.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn how_entity_by_entity_id(&self, entity_id: i64, flags: i64) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .how_entity_by_entity_id(HowEntityByEntityIdRequest { entity_id, flags })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn prime_engine(&self) -> Result<(), SzError> {
        let mut client = self.client.clone();
        runtime()
            .block_on(async { client.prime_engine(PrimeEngineRequest {}).await })
            .map_err(grpc_to_sz_error)?;
        Ok(())
    }

    fn process_redo_record(&mut self, redo_record: &str, flags: i64) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .process_redo_record(ProcessRedoRecordRequest {
                        redo_record: redo_record.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn reevaluate_entity(&mut self, entity_id: i64, flags: i64) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .reevaluate_entity(ReevaluateEntityRequest { entity_id, flags })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn reevaluate_record(
        &mut self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .reevaluate_record(ReevaluateRecordRequest {
                        data_source_code: data_source_code.to_string(),
                        record_id: record_id.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn search_by_attributes(
        &self,
        attributes: &str,
        search_profile: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .search_by_attributes(SearchByAttributesRequest {
                        attributes: attributes.to_string(),
                        search_profile: search_profile.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn why_entities(
        &self,
        entity_id1: i64,
        entity_id2: i64,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .why_entities(WhyEntitiesRequest {
                        entity_id_1: entity_id1,
                        entity_id_2: entity_id2,
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn why_record_in_entity(
        &self,
        data_source_code: &str,
        record_id: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .why_record_in_entity(WhyRecordInEntityRequest {
                        data_source_code: data_source_code.to_string(),
                        record_id: record_id.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn why_records(
        &self,
        data_source_code1: &str,
        record_id1: &str,
        data_source_code2: &str,
        record_id2: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .why_records(WhyRecordsRequest {
                        data_source_code_1: data_source_code1.to_string(),
                        record_id_1: record_id1.to_string(),
                        data_source_code_2: data_source_code2.to_string(),
                        record_id_2: record_id2.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }

    fn why_search(
        &self,
        attributes: &str,
        entity_id: i64,
        search_profile: &str,
        flags: i64,
    ) -> Result<String, SzError> {
        let mut client = self.client.clone();
        let response = runtime()
            .block_on(async {
                client
                    .why_search(WhySearchRequest {
                        attributes: attributes.to_string(),
                        entity_id,
                        search_profile: search_profile.to_string(),
                        flags,
                    })
                    .await
            })
            .map_err(grpc_to_sz_error)?;
        Ok(response.into_inner().result)
    }
}

fn grpc_to_sz_error(status: tonic::Status) -> SzError {
    SzError::General {
        code: status.code() as i32,
        message: status.message().to_string(),
    }
}
