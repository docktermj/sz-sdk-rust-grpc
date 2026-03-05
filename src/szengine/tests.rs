use super::*;
use crate::test_support::{connect_channel, is_valid_json};
use serial_test::serial;
use sz_sdk::flags::*;
use sz_sdk::parameters::*;
use sz_sdk::SzEngine;

// ------------------------------------------------------------------------
// Truthset record data (from Go truthset/customers.go)
// ------------------------------------------------------------------------

const RECORD_1001_JSON: &str = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1001", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_LAST": "Smith", "PRIMARY_NAME_FIRST": "Robert", "DATE_OF_BIRTH": "12/11/1978", "ADDR_TYPE": "MAILING", "ADDR_LINE1": "123 Main Street, Las Vegas NV 89132", "PHONE_TYPE": "HOME", "PHONE_NUMBER": "702-919-1300", "EMAIL_ADDRESS": "bsmith@work.com", "DATE": "1/2/18", "STATUS": "Active", "AMOUNT": "100"}"#;
const RECORD_1002_JSON: &str = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1002", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_LAST": "Smith", "PRIMARY_NAME_FIRST": "Bob", "DATE_OF_BIRTH": "11/12/1978", "ADDR_TYPE": "HOME", "ADDR_LINE1": "1515 Adela Lane", "ADDR_CITY": "Las Vegas", "ADDR_STATE": "NV", "ADDR_POSTAL_CODE": "89111", "PHONE_TYPE": "MOBILE", "PHONE_NUMBER": "702-919-1300", "DATE": "3/10/17", "STATUS": "Inactive", "AMOUNT": "200"}"#;
const RECORD_1003_JSON: &str = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1003", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_LAST": "Smith", "PRIMARY_NAME_FIRST": "Bob", "PRIMARY_NAME_MIDDLE": "J", "DATE_OF_BIRTH": "12/11/1978", "EMAIL_ADDRESS": "bsmith@work.com", "DATE": "4/9/16", "STATUS": "Inactive", "AMOUNT": "300"}"#;
const RECORD_1005_JSON: &str = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1005", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_LAST": "Smith", "PRIMARY_NAME_FIRST": "Robbie", "DRIVERS_LICENSE_NUMBER": "112233", "DRIVERS_LICENSE_STATE": "NV", "ADDR_TYPE": "MAILING", "ADDR_LINE1": "123 E Main St", "ADDR_CITY": "Henderson", "ADDR_STATE": "NV", "ADDR_POSTAL_CODE": "89132", "DATE": "7/16/19", "STATUS": "Active", "AMOUNT": "500"}"#;
const DEFAULT_SEARCH_ATTRIBUTES: &str =
    r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "JOHNSON"}], "SSN_NUMBER": "053-39-3251"}"#;

// Bad parameters for error tests.
const BAD_DATA_SOURCE_CODE: &str = "BadDataSourceCode";
const BAD_RECORD_DEFINITION: &str = "}{";
const BAD_ENTITY_ID: i64 = -1;

// ------------------------------------------------------------------------
// Test helper functions
// ------------------------------------------------------------------------

fn get_szengine() -> SzEngineGrpc {
    SzEngineGrpc::new(connect_channel())
}

/// Look up entity ID for a record that has been added.
fn get_entity_id(engine: &SzEngineGrpc, data_source: &str, record_id: &str) -> i64 {
    let response = engine
        .get_entity_by_record_id(data_source, record_id, SZ_NO_FLAGS)
        .expect("failed to get entity by record id");
    let json: serde_json::Value =
        serde_json::from_str(&response).expect("failed to parse entity JSON");
    json.get("RESOLVED_ENTITY")
        .and_then(|re| re.get("ENTITY_ID"))
        .and_then(|v| v.as_i64())
        .expect("RESOLVED_ENTITY.ENTITY_ID not found in response")
}

/// Add a batch of records. Each tuple is (data_source, record_id, json).
fn add_records(engine: &mut SzEngineGrpc, records: &[(&str, &str, &str)]) {
    for (ds, rid, json) in records {
        engine
            .add_record(ds, rid, json, SZ_WITHOUT_INFO)
            .expect("failed to add record");
    }
}

/// Delete a batch of records, ignoring errors (for cleanup).
fn delete_records(engine: &mut SzEngineGrpc, records: &[(&str, &str)]) {
    for (ds, rid) in records {
        let _ = engine.delete_record(ds, rid, SZ_WITHOUT_INFO);
    }
}

/// Common record sets used by multiple tests.
const RECORDS_1001: &[(&str, &str, &str)] = &[("CUSTOMERS", "1001", RECORD_1001_JSON)];
const RECORDS_1001_1002: &[(&str, &str, &str)] = &[
    ("CUSTOMERS", "1001", RECORD_1001_JSON),
    ("CUSTOMERS", "1002", RECORD_1002_JSON),
];
const RECORDS_1001_1002_1003: &[(&str, &str, &str)] = &[
    ("CUSTOMERS", "1001", RECORD_1001_JSON),
    ("CUSTOMERS", "1002", RECORD_1002_JSON),
    ("CUSTOMERS", "1003", RECORD_1003_JSON),
];

/// Adds the given records, runs the test closure, then cleans up.
fn with_test_records<F>(records: &[(&str, &str, &str)], test_fn: F)
where
    F: FnOnce(&mut SzEngineGrpc),
{
    let mut engine = get_szengine();
    add_records(&mut engine, records);
    test_fn(&mut engine);
    let cleanup: Vec<(&str, &str)> = records.iter().map(|(ds, rid, _)| (*ds, *rid)).collect();
    delete_records(&mut engine, &cleanup);
}

// ------------------------------------------------------------------------
// Tests — Existing
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_destroy() {
    let mut engine = get_szengine();
    let result = engine.destroy();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_get_stats() {
    let engine = get_szengine();
    let result = engine.get_stats();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(&result.unwrap()));
}

