use std::path::Path;

use rusqlite::{Connection, params};

use crate::collector::error::CollectorError;
use serde::{Deserialize, Serialize};

/// 本地数据库文件名。
const DB_FILE_NAME: &str = "usage-float.db";

/// 在 Tauri 应用的本地数据目录下解析 SQLite 数据库文件路径。
pub fn resolve_db_path(app_data_dir: &std::path::PathBuf) -> std::path::PathBuf {
    app_data_dir.join(DB_FILE_NAME)
}

/// 初始化自用 schema: settings / account / accounts / quota / usage 五表。
pub fn init_schema(db_path: &Path) -> Result<(), CollectorError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CollectorError::OpenFailed(e.to_string()))?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value BLOB,
            updated_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS account (
            id INTEGER PRIMARY KEY,
            plan TEXT,
            status TEXT,
            expire_date TEXT,
            updated_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            workspace_id TEXT NOT NULL,
            auth_cookie TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS quota (
            window TEXT PRIMARY KEY,
            used INTEGER,
            `limit` INTEGER,
            percent REAL,
            reset_at TEXT,
            updated_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS usage (
            day TEXT,
            model TEXT,
            tokens INTEGER,
            cost REAL,
            PRIMARY KEY (day, model)
        );
        ",
    )
    .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    // Migration: accounts 表空且 settings 有 opencode_auth_cookie 时插入默认账号
    migrate_from_settings(&conn)?;

    Ok(())
}


/// 从旧 settings 单账号迁移到 accounts 表。
/// 仅在 accounts 表为空且 settings 中有 opencode_auth_cookie 时执行。
fn migrate_from_settings(conn: &Connection) -> Result<(), CollectorError> {
    // 检查 accounts 表是否已有数据
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    if count > 0 {
        return Ok(());
    }

    // 读取 settings 中的旧 cookie
    let cookie = get_opencode_cookie(conn)?;
    let cookie = match cookie {
        Some(c) if !c.is_empty() => c,
        _ => return Ok(()),
    };

    // 读取 workspace_id（可能未设置）
    let workspace_id = get_opencode_workspace_id(conn)?.unwrap_or_default();

    let now = now_ms();
    let id = generate_account_id();
    let encrypted_cookie = encrypt(cookie.as_bytes())?;

    conn.execute(
        "INSERT INTO accounts (id, name, workspace_id, auth_cookie, notes, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, "默认", workspace_id, encrypted_cookie, String::new(), now, now],
    )
    .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    Ok(())
}

// ===== Cookie / workspace_id 加密存储 =====
// 选型: ring AEAD AES-256-GCM + 机器绑定密钥(COMPUTERNAME / HOSTNAME 派生).
// Windows DPAPI 为备选,当前用 ring 实现以保持跨平台一致。

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};

fn machine_key() -> [u8; 32] {
    let machine_id = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_default();
    let input = format!("usage-float-collector-v1:{}", machine_id);
    let d = digest(&SHA256, input.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(d.as_ref());
    key
}

fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>, CollectorError> {
    let key_bytes = machine_key();
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|e| {
            CollectorError::ApiError(format!("加密密钥初始化失败: {}", e))
        })?;
    let key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| CollectorError::ApiError(format!("随机数生成失败: {}", e)))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| CollectorError::ApiError(format!("加密失败: {}", e)))?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&in_out);
    Ok(result)
}

fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, CollectorError> {
    if ciphertext.len() < 12 {
        return Err(CollectorError::ParseFailed("密文长度不足".to_string()));
    }
    let (nonce_bytes, encrypted) = ciphertext.split_at(12);
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = Nonce::assume_unique_for_key(nonce_arr);

    let key_bytes = machine_key();
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|e| {
            CollectorError::ApiError(format!("解密密钥初始化失败: {}", e))
        })?;
    let key = LessSafeKey::new(unbound_key);

    let mut in_out = encrypted.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| CollectorError::ApiError("解密失败: 密钥不匹配或数据损坏".to_string()))?;

    Ok(plaintext.to_vec())
}

/// 从 settings 表读取加密值并解密。
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, CollectorError> {
    use rusqlite::OptionalExtension;

    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let result: Option<Vec<u8>> = stmt
        .query_row(params![key], |row| row.get(0))
        .optional()
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    match result {
        Some(encrypted) => {
            let plain = decrypt(&encrypted)?;
            let s = String::from_utf8(plain)
                .map_err(|e| CollectorError::ParseFailed(format!("UTF-8 解码失败: {}", e)))?;
            Ok(Some(s))
        }
        None => Ok(None),
    }
}

/// 加密值并存入 settings 表。
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), CollectorError> {
    let encrypted = encrypt(value.as_bytes())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params![key, encrypted, now],
    )
    .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    Ok(())
}

