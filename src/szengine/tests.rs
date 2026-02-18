use super::*;
use serial_test::serial;
use sz_sdk::flags::*;
use sz_sdk::parameters::*;
use sz_sdk::SzEngine;

const GRPC_URL: &str = "http://localhost:8261";

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

fn is_valid_json(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

fn get_szengine() -> SzEngineGrpc {
    let channel = crate::runtime::runtime()
        .block_on(async {
            Channel::from_shared(GRPC_URL.to_string())
                .unwrap()
                .connect()
                .await
        })
        .expect("failed to connect to gRPC server");
    SzEngineGrpc::new(channel)
}

/// Look up entity ID for a record that has been added.
fn get_entity_id(engine: &SzEngineGrpc, data_source: &str, record_id: &str) -> i64 {
    let response = engine
        .get_entity_by_record_id(data_source, record_id, SZ_NO_FLAGS)
        .expect("failed to get entity by record id");
    let json: serde_json::Value =
        serde_json::from_str(&response).expect("failed to parse entity JSON");
    json["RESOLVED_ENTITY"]["ENTITY_ID"]
        .as_i64()
        .expect("ENTITY_ID not found in response")
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
    assert!(is_valid_json(result.unwrap()));
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
    assert!(delete_result.is_ok(), "{}", delete_result.as_ref().err().unwrap());
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
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_add_record_bad_record_definition() {
    let mut engine = get_szengine();
    let result = engine.add_record("CUSTOMERS", "9999", BAD_RECORD_DEFINITION, SZ_WITHOUT_INFO);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_delete_record_bad_data_source() {
    let mut engine = get_szengine();
    let result = engine.delete_record(BAD_DATA_SOURCE_CODE, "1001", SZ_WITHOUT_INFO);
    assert!(result.is_err());
}

// ------------------------------------------------------------------------
// Tests — Get record/entity
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_record() {
    let mut engine = get_szengine();
    add_records(&mut engine, &[("CUSTOMERS", "1001", RECORD_1001_JSON)]);
    let result = engine.get_record("CUSTOMERS", "1001", SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(result.unwrap()));
    delete_records(&mut engine, &[("CUSTOMERS", "1001")]);
}

#[test]
#[serial]
fn test_get_record_bad_data_source() {
    let engine = get_szengine();
    let result = engine.get_record(BAD_DATA_SOURCE_CODE, "1001", SZ_NO_FLAGS);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_get_entity_by_record_id() {
    let mut engine = get_szengine();
    add_records(&mut engine, &[("CUSTOMERS", "1001", RECORD_1001_JSON)]);
    let result = engine.get_entity_by_record_id("CUSTOMERS", "1001", SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(result.unwrap()));
    delete_records(&mut engine, &[("CUSTOMERS", "1001")]);
}

#[test]
#[serial]
fn test_get_entity_by_record_id_bad_data_source() {
    let engine = get_szengine();
    let result = engine.get_entity_by_record_id(BAD_DATA_SOURCE_CODE, "1001", SZ_NO_FLAGS);
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_get_entity_by_entity_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
        ],
    );
    let entity_id = get_entity_id(&engine, "CUSTOMERS", "1001");
    let result = engine.get_entity_by_entity_id(entity_id, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    assert!(is_valid_json(result.unwrap()));
    delete_records(&mut engine, &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")]);
}

#[test]
#[serial]
fn test_get_entity_by_entity_id_bad_entity_id() {
    let engine = get_szengine();
    let result = engine.get_entity_by_entity_id(BAD_ENTITY_ID, SZ_NO_FLAGS);
    assert!(result.is_err());
}

// ------------------------------------------------------------------------
// Tests — Search & preview
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_search_by_attributes() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let result =
        engine.search_by_attributes(DEFAULT_SEARCH_ATTRIBUTES, SZ_NO_SEARCH_PROFILE, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
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
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
        ],
    );
    let entity_id1 = get_entity_id(&engine, "CUSTOMERS", "1001");
    let entity_id2 = get_entity_id(&engine, "CUSTOMERS", "1002");
    let result = engine.why_entities(entity_id1, entity_id2, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")]);
}

#[test]
#[serial]
fn test_why_records() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
        ],
    );
    let result = engine.why_records("CUSTOMERS", "1001", "CUSTOMERS", "1002", SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")]);
}

#[test]
#[serial]
fn test_why_record_in_entity() {
    let mut engine = get_szengine();
    add_records(&mut engine, &[("CUSTOMERS", "1001", RECORD_1001_JSON)]);
    let result = engine.why_record_in_entity("CUSTOMERS", "1001", SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001")]);
}

#[test]
#[serial]
fn test_why_search() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let entity_id = get_entity_id(&engine, "CUSTOMERS", "1001");
    let result = engine.why_search(
        DEFAULT_SEARCH_ATTRIBUTES,
        entity_id,
        SZ_NO_SEARCH_PROFILE,
        SZ_NO_FLAGS,
    );
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

#[test]
#[serial]
fn test_how_entity_by_entity_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
        ],
    );
    let entity_id = get_entity_id(&engine, "CUSTOMERS", "1001");
    let result = engine.how_entity_by_entity_id(entity_id, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")]);
}