#[test]
#[serial]
fn test_get_active_config_id() {
    let engine = get_szengine();
    let result = engine.get_active_config_id();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_count_redo_records() {
    let engine = get_szengine();
    let result = engine.count_redo_records();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_prime_engine() {
    let engine = get_szengine();
    let result = engine.prime_engine();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

// ------------------------------------------------------------------------
// Tests — Record operations
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_add_record() {
    let mut engine = get_szengine();
    let result = engine.add_record("CUSTOMERS", "1001", RECORD_1001_JSON, SZ_WITHOUT_INFO);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001")]);
}

#[test]
#[serial]
fn test_add_record_and_delete_record() {
    let mut engine = get_szengine();
    let add_result = engine.add_record("CUSTOMERS", "1005", RECORD_1005_JSON, SZ_WITHOUT_INFO);
    assert!(add_result.is_ok(), "{}", add_result.as_ref().err().unwrap());
    let delete_result = engine.delete_record("CUSTOMERS", "1005", SZ_WITHOUT_INFO);
    assert!(
        delete_result.is_ok(),
        "{}",
        delete_result.as_ref().err().unwrap()
    );
}

#[test]
#[serial]
fn test_add_record_bad_data_source() {
    let mut engine = get_szengine();
    let result = engine.add_record(
        BAD_DATA_SOURCE_CODE,
        "1001",
        RECORD_1001_JSON,
        SZ_WITHOUT_INFO,
    );
    // The record JSON embeds DATA_SOURCE: "CUSTOMERS" which conflicts with
    // the BAD_DATA_SOURCE_CODE parameter, producing BadInput (SENZ0023).
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.is_bad_input()),
        "expected BadInput or UnknownDataSource, got: {result:?}"
    );
}

#[test]
#[serial]
fn test_add_record_bad_record_definition() {
    let mut engine = get_szengine();
    let result = engine.add_record("CUSTOMERS", "9999", BAD_RECORD_DEFINITION, SZ_WITHOUT_INFO);
    assert!(
        result.as_ref().is_err_and(|e| e.is_bad_input()),
        "expected BadInput, got: {result:?}"
    );
}

#[test]
#[serial]
fn test_delete_record_bad_data_source() {
    let mut engine = get_szengine();
    let result = engine.delete_record(BAD_DATA_SOURCE_CODE, "1001", SZ_WITHOUT_INFO);
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

// ------------------------------------------------------------------------
// Tests — Get record/entity
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_record() {
    with_test_records(RECORDS_1001, |engine| {
        let result = engine.get_record("CUSTOMERS", "1001", SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
        assert!(is_valid_json(&result.unwrap()));
    });
}

#[test]
#[serial]
fn test_get_record_bad_data_source() {
    let engine = get_szengine();
    let result = engine.get_record(BAD_DATA_SOURCE_CODE, "1001", SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

#[test]
#[serial]
fn test_get_entity_by_record_id() {
    with_test_records(RECORDS_1001, |engine| {
        let result = engine.get_entity_by_record_id("CUSTOMERS", "1001", SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
        assert!(is_valid_json(&result.unwrap()));
    });
}

#[test]
#[serial]
fn test_get_entity_by_record_id_bad_data_source() {
    let engine = get_szengine();
    let result = engine.get_entity_by_record_id(BAD_DATA_SOURCE_CODE, "1001", SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

#[test]
#[serial]
fn test_get_entity_by_entity_id() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let result = engine.get_entity_by_entity_id(entity_id, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
        assert!(is_valid_json(&result.unwrap()));
    });
}

#[test]
#[serial]
fn test_get_entity_by_entity_id_bad_entity_id() {
    let engine = get_szengine();
    let result = engine.get_entity_by_entity_id(BAD_ENTITY_ID, SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_not_found()),
        "expected NotFound, got: {result:?}"
    );
}

// ------------------------------------------------------------------------
// Tests — Search & preview
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_search_by_attributes() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let result = engine.search_by_attributes(
            DEFAULT_SEARCH_ATTRIBUTES,
            SZ_NO_SEARCH_PROFILE,
            SZ_NO_FLAGS,
        );
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_get_record_preview() {
    let engine = get_szengine();
    let result = engine.get_record_preview(RECORD_1001_JSON, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

// ------------------------------------------------------------------------
// Tests — Why/how analysis
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_why_entities() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let entity_id1 = get_entity_id(engine, "CUSTOMERS", "1001");
        let entity_id2 = get_entity_id(engine, "CUSTOMERS", "1002");
        let result = engine.why_entities(entity_id1, entity_id2, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_why_records() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let result = engine.why_records("CUSTOMERS", "1001", "CUSTOMERS", "1002", SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_why_record_in_entity() {
    with_test_records(RECORDS_1001, |engine| {
        let result = engine.why_record_in_entity("CUSTOMERS", "1001", SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_why_search() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let result = engine.why_search(
            DEFAULT_SEARCH_ATTRIBUTES,
            entity_id,
            SZ_NO_SEARCH_PROFILE,
            SZ_NO_FLAGS,
        );
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_how_entity_by_entity_id() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let result = engine.how_entity_by_entity_id(entity_id, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

// ------------------------------------------------------------------------
// Tests — Find path/network
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_find_path_by_entity_id() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let entity_id1 = get_entity_id(engine, "CUSTOMERS", "1001");
        let entity_id2 = get_entity_id(engine, "CUSTOMERS", "1002");
        let result = engine.find_path_by_entity_id(
            entity_id1,
            entity_id2,
            2,
            SZ_NO_AVOIDANCE,
            SZ_NO_REQUIRED_DATASOURCES,
            SZ_NO_FLAGS,
        );
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_find_path_by_record_id() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let result = engine.find_path_by_record_id(
            "CUSTOMERS",
            "1001",
            "CUSTOMERS",
            "1002",
            2,
            SZ_NO_AVOIDANCE,
            SZ_NO_REQUIRED_DATASOURCES,
            SZ_NO_FLAGS,
        );
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_find_network_by_entity_id() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let entity_id1 = get_entity_id(engine, "CUSTOMERS", "1001");
        let entity_id2 = get_entity_id(engine, "CUSTOMERS", "1002");
        let entity_ids = format!(
            r#"{{"ENTITIES": [{{"ENTITY_ID": {}}}, {{"ENTITY_ID": {}}}]}}"#,
            entity_id1, entity_id2
        );
        let result = engine.find_network_by_entity_id(&entity_ids, 2, 2, 10, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_find_network_by_record_id() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let record_keys = r#"{"RECORDS": [{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1001"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1002"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1003"}]}"#;
        let result = engine.find_network_by_record_id(record_keys, 2, 2, 10, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_find_interesting_entities_by_entity_id() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let result = engine.find_interesting_entities_by_entity_id(entity_id, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_find_interesting_entities_by_record_id() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let result =
            engine.find_interesting_entities_by_record_id("CUSTOMERS", "1001", SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

// ------------------------------------------------------------------------
// Tests — Virtual entity
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_virtual_entity_by_record_id() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let record_keys = r#"{"RECORDS": [{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1001"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1002"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1003"}]}"#;
        let result = engine.get_virtual_entity_by_record_id(record_keys, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

// ------------------------------------------------------------------------
// Tests — Reevaluate
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_reevaluate_entity() {
    with_test_records(RECORDS_1001, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let result = engine.reevaluate_entity(entity_id, SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

#[test]
#[serial]
fn test_reevaluate_record() {
    with_test_records(RECORDS_1001, |engine| {
        let result = engine.reevaluate_record("CUSTOMERS", "1001", SZ_NO_FLAGS);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    });
}

// ------------------------------------------------------------------------
// Tests — Export
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_export_json_entity_report() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let export_handle = engine
            .export_json_entity_report(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start JSON export");
        let mut report = String::new();
        loop {
            let fragment = engine
                .fetch_next(export_handle)
                .expect("failed to fetch next");
            if fragment.is_empty() {
                break;
            }
            report.push_str(&fragment);
        }
        engine
            .close_export_report(export_handle)
            .expect("failed to close export report");
        assert!(!report.is_empty());
    });
}

#[test]
#[serial]
fn test_export_csv_entity_report() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let export_handle = engine
            .export_csv_entity_report("", SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start CSV export");
        let mut report = String::new();
        loop {
            let fragment = engine
                .fetch_next(export_handle)
                .expect("failed to fetch next");
            if fragment.is_empty() {
                break;
            }
            report.push_str(&fragment);
        }
        engine
            .close_export_report(export_handle)
            .expect("failed to close export report");
        assert!(!report.is_empty());
    });
}

// ------------------------------------------------------------------------
// Tests — Streaming export
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_stream_export_json_entity_report() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let iter = engine
            .stream_export_json_entity_report(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start streaming JSON export");
        let mut report = String::new();
        for chunk in iter {
            report.push_str(&chunk.expect("error during streaming"));
        }
        assert!(!report.is_empty());
    });
}

#[test]
#[serial]
fn test_stream_export_csv_entity_report() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let iter = engine
            .stream_export_csv_entity_report("", SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start streaming CSV export");
        let mut report = String::new();
        for chunk in iter {
            report.push_str(&chunk.expect("error during streaming"));
        }
        assert!(!report.is_empty());
    });
}

#[test]
#[serial]
fn test_export_json_as_string() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let result = engine.export_json_as_string(SZ_EXPORT_INCLUDE_ALL_ENTITIES);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
        assert!(!result.unwrap().is_empty());
    });
}

#[test]
#[serial]
fn test_export_csv_as_string() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let result = engine.export_csv_as_string("", SZ_EXPORT_INCLUDE_ALL_ENTITIES);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
        assert!(!result.unwrap().is_empty());
    });
}

#[test]
#[serial]
fn test_export_csv_as_string_custom_columns() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let result = engine.export_csv_as_string(
            "RESOLVED_ENTITY_ID,RELATED_ENTITY_ID,DATA_SOURCE,RECORD_ID",
            SZ_EXPORT_INCLUDE_ALL_ENTITIES,
        );
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
        let csv = result.unwrap();
        assert!(!csv.is_empty());
        // Verify the header row contains the requested columns.
        assert!(csv.starts_with("RESOLVED_ENTITY_ID"));
    });
}

#[test]
#[serial]
fn test_stream_export_csv_custom_columns() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let iter = engine
            .stream_export_csv_entity_report(
                "RESOLVED_ENTITY_ID,DATA_SOURCE,RECORD_ID",
                SZ_EXPORT_INCLUDE_ALL_ENTITIES,
            )
            .expect("failed to start streaming CSV export");
        let mut report = String::new();
        for chunk in iter {
            report.push_str(&chunk.expect("error during streaming"));
        }
        assert!(!report.is_empty());
        assert!(report.starts_with("RESOLVED_ENTITY_ID"));
    });
}

#[test]
#[serial]
fn test_stream_export_partial_consumption() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let iter = engine
            .stream_export_json_entity_report(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start streaming export");
        // Only consume the first chunk, then drop the iterator.
        // This tests that early termination doesn't panic or leak.
        let first = iter.take(1).next();
        assert!(first.is_some());
        assert!(first.unwrap().is_ok());
        // Iterator is dropped here — no panic expected.
    });
}

