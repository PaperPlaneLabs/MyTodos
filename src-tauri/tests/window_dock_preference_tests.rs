mod common;

use common::*;
use my_todos_lib::commands::{get_saved_window_dock_preference, set_saved_window_dock_preference};
use my_todos_lib::error::AppError;

#[test]
fn test_saved_window_dock_preference_defaults_to_none() {
    let db = setup_test_db();

    let dock_preference = get_saved_window_dock_preference(&db).unwrap();

    assert_eq!(dock_preference, None);
}

#[test]
fn test_saved_window_dock_preference_persists_and_updates() {
    let db = setup_test_db();

    let saved_left = set_saved_window_dock_preference(&db, "left").unwrap();
    assert_eq!(saved_left, "left");
    assert_eq!(
        get_saved_window_dock_preference(&db).unwrap(),
        Some("left".to_string())
    );

    let saved_center = set_saved_window_dock_preference(&db, "center").unwrap();
    assert_eq!(saved_center, "center");
    assert_eq!(
        get_saved_window_dock_preference(&db).unwrap(),
        Some("center".to_string())
    );
}

#[test]
fn test_saved_window_dock_preference_rejects_invalid_values() {
    let db = setup_test_db();

    let result = set_saved_window_dock_preference(&db, "bottom");

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::InvalidInput(message) => {
            assert!(message.contains("Unsupported dock preference"));
        }
        other => panic!("Expected InvalidInput error, got {:?}", other),
    }
}