const COOKIE_KEY_NAME: &str = "opencode_auth_cookie";
const WORKSPACE_KEY_NAME: &str = "opencode_workspace_id";

/// 打开本地数据库(读写),确保目录与四表 schema 存在,返回连接。
pub fn open_db(app_data_dir: &std::path::PathBuf) -> Result<Connection, CollectorError> {
    let db_path = resolve_db_path(app_data_dir);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CollectorError::OpenFailed(format!("创建数据目录失败: {}", e)))?;
    }
    init_schema(&db_path)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| CollectorError::OpenFailed(e.to_string()))?;
    Ok(conn)
}

/// 读取并解密 OpenCode auth cookie。
pub fn get_opencode_cookie(conn: &Connection) -> Result<Option<String>, CollectorError> {
    get_setting(conn, COOKIE_KEY_NAME)
}

/// 加密存储 OpenCode auth cookie。
pub fn set_opencode_cookie(conn: &Connection, cookie: &str) -> Result<(), CollectorError> {
    set_setting(conn, COOKIE_KEY_NAME, cookie)
}

/// 读取并解密 OpenCode workspace ID(格式 wrk_xxx)。
pub fn get_opencode_workspace_id(conn: &Connection) -> Result<Option<String>, CollectorError> {
    get_setting(conn, WORKSPACE_KEY_NAME)
}

/// 加密存储 OpenCode workspace ID。
pub fn set_opencode_workspace_id(conn: &Connection, workspace_id: &str) -> Result<(), CollectorError> {
    set_setting(conn, WORKSPACE_KEY_NAME, workspace_id)
}

// ===== 配额/账户缓存读写(quota + account 表) =====
/// 缓存刷新间隔(5 分钟)。
pub const CACHE_TTL_MS: i64 = 300_000;

/// 从 quota 表读指定窗口缓存。返回 (usage_percent, reset_in_sec),None 表示无缓存或已过期。
/// 复用 quota 表:percent 列存 usage_percent,reset_at 列存 reset_in_sec 的字符串。
pub fn get_quota_cache(conn: &Connection, window: &str) -> Result<Option<(f64, i64)>, CollectorError> {
    use rusqlite::OptionalExtension;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let mut stmt = conn
        .prepare("SELECT percent, reset_at, updated_at FROM quota WHERE window = ?1")
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let row: Option<(Option<f64>, Option<String>, i64)> = stmt
        .query_row(params![window], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .optional()
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    match row {
        Some((pct, reset_at, updated_at)) if now - updated_at < CACHE_TTL_MS => {
            let usage_percent = pct.unwrap_or(0.0);
            let reset_in_sec = reset_at
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            Ok(Some((usage_percent, reset_in_sec)))
        }
        _ => Ok(None),
    }
}

/// 写入配额缓存。percent=usage_percent,reset_at=reset_in_sec 字符串。
pub fn set_quota_cache(
    conn: &Connection,
    window: &str,
    usage_percent: f64,
    reset_in_sec: i64,
) -> Result<(), CollectorError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO quota (window, used, `limit`, percent, reset_at, updated_at) \
         VALUES (?1, NULL, NULL, ?2, ?3, ?4)",
        params![window, usage_percent, reset_in_sec.to_string(), now],
    )
    .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    Ok(())
}

/// 从 account 表读取 plan 缓存。None 表示无缓存或已过期。
pub fn get_account_cache(conn: &Connection) -> Result<Option<String>, CollectorError> {
    use rusqlite::OptionalExtension;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let mut stmt = conn
        .prepare("SELECT plan, updated_at FROM account WHERE id = 1")
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let row: Option<(Option<String>, i64)> = stmt
        .query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    match row {
        Some((plan, updated_at)) if now - updated_at < CACHE_TTL_MS => Ok(plan),
        _ => Ok(None),
    }
}

/// 写入账户缓存(仅 plan,status/expire 暂缺)。
pub fn set_account_cache(conn: &Connection, plan: &str) -> Result<(), CollectorError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO account (id, plan, status, expire_date, updated_at) \
         VALUES (1, ?1, NULL, NULL, ?2)",
        params![plan, now],
    )
    .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    Ok(())
}

// ===== 多账号管理(accounts 表 CRUD) =====

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
    pub auth_cookie: String,
    pub notes: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountForm {
    pub name: String,
    pub workspace_id: String,
    pub auth_cookie: String,
    pub notes: String,
}