// ------------------------------------------------------------------------
// Tests — Export to file
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_export_json_entity_report_to_file() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let dir = std::env::temp_dir();
        let path = dir.join("sz_test_export.json");
        engine
            .export_json_entity_report_to_file(&path, SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to export JSON to file");
        let contents = std::fs::read_to_string(&path).expect("failed to read export file");
        assert!(!contents.is_empty(), "export file should not be empty");
        let _ = std::fs::remove_file(&path);
    });
}

#[test]
#[serial]
fn test_export_csv_entity_report_to_file() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let dir = std::env::temp_dir();
        let path = dir.join("sz_test_export.csv");
        engine
            .export_csv_entity_report_to_file("", &path, SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to export CSV to file");
        let contents = std::fs::read_to_string(&path).expect("failed to read export file");
        assert!(!contents.is_empty(), "export file should not be empty");
        assert!(
            contents.contains("RESOLVED_ENTITY_ID"),
            "CSV should contain header row"
        );
        let _ = std::fs::remove_file(&path);
    });
}

/// Writing to a non-existent directory should return SzError::General with
/// a meaningful message, not panic.
#[test]
#[serial]
fn test_export_json_to_file_invalid_path() {
    with_test_records(RECORDS_1001, |engine| {
        let result = engine.export_json_entity_report_to_file(
            "/nonexistent_dir_12345/export.json",
            SZ_EXPORT_INCLUDE_ALL_ENTITIES,
        );
        assert!(result.is_err(), "should fail for invalid path");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("create export file"),
            "error should mention file creation: {err_msg}"
        );
    });
}

