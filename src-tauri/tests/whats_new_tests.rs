mod common;

use common::setup_test_db;
use my_todos_lib::services::whats_new_service;

#[test]
fn last_seen_version_defaults_to_none() {
    let db = setup_test_db();

    assert_eq!(whats_new_service::get_last_seen_version(&db).unwrap(), None);
}

#[test]
fn last_seen_version_persists_and_updates() {
    let db = setup_test_db();

    whats_new_service::set_last_seen_version(&db, "0.1.65").unwrap();
    assert_eq!(
        whats_new_service::get_last_seen_version(&db).unwrap(),
        Some("0.1.65".to_string())
    );

    whats_new_service::set_last_seen_version(&db, "0.2.0").unwrap();
    assert_eq!(
        whats_new_service::get_last_seen_version(&db).unwrap(),
        Some("0.2.0".to_string())
    );
}

#[test]
fn last_seen_version_rejects_invalid_values() {
    let db = setup_test_db();

    assert!(whats_new_service::set_last_seen_version(&db, "release-latest").is_err());
    assert_eq!(whats_new_service::get_last_seen_version(&db).unwrap(), None);
}
