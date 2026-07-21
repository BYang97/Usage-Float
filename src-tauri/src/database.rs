use std::path::Path;

use rusqlite::{Connection, params};

use crate::collector::error::CollectorError;

/// 本地数据库文件名。
const DB_FILE_NAME: &str = "usage-float.db";

/// 在 Tauri 应用的本地数据目录下解析 SQLite 数据库文件路径。
pub fn resolve_db_path(app_data_dir: &std::path::PathBuf) -> std::path::PathBuf {
    app_data_dir.join(DB_FILE_NAME)
}

/// 初始化 Phase 2 自用 schema: settings / account / quota / usage 四表。
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

        CREATE TABLE IF NOT EXISTS quota (
            window TEXT PRIMARY KEY,
            used INTEGER,
            limit INTEGER,
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

    Ok(())
}

// ===== Cookie 加密存储 =====
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