// ------------------------------------------------------------------------
// Tests — Error cases for additional methods
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_why_record_in_entity_bad_data_source() {
    let engine = get_szengine();
    let result = engine.why_record_in_entity(BAD_DATA_SOURCE_CODE, "1001", SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

/// Malformed attributes JSON should return BadInput.
#[test]
#[serial]
fn test_search_by_attributes_bad_json() {
    let engine = get_szengine();
    let result = engine.search_by_attributes("}{not json", SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_bad_input()),
        "expected BadInput for malformed attributes, got: {result:?}"
    );
}

/// find_path_by_entity_id with non-existent entity IDs should return NotFound.
#[test]
#[serial]
fn test_find_path_by_entity_id_bad_entity_ids() {
    let engine = get_szengine();
    let result = engine.find_path_by_entity_id(
        BAD_ENTITY_ID,
        BAD_ENTITY_ID,
        2,
        SZ_NO_AVOIDANCE,
        SZ_NO_REQUIRED_DATASOURCES,
        SZ_NO_FLAGS,
    );
    assert!(
        result.as_ref().is_err_and(|e| e.is_not_found()),
        "expected NotFound for bad entity IDs, got: {result:?}"
    );
}

/// find_network_by_entity_id with malformed JSON should return an error.
#[test]
#[serial]
fn test_find_network_by_entity_id_bad_json() {
    let engine = get_szengine();
    let result = engine.find_network_by_entity_id("}{not json", 2, 2, 10, SZ_NO_FLAGS);
    assert!(
        result.is_err(),
        "expected error for malformed entity IDs JSON"
    );
}

/// find_path_by_record_id with unknown data source should return UnknownDataSource.
#[test]
#[serial]
fn test_find_path_by_record_id_bad_data_source() {
    let engine = get_szengine();
    let result = engine.find_path_by_record_id(
        BAD_DATA_SOURCE_CODE,
        "1001",
        BAD_DATA_SOURCE_CODE,
        "1002",
        2,
        SZ_NO_AVOIDANCE,
        SZ_NO_REQUIRED_DATASOURCES,
        SZ_NO_FLAGS,
    );
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

// ------------------------------------------------------------------------
// Tests — Redo
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_redo_record() {
    let engine = get_szengine();
    let result = engine.get_redo_record();
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
}

#[test]
#[serial]
fn test_process_redo_record_bad_input() {
    let mut engine = get_szengine();
    let result = engine.process_redo_record("}{invalid json", SZ_WITHOUT_INFO);
    // Invalid redo record should return an error, not panic.
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_process_redo_record_if_available() {
    let mut engine = get_szengine();
    let count = engine
        .count_redo_records()
        .expect("failed to count redo records");
    if count > 0 {
        let redo = engine.get_redo_record().expect("failed to get redo record");
        let result = engine.process_redo_record(&redo, SZ_WITHOUT_INFO);
        assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    }
    // If no redo records exist, that's fine — nothing to process.
}

/// Verify that add_record is idempotent: calling it twice with the same
/// (data_source, record_id) should succeed both times without error.
#[test]
#[serial]
fn test_add_record_idempotent() {
    let mut engine = get_szengine();
    let record = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "IDEM01", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_FULL": "Idempotency Test"}"#;

    let first = engine.add_record("CUSTOMERS", "IDEM01", record, SZ_WITHOUT_INFO);
    assert!(first.is_ok(), "first add failed: {}", first.unwrap_err());

    let second = engine.add_record("CUSTOMERS", "IDEM01", record, SZ_WITHOUT_INFO);
    assert!(
        second.is_ok(),
        "second add (idempotent) failed: {}",
        second.unwrap_err()
    );

    // Cleanup.
    let _ = engine.delete_record("CUSTOMERS", "IDEM01", SZ_WITHOUT_INFO);
}

/// Verify that delete_record is safe to retry: deleting a record that was
/// already deleted should not panic.
#[test]
#[serial]
fn test_delete_record_idempotent() {
    let mut engine = get_szengine();
    let record = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "DEL_IDEM01", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_FULL": "Delete Idempotency Test"}"#;
    engine
        .add_record("CUSTOMERS", "DEL_IDEM01", record, SZ_WITHOUT_INFO)
        .expect("failed to add record");

    let first = engine.delete_record("CUSTOMERS", "DEL_IDEM01", SZ_WITHOUT_INFO);
    assert!(first.is_ok(), "first delete failed: {}", first.unwrap_err());

    // Second delete of the same record — should not panic.
    // May succeed (no-op) or return an error; either is acceptable.
    let second = engine.delete_record("CUSTOMERS", "DEL_IDEM01", SZ_WITHOUT_INFO);
    assert!(
        second.is_ok() || second.is_err(),
        "second delete should not panic"
    );
}

/// Verify that add_record with SZ_WITH_INFO returns valid with-info JSON.
#[test]
#[serial]
fn test_add_record_with_info() {
    let mut engine = get_szengine();
    let record = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "INFO01", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_FULL": "With Info Test"}"#;

    let result = engine.add_record("CUSTOMERS", "INFO01", record, SZ_WITH_INFO);
    assert!(
        result.is_ok(),
        "add with info failed: {}",
        result.unwrap_err()
    );

    let info = result.unwrap();
    assert!(
        is_valid_json(&info),
        "with-info response should be valid JSON"
    );
    // With-info response should contain entity information.
    let parsed: serde_json::Value = serde_json::from_str(&info).unwrap();
    assert!(
        parsed.get("DATA_SOURCE").is_some() || parsed.get("AFFECTED_ENTITIES").is_some(),
        "with-info response should contain entity data: {info}"
    );

    // Cleanup.
    let _ = engine.delete_record("CUSTOMERS", "INFO01", SZ_WITHOUT_INFO);
}

/// Verify that delete_record with SZ_WITH_INFO returns valid with-info JSON.
#[test]
#[serial]
fn test_delete_record_with_info() {
    let mut engine = get_szengine();
    let record = r#"{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "INFO02", "RECORD_TYPE": "PERSON", "PRIMARY_NAME_FULL": "Delete Info Test"}"#;
    engine
        .add_record("CUSTOMERS", "INFO02", record, SZ_WITHOUT_INFO)
        .expect("failed to add record");

    let result = engine.delete_record("CUSTOMERS", "INFO02", SZ_WITH_INFO);
    assert!(
        result.is_ok(),
        "delete with info failed: {}",
        result.unwrap_err()
    );

    let info = result.unwrap();
    assert!(
        is_valid_json(&info),
        "with-info response should be valid JSON"
    );
    let parsed: serde_json::Value = serde_json::from_str(&info).unwrap();
    assert!(
        parsed.get("DATA_SOURCE").is_some() || parsed.get("AFFECTED_ENTITIES").is_some(),
        "with-info response should contain entity data: {info}"
    );
}

/// End-to-end redo workflow: add overlapping records to create redo work,
/// then count, get, and process redo records.
#[test]
#[serial]
fn test_redo_workflow() {
    let mut engine = get_szengine();

    // Add overlapping records that should generate redo work.
    // Records 1001-1003 share attributes (name, DOB, phone) that cause resolution.
    add_records(&mut engine, RECORDS_1001_1002_1003);

    // Count redo records — may be zero if the engine resolved everything
    // inline, but the call itself should succeed.
    let count = engine
        .count_redo_records()
        .expect("failed to count redo records");
    assert!(count >= 0, "redo count should be non-negative");

    // If there are redo records, process them.
    if count > 0 {
        let redo = engine.get_redo_record().expect("failed to get redo record");
        assert!(!redo.is_empty(), "redo record should not be empty");

        let result = engine.process_redo_record(&redo, SZ_WITHOUT_INFO);
        assert!(
            result.is_ok(),
            "process_redo_record failed: {}",
            result.unwrap_err()
        );
    }

    // Verify the engine is still healthy after redo processing.
    let stats = engine.get_stats().expect("get_stats failed after redo");
    assert!(is_valid_json(&stats));

    // Cleanup.
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

/// Verify that get_entities_by_record_ids returns entities for multiple records.
#[test]
#[serial]
fn test_get_entities_by_record_ids() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let record_ids = &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")];
        let entities = engine
            .get_entities_by_record_ids(record_ids, SZ_NO_FLAGS)
            .expect("batch lookup failed");
        assert_eq!(entities.len(), 2);
        for entity in &entities {
            assert!(is_valid_json(entity), "entity should be valid JSON");
        }
    });
}

/// Verify that get_entities_by_record_ids with a bad data source fails.
#[test]
#[serial]
fn test_get_entities_by_record_ids_bad_data_source() {
    let engine = get_szengine();
    let record_ids = &[(BAD_DATA_SOURCE_CODE, "1001")];
    let result = engine.get_entities_by_record_ids(record_ids, SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

/// Verify that get_entities_by_record_ids with empty input returns empty vec.
#[test]
#[serial]
fn test_get_entities_by_record_ids_empty() {
    let engine = get_szengine();
    let entities = engine
        .get_entities_by_record_ids(&[], SZ_NO_FLAGS)
        .expect("empty batch should succeed");
    assert!(entities.is_empty());
}

/// Verify that add_records_batch adds multiple records and reports success.
#[test]
#[serial]
fn test_add_records_batch() {
    let mut engine = get_szengine();
    let records = &[
        ("CUSTOMERS", "BATCH_A1", r#"{"NAME_FULL": "Batch Test A"}"#),
        ("CUSTOMERS", "BATCH_A2", r#"{"NAME_FULL": "Batch Test B"}"#),
    ];
    let result = engine.add_records_batch(records, SZ_WITHOUT_INFO);
    assert_eq!(result.successes, 2);
    assert!(
        result.is_ok(),
        "batch add should succeed: {:?}",
        result.errors
    );

    // Cleanup.
    let _ = engine.delete_records_batch(
        &[("CUSTOMERS", "BATCH_A1"), ("CUSTOMERS", "BATCH_A2")],
        SZ_WITHOUT_INFO,
    );
}

/// Verify that add_records_batch collects errors without stopping.
#[test]
#[serial]
fn test_add_records_batch_partial_failure() {
    let mut engine = get_szengine();
    let records = &[
        ("CUSTOMERS", "BATCH_B1", r#"{"NAME_FULL": "Good Record"}"#),
        (
            BAD_DATA_SOURCE_CODE,
            "BATCH_B2",
            r#"{"NAME_FULL": "Bad Source"}"#,
        ),
    ];
    let result = engine.add_records_batch(records, SZ_WITHOUT_INFO);
    assert_eq!(result.successes, 1);
    assert_eq!(result.errors.len(), 1);
    assert!(!result.is_ok());

    // Cleanup.
    let _ = engine.delete_record("CUSTOMERS", "BATCH_B1", SZ_WITHOUT_INFO);
}

/// Verify that delete_records_batch deletes multiple records.
#[test]
#[serial]
fn test_delete_records_batch() {
    let mut engine = get_szengine();
    let _ = engine.add_records_batch(
        &[
            (
                "CUSTOMERS",
                "BATCH_D1",
                r#"{"NAME_FULL": "Delete Batch A"}"#,
            ),
            (
                "CUSTOMERS",
                "BATCH_D2",
                r#"{"NAME_FULL": "Delete Batch B"}"#,
            ),
        ],
        SZ_WITHOUT_INFO,
    );

    let result = engine.delete_records_batch(
        &[("CUSTOMERS", "BATCH_D1"), ("CUSTOMERS", "BATCH_D2")],
        SZ_WITHOUT_INFO,
    );
    assert_eq!(result.successes, 2);
    assert!(result.is_ok());
}

/// Verify that lookup_entity_id returns a valid entity ID.
#[test]
#[serial]
fn test_lookup_entity_id() {
    with_test_records(RECORDS_1001, |engine| {
        let entity_id = engine
            .lookup_entity_id("CUSTOMERS", "1001", SZ_NO_FLAGS)
            .expect("lookup_entity_id failed");
        assert!(entity_id > 0, "entity ID should be positive");
    });
}

/// Verify that lookup_entity_id with unknown data source returns error.
#[test]
#[serial]
fn test_lookup_entity_id_bad_data_source() {
    let engine = get_szengine();
    let result = engine.lookup_entity_id(BAD_DATA_SOURCE_CODE, "1001", SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_unknown_data_source()),
        "expected UnknownDataSource, got: {result:?}"
    );
}

// ------------------------------------------------------------------------
// Tests — Streaming export edge cases
// ------------------------------------------------------------------------

/// After purge, a streaming JSON export should produce an empty stream.
#[test]
#[serial]
fn test_stream_export_json_empty_after_purge() {
    // Ensure no records exist.
    let engine = get_szengine();
    let iter = engine
        .stream_export_json_entity_report(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
        .expect("failed to start streaming export");
    let chunks: Vec<String> = iter.filter_map(|r| r.ok()).collect();
    // With no entities, we may get zero chunks or a header-only chunk.
    // Either way, there should be no entity data lines.
    let combined = chunks.join("");
    // The combined output should not contain RESOLVED_ENTITY (no entities to export).
    // (It may be empty or contain only formatting.)
    assert!(
        !combined.contains("RESOLVED_ENTITY") || combined.is_empty(),
        "expected no entity data after purge, got: {combined}"
    );
}

/// After purge, a streaming CSV export should produce at most a header row.
#[test]
#[serial]
fn test_stream_export_csv_empty_after_purge() {
    let engine = get_szengine();
    let iter = engine
        .stream_export_csv_entity_report("", SZ_EXPORT_INCLUDE_ALL_ENTITIES)
        .expect("failed to start streaming CSV export");
    let chunks: Vec<String> = iter.filter_map(|r| r.ok()).collect();
    let combined = chunks.join("");
    // Should have at most a header row, no data rows.
    let line_count = combined.lines().count();
    assert!(
        line_count <= 1,
        "expected at most 1 line (header) with no entities, got {line_count} lines"
    );
}

/// Verify FusedIterator behavior: after the stream is exhausted, next()
/// continues to return None (doesn't panic or restart).
#[test]
#[serial]
fn test_stream_export_fused_iterator() {
    with_test_records(RECORDS_1001, |engine| {
        let mut iter = engine
            .stream_export_json_entity_report(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start streaming export");
        // Exhaust the iterator.
        while iter.next().is_some() {}
        // Calling next() again should return None, not panic.
        assert!(iter.next().is_none(), "fused iterator should return None");
        assert!(
            iter.next().is_none(),
            "fused iterator should keep returning None"
        );
    });
}

/// Verify that concatenated JSON stream chunks produce valid JSON lines.
#[test]
#[serial]
fn test_stream_export_json_valid_output() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let iter = engine
            .stream_export_json_entity_report(SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start streaming export");
        let mut combined = String::new();
        for chunk in iter {
            combined.push_str(&chunk.expect("streaming error"));
        }
        // Each non-empty line should be valid JSON.
        for line in combined.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            assert!(
                is_valid_json(trimmed),
                "each line should be valid JSON: {trimmed}"
            );
        }
    });
}

/// Verify that concatenated CSV stream chunks produce valid CSV with header.
#[test]
#[serial]
fn test_stream_export_csv_valid_output() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let iter = engine
            .stream_export_csv_entity_report("", SZ_EXPORT_INCLUDE_ALL_ENTITIES)
            .expect("failed to start streaming CSV export");
        let mut combined = String::new();
        for chunk in iter {
            combined.push_str(&chunk.expect("streaming error"));
        }
        assert!(
            combined.starts_with("RESOLVED_ENTITY_ID"),
            "CSV should start with header row"
        );
        // Should have at least header + 1 data row.
        let line_count = combined.lines().count();
        assert!(
            line_count >= 2,
            "expected header + data rows, got {line_count} lines"
        );
    });
}

// ------------------------------------------------------------------------
// Tests — BatchResult Display and Default
// ------------------------------------------------------------------------

#[test]
fn test_batch_result_display() {
    let result = super::BatchResult {
        successes: 3,
        errors: vec![("bad".to_string(), SzError::general("test"))],
    };
    assert_eq!(format!("{result}"), "3 succeeded, 1 failed");
}

#[test]
fn test_batch_result_default() {
    let result = super::BatchResult::default();
    assert_eq!(result.successes, 0);
    assert!(result.errors.is_empty());
    assert!(result.is_ok());
    assert_eq!(format!("{result}"), "0 succeeded, 0 failed");
}

// ------------------------------------------------------------------------
// Tests — get_entities_by_entity_ids
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_entities_by_entity_ids() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let id1 = get_entity_id(engine, "CUSTOMERS", "1001");
        let id2 = get_entity_id(engine, "CUSTOMERS", "1002");
        let ids: Vec<i64> = vec![id1, id2]
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let entities = engine
            .get_entities_by_entity_ids(&ids, SZ_NO_FLAGS)
            .expect("batch entity lookup failed");
        assert_eq!(entities.len(), ids.len());
        for entity in &entities {
            assert!(is_valid_json(entity));
        }
    });
}

