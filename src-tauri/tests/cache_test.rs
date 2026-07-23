use app_lib::database;
use rusqlite::Connection;
use std::path::Path;
use tempfile::TempDir;

/// Create a temporary database with initialized schema.
/// Returns the TempDir (keep alive for the test scope) and the Connection.
fn open_temp_db() -> (TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    database::init_schema(&db_path).unwrap();
    let conn = Connection::open(&db_path).unwrap();
    (dir, conn)
}

// =====================================================================
//  1. set_quota_cache / get_quota_cache — single window
// =====================================================================
#[test]
fn set_quota_cache_single_window() {
    let (_dir, conn) = open_temp_db();
    database::set_quota_cache(&conn, "five_hour", 87.0, 481129).unwrap();
    let result = database::get_quota_cache(&conn, "five_hour").unwrap();
    assert_eq!(result, Some((87.0, 481129)));
}

// =====================================================================
//  2. set_quota_cache / get_quota_cache — three windows
// =====================================================================
#[test]
fn set_quota_cache_multiple_windows() {
    let (_dir, conn) = open_temp_db();
    database::set_quota_cache(&conn, "five_hour", 87.0, 481129).unwrap();
    database::set_quota_cache(&conn, "weekly", 50.0, 604800).unwrap();
    database::set_quota_cache(&conn, "monthly", 30.0, 2592000).unwrap();

    assert_eq!(
        database::get_quota_cache(&conn, "five_hour").unwrap(),
        Some((87.0, 481129))
    );
    assert_eq!(
        database::get_quota_cache(&conn, "weekly").unwrap(),
        Some((50.0, 604800))
    );
    assert_eq!(
        database::get_quota_cache(&conn, "monthly").unwrap(),
        Some((30.0, 2592000))
    );
}

// =====================================================================
//  3. get_quota_cache — nonexistent window returns None
// =====================================================================
#[test]
fn get_quota_cache_nonexistent_window() {
    let (_dir, conn) = open_temp_db();
    let result = database::get_quota_cache(&conn, "nonexistent").unwrap();
    assert_eq!(result, None);
}

// =====================================================================
//  4. set_account_cache / get_account_cache
// =====================================================================
#[test]
fn set_and_get_account_cache() {
    let (_dir, conn) = open_temp_db();
    database::set_account_cache(&conn, "go-monthly").unwrap();
    let result = database::get_account_cache(&conn).unwrap();
    assert_eq!(result, Some("go-monthly".to_string()));
}

// =====================================================================
//  5. get_account_cache — no cache returns None
// =====================================================================
#[test]
fn get_account_cache_nonexistent() {
    let (_dir, conn) = open_temp_db();
    let result = database::get_account_cache(&conn).unwrap();
    assert_eq!(result, None);
}