// ------------------------------------------------------------------------
// Tests — Find path/network
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_find_path_by_entity_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
        ],
    );
    let entity_id1 = get_entity_id(&engine, "CUSTOMERS", "1001");
    let entity_id2 = get_entity_id(&engine, "CUSTOMERS", "1002");
    let result = engine.find_path_by_entity_id(
        entity_id1,
        entity_id2,
        2,
        SZ_NO_AVOIDANCE,
        SZ_NO_REQUIRED_DATASOURCES,
        SZ_NO_FLAGS,
    );
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")]);
}

#[test]
#[serial]
fn test_find_path_by_record_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
        ],
    );
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
    delete_records(&mut engine, &[("CUSTOMERS", "1001"), ("CUSTOMERS", "1002")]);
}

#[test]
#[serial]
fn test_find_network_by_entity_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let entity_id1 = get_entity_id(&engine, "CUSTOMERS", "1001");
    let entity_id2 = get_entity_id(&engine, "CUSTOMERS", "1002");
    let entity_ids = format!(
        r#"{{"ENTITIES": [{{"ENTITY_ID": {}}}, {{"ENTITY_ID": {}}}]}}"#,
        entity_id1, entity_id2
    );
    let result = engine.find_network_by_entity_id(&entity_ids, 2, 2, 10, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

#[test]
#[serial]
fn test_find_network_by_record_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let record_keys = r#"{"RECORDS": [{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1001"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1002"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1003"}]}"#;
    let result = engine.find_network_by_record_id(record_keys, 2, 2, 10, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

#[test]
#[serial]
fn test_find_interesting_entities_by_entity_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let entity_id = get_entity_id(&engine, "CUSTOMERS", "1001");
    let result = engine.find_interesting_entities_by_entity_id(entity_id, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

#[test]
#[serial]
fn test_find_interesting_entities_by_record_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let result = engine.find_interesting_entities_by_record_id("CUSTOMERS", "1001", SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

// ------------------------------------------------------------------------
// Tests — Virtual entity
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_get_virtual_entity_by_record_id() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
    let record_keys = r#"{"RECORDS": [{"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1001"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1002"}, {"DATA_SOURCE": "CUSTOMERS", "RECORD_ID": "1003"}]}"#;
    let result = engine.get_virtual_entity_by_record_id(record_keys, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

// ------------------------------------------------------------------------
// Tests — Reevaluate
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_reevaluate_entity() {
    let mut engine = get_szengine();
    add_records(&mut engine, &[("CUSTOMERS", "1001", RECORD_1001_JSON)]);
    let entity_id = get_entity_id(&engine, "CUSTOMERS", "1001");
    let result = engine.reevaluate_entity(entity_id, SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001")]);
}

#[test]
#[serial]
fn test_reevaluate_record() {
    let mut engine = get_szengine();
    add_records(&mut engine, &[("CUSTOMERS", "1001", RECORD_1001_JSON)]);
    let result = engine.reevaluate_record("CUSTOMERS", "1001", SZ_NO_FLAGS);
    assert!(result.is_ok(), "{}", result.as_ref().err().unwrap());
    delete_records(&mut engine, &[("CUSTOMERS", "1001")]);
}

// ------------------------------------------------------------------------
// Tests — Export
// ------------------------------------------------------------------------

#[test]
#[serial]
fn test_export_json_entity_report() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
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
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
    );
}

#[test]
#[serial]
fn test_export_csv_entity_report() {
    let mut engine = get_szengine();
    add_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001", RECORD_1001_JSON),
            ("CUSTOMERS", "1002", RECORD_1002_JSON),
            ("CUSTOMERS", "1003", RECORD_1003_JSON),
        ],
    );
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
    delete_records(
        &mut engine,
        &[
            ("CUSTOMERS", "1001"),
            ("CUSTOMERS", "1002"),
            ("CUSTOMERS", "1003"),
        ],
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