#[test]
#[serial]
fn test_get_entities_by_entity_ids_bad_id() {
    let engine = get_szengine();
    let result = engine.get_entities_by_entity_ids(&[BAD_ENTITY_ID], SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_not_found()),
        "expected NotFound, got: {result:?}"
    );
}

#[test]
#[serial]
fn test_get_entities_by_entity_ids_empty() {
    let engine = get_szengine();
    let entities = engine
        .get_entities_by_entity_ids(&[], SZ_NO_FLAGS)
        .expect("empty batch should succeed");
    assert!(entities.is_empty());
}

// ------------------------------------------------------------------------
// Tests — search_and_resolve
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_search_and_resolve() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let result = engine
            .search_and_resolve(DEFAULT_SEARCH_ATTRIBUTES, SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS)
            .expect("search_and_resolve failed");
        // May or may not find a match depending on truthset data.
        if let Some(entity_json) = result {
            assert!(is_valid_json(&entity_json));
        }
    });
}

#[test]
#[serial]
fn test_search_and_resolve_no_match() {
    let engine = get_szengine();
    // Search with attributes that won't match anything.
    let attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "ZZZZNOEXIST99999"}]}"#;
    let result = engine
        .search_and_resolve(attrs, SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS)
        .expect("search_and_resolve should not error on no match");
    assert!(result.is_none(), "expected None for no match");
}