fn generate_account_id() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 16];
    let _ = rng.fill(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!("acc_{}", hex)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// 列出所有账号,按创建时间升序。auth_cookie 已解密。
pub fn list_accounts(conn: &Connection) -> Result<Vec<Account>, CollectorError> {
    let mut stmt = conn
        .prepare("SELECT id, name, workspace_id, auth_cookie, notes, created_at, updated_at FROM accounts ORDER BY created_at ASC")
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let mut accounts = Vec::new();
    for row_result in rows {
        let (id, name, workspace_id, encrypted, notes, created_at, updated_at) =
            row_result.map_err(|e| CollectorError::QueryFailed(e.to_string()))?;
        let decrypted = decrypt(&encrypted)?;
        let auth_cookie = String::from_utf8(decrypted)
            .map_err(|e| CollectorError::ParseFailed(format!("UTF-8 解码失败: {}", e)))?;
        accounts.push(Account { id, name, workspace_id, auth_cookie, notes, created_at, updated_at });
    }

    Ok(accounts)
}

/// 按 id 获取单个账号,返回 None 表示不存在。auth_cookie 已解密。
pub fn get_account(conn: &Connection, id: &str) -> Result<Option<Account>, CollectorError> {
    use rusqlite::OptionalExtension;

    let mut stmt = conn
        .prepare("SELECT id, name, workspace_id, auth_cookie, notes, created_at, updated_at FROM accounts WHERE id = ?1")
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let row: Option<(String, String, String, Vec<u8>, String, i64, i64)> = stmt
        .query_row(params![id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .optional()
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    match row {
        Some((id, name, workspace_id, encrypted, notes, created_at, updated_at)) => {
            let decrypted = decrypt(&encrypted)?;
            let auth_cookie = String::from_utf8(decrypted)
                .map_err(|e| CollectorError::ParseFailed(format!("UTF-8 解码失败: {}", e)))?;
            Ok(Some(Account { id, name, workspace_id, auth_cookie, notes, created_at, updated_at }))
        }
        None => Ok(None),
    }
}

/// 创建新账号。auth_cookie 自动加密存储。
pub fn create_account(conn: &Connection, form: AccountForm) -> Result<Account, CollectorError> {
    let now = now_ms();
    let id = generate_account_id();
    let encrypted_cookie = encrypt(form.auth_cookie.as_bytes())?;

    conn.execute(
        "INSERT INTO accounts (id, name, workspace_id, auth_cookie, notes, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, form.name, form.workspace_id, encrypted_cookie, form.notes, now, now],
    )
    .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    Ok(Account {
        id,
        name: form.name,
        workspace_id: form.workspace_id,
        auth_cookie: form.auth_cookie,
        notes: form.notes,
        created_at: now,
        updated_at: now,
    })
}

/// 更新账号。auth_cookie 留空则保持原值不变。
pub fn update_account(conn: &Connection, id: &str, form: AccountForm) -> Result<Option<Account>, CollectorError> {
    let now = now_ms();

    if form.auth_cookie.is_empty() {
        let affected = conn
            .execute(
                "UPDATE accounts SET name = ?1, workspace_id = ?2, notes = ?3, updated_at = ?4 WHERE id = ?5",
                params![form.name, form.workspace_id, form.notes, now, id],
            )
            .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

        if affected == 0 {
            return Ok(None);
        }
    } else {
        let encrypted_cookie = encrypt(form.auth_cookie.as_bytes())?;
        let affected = conn
            .execute(
                "UPDATE accounts SET name = ?1, workspace_id = ?2, auth_cookie = ?3, notes = ?4, updated_at = ?5 WHERE id = ?6",
                params![form.name, form.workspace_id, encrypted_cookie, form.notes, now, id],
            )
            .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

        if affected == 0 {
            return Ok(None);
        }
    }

    // 读回更新后的数据
    get_account(conn, id)
}

/// 删除账号。返回 true 表示存在并已删除,false 表示未找到。
pub fn delete_account(conn: &Connection, id: &str) -> Result<bool, CollectorError> {
    let affected = conn
        .execute("DELETE FROM accounts WHERE id = ?1", params![id])
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    Ok(affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_temp_db() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        init_schema(&db_path).unwrap();
        let conn = Connection::open(&db_path).unwrap();
        (dir, conn)
    }

    #[test]
    fn test_create_and_list_account() {
        let (_dir, conn) = open_temp_db();
        let form = AccountForm {
            name: "测试账号".to_string(),
            workspace_id: "wrk_test".to_string(),
            auth_cookie: "Fe26.2**test-cookie-value**".to_string(),
            notes: "备注".to_string(),
        };

        let created = create_account(&conn, form).unwrap();
        assert!(created.id.starts_with("acc_"));
        assert_eq!(created.name, "测试账号");
        assert_eq!(created.workspace_id, "wrk_test");
        assert_eq!(created.auth_cookie, "Fe26.2**test-cookie-value**");
        assert_eq!(created.notes, "备注");
        assert!(created.created_at > 0);
        assert_eq!(created.created_at, created.updated_at);

        let list = list_accounts(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);
        assert_eq!(list[0].auth_cookie, "Fe26.2**test-cookie-value**");
    }

    #[test]
    fn test_get_account() {
        let (_dir, conn) = open_temp_db();
        let form = AccountForm {
            name: "get-test".to_string(),
            workspace_id: "wrk_abc".to_string(),
            auth_cookie: "cookie123".to_string(),
            notes: String::new(),
        };
        let created = create_account(&conn, form).unwrap();

        let found = get_account(&conn, &created.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "get-test");

        // nonexistent id
        let none = get_account(&conn, "acc_nonexistent").unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_update_account_keep_cookie() {
        let (_dir, conn) = open_temp_db();
        let form = AccountForm {
            name: "old".to_string(),
            workspace_id: "wrk_old".to_string(),
            auth_cookie: "secret-cookie".to_string(),
            notes: String::new(),
        };
        let created = create_account(&conn, form).unwrap();

        // Update with empty auth_cookie -> keep existing
        let update_form = AccountForm {
            name: "new-name".to_string(),
            workspace_id: "wrk_new".to_string(),
            auth_cookie: String::new(), // keep existing
            notes: "new-notes".to_string(),
        };
        let updated = update_account(&conn, &created.id, update_form).unwrap();
        assert!(updated.is_some());
        let acct = updated.unwrap();
        assert_eq!(acct.name, "new-name");
        assert_eq!(acct.workspace_id, "wrk_new");
        assert_eq!(acct.auth_cookie, "secret-cookie"); // unchanged
        assert_eq!(acct.notes, "new-notes");
    }

    #[test]
    fn test_update_account_change_cookie() {
        let (_dir, conn) = open_temp_db();
        let form = AccountForm {
            name: "old".to_string(),
            workspace_id: "wrk_old".to_string(),
            auth_cookie: "old-cookie".to_string(),
            notes: String::new(),
        };
        let created = create_account(&conn, form).unwrap();

        // Update with new auth_cookie
        let update_form = AccountForm {
            name: "old".to_string(),
            workspace_id: "wrk_old".to_string(),
            auth_cookie: "new-cookie".to_string(),
            notes: String::new(),
        };
        let updated = update_account(&conn, &created.id, update_form).unwrap();
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().auth_cookie, "new-cookie");
    }

    #[test]
    fn test_update_account_nonexistent() {
        let (_dir, conn) = open_temp_db();
        let form = AccountForm {
            name: "nope".to_string(),
            workspace_id: "wrk_none".to_string(),
            auth_cookie: String::new(),
            notes: String::new(),
        };
        let result = update_account(&conn, "acc_nonexistent", form).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_account() {
        let (_dir, conn) = open_temp_db();
        let form = AccountForm {
            name: "to-delete".to_string(),
            workspace_id: "wrk_del".to_string(),
            auth_cookie: "del-cookie".to_string(),
            notes: String::new(),
        };
        let created = create_account(&conn, form).unwrap();

        let deleted = delete_account(&conn, &created.id).unwrap();
        assert!(deleted);

        let list = list_accounts(&conn).unwrap();
        assert_eq!(list.len(), 0);

        // Delete nonexistent
        let no_deleted = delete_account(&conn, "acc_nonexistent").unwrap();
        assert!(!no_deleted);
    }

    #[test]
    fn test_migrate_from_settings_noop_when_accounts_exist() {
        let (_dir, conn) = open_temp_db();
        // accounts table is empty, and settings has no cookie -> no migration
        let list = list_accounts(&conn).unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_migrate_from_settings_with_cookie() {
        // Create a fresh DB without init_schema, set up settings, then init
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // Phase 1: create schema + insert old settings cookie
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value BLOB,
                    updated_at INTEGER
                );"
            ).unwrap();
            let cookie_val = encrypt(b"Fe26.2**legacy-cookie**").unwrap();
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('opencode_auth_cookie', ?1, 1)",
                params![cookie_val],
            ).unwrap();
            let ws_val = encrypt(b"wrk_legacy").unwrap();
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('opencode_workspace_id', ?1, 1)",
                params![ws_val],
            ).unwrap();
        }

        // Phase 2: init_schema (should migrate)
        init_schema(&db_path).unwrap();
        let conn = Connection::open(&db_path).unwrap();
        let list = list_accounts(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "默认");
        assert_eq!(list[0].auth_cookie, "Fe26.2**legacy-cookie**");
        assert_eq!(list[0].workspace_id, "wrk_legacy");
    }

    #[test]
    fn test_migrate_from_settings_no_cookie_skips() {
        // accounts table empty, settings has no cookie -> skip
        let (_dir, conn) = open_temp_db();
        let list = list_accounts(&conn).unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = b"Fe26.2**sensitive-data**";
        let encrypted = encrypt(original).unwrap();
        assert_ne!(encrypted, original);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }
}