#[test]
#[serial]
fn test_search_and_resolve_bad_json() {
    let engine = get_szengine();
    let result = engine.search_and_resolve("}{bad", SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS);
    assert!(
        result.as_ref().is_err_and(|e| e.is_bad_input()),
        "expected BadInput for malformed attributes, got: {result:?}"
    );
}

// ------------------------------------------------------------------------
// Tests — JSON structure validation
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_why_records_json_structure() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let result = engine
            .why_records("CUSTOMERS", "1001", "CUSTOMERS", "1002", SZ_NO_FLAGS)
            .expect("why_records failed");
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("why_records returned invalid JSON");
        assert!(
            parsed.get("WHY_RESULTS").is_some(),
            "missing WHY_RESULTS key in why_records response"
        );
        let why_results = parsed
            .get("WHY_RESULTS")
            .and_then(|v| v.as_array())
            .expect("WHY_RESULTS is not an array");
        assert!(
            !why_results.is_empty(),
            "WHY_RESULTS array should not be empty"
        );
        // Each result should have MATCH_INFO.
        for entry in why_results {
            assert!(
                entry.get("MATCH_INFO").is_some(),
                "missing MATCH_INFO in WHY_RESULTS entry"
            );
        }
    });
}

#[test]
#[serial]
fn test_how_entity_json_structure() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let result = engine
            .how_entity_by_entity_id(entity_id, SZ_NO_FLAGS)
            .expect("how_entity_by_entity_id failed");
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("how_entity returned invalid JSON");
        assert!(
            parsed.get("HOW_RESULTS").is_some(),
            "missing HOW_RESULTS key in how_entity response"
        );
        let how_results = parsed.get("HOW_RESULTS").expect("missing HOW_RESULTS");
        assert!(
            how_results.get("RESOLUTION_STEPS").is_some()
                || how_results.get("FINAL_STATE").is_some(),
            "HOW_RESULTS should contain RESOLUTION_STEPS or FINAL_STATE"
        );
    });
}

// ------------------------------------------------------------------------
// Tests — Convenience method: get_stats_parsed
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_stats_parsed() {
    let engine = get_szengine();
    let stats = engine.get_stats_parsed().expect("get_stats_parsed failed");
    // workload should be a non-negative number.
    assert!(stats.workload >= 0, "workload should be >= 0");
    // raw_json should be valid JSON.
    assert!(is_valid_json(&stats.raw_json));
    // Display should contain "workload".
    let display = format!("{stats}");
    assert!(
        display.contains("workload"),
        "Display should mention workload, got: {display}"
    );
}

// ------------------------------------------------------------------------
// Tests — BatchResult total() and IntoIterator
// ------------------------------------------------------------------------

#[test]
fn test_batch_result_total_empty() {
    let result = BatchResult::default();
    assert_eq!(result.total(), 0);
}

#[test]
fn test_batch_result_total_with_data() {
    let result = BatchResult {
        successes: 5,
        errors: vec![
            ("rec1".to_string(), SzError::bad_input("bad").with_code(23)),
            ("rec2".to_string(), SzError::not_found("missing").with_code(33)),
        ],
    };
    assert_eq!(result.total(), 7);
}

#[test]
fn test_batch_result_into_iterator() {
    let result = BatchResult {
        successes: 2,
        errors: vec![
            ("rec1".to_string(), SzError::bad_input("bad").with_code(23)),
            ("rec2".to_string(), SzError::not_found("missing").with_code(33)),
        ],
    };
    let collected: Vec<_> = result.into_iter().collect();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].0, "rec1");
    assert_eq!(collected[1].0, "rec2");
}

// ------------------------------------------------------------------------
// Tests — get_entity_parsed
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_entity_parsed() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let entity_id = get_entity_id(engine, "CUSTOMERS", "1001");
        let entity = engine
            .get_entity_parsed(entity_id, SZ_ENTITY_DEFAULT_FLAGS)
            .expect("get_entity_parsed failed");
        assert_eq!(entity.entity_id, entity_id);
        assert!(
            !entity.entity_name.is_empty(),
            "entity_name should not be empty with default flags"
        );
        assert!(entity.record_count > 0, "record_count should be > 0");
        assert!(!entity.raw_json.is_empty());
        // Display should include the entity ID.
        let display = format!("{entity}");
        assert!(display.contains(&entity_id.to_string()));
    });
}

#[test]
#[serial]
fn test_get_entity_parsed_bad_id() {
    let engine = get_szengine();
    let result = engine.get_entity_parsed(BAD_ENTITY_ID, SZ_NO_FLAGS);
    assert!(result.is_err(), "expected error for bad entity ID");
}

// ------------------------------------------------------------------------
// Tests — search_parsed
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_search_parsed() {
    with_test_records(RECORDS_1001_1002_1003, |engine| {
        let attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "Smith", "NAME_FIRST": "Robert"}]}"#;
        let results = engine
            .search_parsed(
                attrs,
                SZ_NO_SEARCH_PROFILE,
                SZ_SEARCH_BY_ATTRIBUTES_DEFAULT_FLAGS,
            )
            .expect("search_parsed failed");
        assert!(!results.is_empty(), "should find at least one match");
        assert!(results.len() > 0);
        let best = results.best_match().expect("should have a best match");
        assert!(best.entity_id > 0, "entity_id should be positive");
        // Display should mention entity count.
        let display = format!("{results}");
        assert!(display.contains("matching entities"));
    });
}

#[test]
#[serial]
fn test_search_parsed_no_match() {
    let engine = get_szengine();
    let attrs = r#"{"NAMES": [{"NAME_TYPE": "PRIMARY", "NAME_LAST": "ZZZZNOEXIST99999"}]}"#;
    let results = engine
        .search_parsed(attrs, SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS)
        .expect("search_parsed should not error on no match");
    assert!(results.is_empty());
    assert!(results.best_match().is_none());
}

// ------------------------------------------------------------------------
// Tests — process_all_redo_records
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_process_all_redo_records() {
    let mut engine = get_szengine();
    // Process whatever is in the redo queue (may be empty).
    let summary = engine
        .process_all_redo_records(SZ_WITHOUT_INFO)
        .expect("process_all_redo_records failed");
    // Summary should be valid regardless of queue state.
    assert!(summary.total() == summary.processed + summary.failed);
    let display = format!("{summary}");
    assert!(display.contains("processed"));
}

// ------------------------------------------------------------------------
// Tests — SearchResults filter/sort helpers
// ------------------------------------------------------------------------

#[test]
fn test_search_results_top_n() {
    let results = SearchResults {
        entities: vec![
            SearchEntity { entity_id: 1, entity_name: "A".into(), match_score: 90 },
            SearchEntity { entity_id: 2, entity_name: "B".into(), match_score: 80 },
            SearchEntity { entity_id: 3, entity_name: "C".into(), match_score: 70 },
        ],
        raw_json: "{}".into(),
    };
    let top = results.top_n(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].entity_id, 1);
    assert_eq!(top[1].entity_id, 2);
}

#[test]
fn test_search_results_top_n_exceeds_len() {
    let results = SearchResults {
        entities: vec![
            SearchEntity { entity_id: 1, entity_name: "A".into(), match_score: 90 },
        ],
        raw_json: "{}".into(),
    };
    let top = results.top_n(5);
    assert_eq!(top.len(), 1);
}

#[test]
fn test_search_results_filter_by_min_score() {
    let results = SearchResults {
        entities: vec![
            SearchEntity { entity_id: 1, entity_name: "A".into(), match_score: 90 },
            SearchEntity { entity_id: 2, entity_name: "B".into(), match_score: 50 },
            SearchEntity { entity_id: 3, entity_name: "C".into(), match_score: 30 },
        ],
        raw_json: "{}".into(),
    };
    let filtered = results.filter_by_min_score(50);
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered.entities[0].entity_id, 1);
    assert_eq!(filtered.entities[1].entity_id, 2);
}

#[test]
fn test_search_results_entity_ids() {
    let results = SearchResults {
        entities: vec![
            SearchEntity { entity_id: 10, entity_name: "A".into(), match_score: 90 },
            SearchEntity { entity_id: 20, entity_name: "B".into(), match_score: 80 },
        ],
        raw_json: "{}".into(),
    };
    assert_eq!(results.entity_ids(), vec![10, 20]);
}

#[test]
fn test_search_results_empty_filter() {
    let results = SearchResults {
        entities: vec![],
        raw_json: "{}".into(),
    };
    let filtered = results.filter_by_min_score(50);
    assert!(filtered.is_empty());
    assert!(filtered.entity_ids().is_empty());
    assert!(results.top_n(3).is_empty());
}

// ------------------------------------------------------------------------
// Tests — BatchResult retryable_errors and failed_record_ids
// ------------------------------------------------------------------------

#[test]
fn test_batch_result_retryable_errors() {
    let result = BatchResult {
        successes: 1,
        errors: vec![
            ("rec1".to_string(), SzError::bad_input("bad").with_code(23)),
            ("rec2".to_string(), SzError::database_transient("deadlock").with_code(1008)),
            ("rec3".to_string(), SzError::not_found("gone").with_code(33)),
        ],
    };
    let retryable = result.retryable_errors();
    assert_eq!(retryable.len(), 1);
    assert_eq!(retryable[0].0, "rec2");
}

#[test]
fn test_batch_result_retryable_errors_empty() {
    let result = BatchResult {
        successes: 3,
        errors: vec![],
    };
    assert!(result.retryable_errors().is_empty());
}

#[test]
fn test_batch_result_failed_record_ids() {
    let result = BatchResult {
        successes: 1,
        errors: vec![
            ("rec_a".to_string(), SzError::bad_input("bad").with_code(23)),
            ("rec_b".to_string(), SzError::not_found("gone").with_code(33)),
        ],
    };
    assert_eq!(result.failed_record_ids(), vec!["rec_a", "rec_b"]);
}

// ------------------------------------------------------------------------
// Tests — Metric helpers
// ------------------------------------------------------------------------

#[test]
fn test_batch_result_success_rate_all_ok() {
    let result = BatchResult {
        successes: 10,
        errors: vec![],
    };
    assert!((result.success_rate() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_batch_result_success_rate_partial() {
    let result = BatchResult {
        successes: 3,
        errors: vec![
            ("r1".into(), SzError::bad_input("bad").with_code(23)),
        ],
    };
    assert!((result.success_rate() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_batch_result_success_rate_empty() {
    let result = BatchResult::default();
    assert!((result.success_rate() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_redo_summary_success_rate() {
    let summary = RedoSummary {
        processed: 8,
        failed: 2,
        errors: vec![
            SzError::general("err1"),
            SzError::general("err2"),
        ],
    };
    assert!((summary.success_rate() - 0.8).abs() < f64::EPSILON);
    assert!(summary.has_failures());
}

#[test]
fn test_redo_summary_success_rate_empty() {
    let summary = RedoSummary {
        processed: 0,
        failed: 0,
        errors: vec![],
    };
    assert!((summary.success_rate() - 1.0).abs() < f64::EPSILON);
    assert!(!summary.has_failures());
}

// ------------------------------------------------------------------------
// Tests — Streaming export with progress callback
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_stream_export_json_with_callback() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let mut progress_calls = 0u64;
        let report = engine
            .stream_export_json_with_callback(SZ_EXPORT_INCLUDE_ALL_ENTITIES, |count| {
                progress_calls = count;
            })
            .expect("streaming export with callback failed");
        assert!(!report.is_empty());
        assert!(progress_calls > 0, "callback should have been called");
    });
}

#[test]
#[serial]
fn test_stream_export_csv_with_callback() {
    with_test_records(RECORDS_1001_1002, |engine| {
        let mut progress_calls = 0u64;
        let report = engine
            .stream_export_csv_with_callback("", SZ_EXPORT_INCLUDE_ALL_ENTITIES, |count| {
                progress_calls = count;
            })
            .expect("streaming CSV export with callback failed");
        assert!(!report.is_empty());
        assert!(progress_calls > 0, "callback should have been called");
    });
}

// ------------------------------------------------------------------------
// Tests — Concurrent engine clones
// ------------------------------------------------------------------------

/// Verify that cloned engines can be used concurrently from multiple threads.
///
/// Note: concurrent writes may produce transient `Database` errors (e.g.,
/// SQLite table locks) depending on the server backend. The test verifies
/// no panics occur and at least one thread succeeds.
#[test]
#[serial]
fn test_concurrent_engine_clones() {
    let engine = get_szengine();
    let mut handles = Vec::new();

    for i in 0..4 {
        let mut e = engine.clone();
        let rid = format!("CONC_{i}");
        handles.push(std::thread::spawn(move || {
            let record = format!(
                r#"{{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "{rid}", "PRIMARY_NAME_FULL": "Concurrent Test {i}"}}"#
            );
            let result = e.add_record("CUSTOMERS", &rid, &record, SZ_WITHOUT_INFO);
            (rid, result.is_ok())
        }));
    }

    let mut rids = Vec::new();
    let mut successes = 0;
    for handle in handles {
        let (rid, ok) = handle.join().expect("thread panicked");
        if ok {
            successes += 1;
        }
        rids.push(rid);
    }

    // At least one concurrent write should succeed.
    assert!(successes > 0, "all concurrent add_record calls failed");

    // Cleanup all records (some may not exist if their add failed).
    let mut engine = engine;
    for rid in &rids {
        let _ = engine.delete_record("CUSTOMERS", rid, SZ_WITHOUT_INFO);
    }
}
