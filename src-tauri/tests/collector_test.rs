use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use app_lib::collector::error::CollectorError;
use app_lib::collector::opencode::{aggregate_local, read_all_sessions, resolve_opencode_db};
use rusqlite::Connection;

// ── env-var serialisation ──────────────────────────────────
static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn lock_env() -> MutexGuard<'static, ()> {
    // If a previous test panicked while holding the lock, recover from poison.
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

/// Snapshot relevant env vars, clear them, return a guard that restores on drop.
struct EnvCleanRoom {
    opencode_db: Option<String>,
    xdg_data_home: Option<String>,
    home: Option<String>,
    userprofile: Option<String>,
    #[allow(dead_code)]
    lock: MutexGuard<'static, ()>,
}

impl EnvCleanRoom {
    fn enter() -> Self {
        let lock = lock_env();
        let opencode_db = std::env::var("OPENCODE_DB").ok();
        let xdg_data_home = std::env::var("XDG_DATA_HOME").ok();
        let home = std::env::var("HOME").ok();
        let userprofile = std::env::var("USERPROFILE").ok();
        for k in &["OPENCODE_DB", "XDG_DATA_HOME", "HOME", "USERPROFILE"] {
            std::env::remove_var(k);
        }
        EnvCleanRoom { opencode_db, xdg_data_home, home, userprofile, lock }
    }
}

impl Drop for EnvCleanRoom {
    fn drop(&mut self) {
        fn restore(key: &str, val: &Option<String>) {
            match val {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        restore("OPENCODE_DB", &self.opencode_db);
        restore("XDG_DATA_HOME", &self.xdg_data_home);
        restore("HOME", &self.home);
        restore("USERPROFILE", &self.userprofile);
    }
}

/// Set OPENCODE_DB to path inside a clean room (env already cleared).
fn set_opencode_db(path: &PathBuf) {
    std::env::set_var("OPENCODE_DB", path.to_str().unwrap());
}

// ── helpers ────────────────────────────────────────────────

fn create_empty_session_db(dir: &tempfile::TempDir) -> PathBuf {
    let path = dir.path().join("opencode.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE session (
            id TEXT PRIMARY KEY,
            model TEXT,
            cost REAL DEFAULT 0,
            tokens_input INTEGER DEFAULT 0,
            tokens_output INTEGER DEFAULT 0,
            tokens_reasoning INTEGER DEFAULT 0,
            tokens_cache_read INTEGER DEFAULT 0,
            tokens_cache_write INTEGER DEFAULT 0,
            time_created INTEGER DEFAULT 0
        );",
    )
    .unwrap();
    path
}

// =====================================================================
//  1. 无 OpenCode 环境
// =====================================================================
#[test]
fn resolve_opencode_db_returns_not_found_when_no_env() {
    let _room = EnvCleanRoom::enter();
    let result = resolve_opencode_db();
    assert!(matches!(result, Err(CollectorError::NotFound)));
}

// =====================================================================
//  2. 数据为空
// =====================================================================
#[test]
fn empty_data_returns_zero_aggregate() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_empty_session_db(&dir);
    set_opencode_db(&path);

    let db_path = resolve_opencode_db().expect("should resolve");
    let sessions = read_all_sessions(&db_path).expect("should read");
    assert!(sessions.is_empty());

    let agg = aggregate_local(&sessions, 1_000_000);
    assert_eq!(agg.total_tokens, 0);
    assert_eq!(agg.total_cost, 0.0);
    assert_eq!(agg.daily_history.len(), 7);
    for day in &agg.daily_history {
        assert_eq!(day.tokens, 0.0);
    }
    assert!(agg.models.is_empty());
}

// =====================================================================
//  3. 数据损坏
// =====================================================================
#[test]
fn non_sqlite_file_causes_open_failed() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("opencode.db");
    std::fs::write(&path, b"this is not a valid SQLite database file").unwrap();
    set_opencode_db(&path);

    let db_path = resolve_opencode_db().expect("should resolve (file exists)");
    // SQLite defers format validation to first query, so we get QueryFailed, not OpenFailed.
    let result = read_all_sessions(&db_path);
    assert!(matches!(result, Err(CollectorError::QueryFailed(_))));
}

#[test]
fn session_table_missing_columns_causes_query_failed() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("opencode.db");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE session (id TEXT PRIMARY KEY, title TEXT);")
            .unwrap();
        conn.execute("INSERT INTO session (id, title) VALUES ('s1', 'orphan')", [])
            .unwrap();
    }
    set_opencode_db(&path);

    let db_path = resolve_opencode_db().expect("should resolve");
    let result = read_all_sessions(&db_path);
    assert!(matches!(result, Err(CollectorError::QueryFailed(_))));
}

// =====================================================================
//  4. 多 session —— 正确累加 token 并按 model id 分组
// =====================================================================
#[test]
fn multi_session_aggregation_groups_by_model() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_empty_session_db(&dir);

    {
        let conn = Connection::open(&path).unwrap();
        let mut stmt = conn
            .prepare(
                "INSERT INTO session (id, model, cost, tokens_input, tokens_output,
                                       tokens_reasoning, tokens_cache_read, tokens_cache_write,
                                       time_created)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .unwrap();

        // gpt-4 sessions: total tokens = (100+50+10+5+2) + (200+100+20+10+5) = 167 + 335 = 502
        stmt.execute(rusqlite::params![
            "s1",
            r#"{"id":"gpt-4","providerID":"openai"}"#,
            0.1f64,
            100i64, 50, 10, 5, 2,
            1000i64
        ])
        .unwrap();
        stmt.execute(rusqlite::params![
            "s2",
            r#"{"id":"gpt-4","providerID":"openai"}"#,
            0.2f64,
            200i64, 100, 20, 10, 5,
            2000i64
        ])
        .unwrap();

        // claude-3 session: tokens = 300+150+30+15+8 = 503
        stmt.execute(rusqlite::params![
            "s3",
            r#"{"id":"claude-3","providerID":"anthropic"}"#,
            0.3f64,
            300i64, 150, 30, 15, 8,
            3000i64
        ])
        .unwrap();
    }

    set_opencode_db(&path);
    let db_path = resolve_opencode_db().expect("should resolve");
    let sessions = read_all_sessions(&db_path).expect("should read");
    assert_eq!(sessions.len(), 3);

    let agg = aggregate_local(&sessions, 1_000_000);
    // grand total: 502 + 503 = 1005
    assert_eq!(agg.total_tokens, 1005);
    assert!((agg.total_cost - 0.6).abs() < f64::EPSILON);

    assert_eq!(agg.models.len(), 2);
    for m in &agg.models {
        match m.name.as_str() {
            "gpt-4" => assert!((m.percentage - 502.0 / 1005.0 * 100.0).abs() < 0.01),
            "claude-3" => assert!((m.percentage - 503.0 / 1005.0 * 100.0).abs() < 0.01),
            _ => panic!("unexpected model name: {}", m.name),
        }
    }
}

// =====================================================================
//  5. 时间窗口分桶
// =====================================================================
#[test]
fn daily_history_buckets_are_correct() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_empty_session_db(&dir);

    let day_ms: i64 = 86_400_000;
    // now_ms corresponds to a Monday midnight; each bucket index 0..7
    // maps to WEEKDAYS_CN[6]=周日, [5]=周六, [4]=周五, [3]=周四, …
    let now_ms: i64 = 7 * day_ms;

    {
        let conn = Connection::open(&path).unwrap();
        let mut stmt = conn
            .prepare(
                "INSERT INTO session (id, model, cost, tokens_input, tokens_output,
                                       tokens_reasoning, tokens_cache_read, tokens_cache_write,
                                       time_created)
                 VALUES (?1, '{}', 0.0, ?2, 0, 0, 0, 0, ?3)",
            )
            .unwrap();

        // today      → offset 0 → 周日(index 6)
        stmt.execute(rusqlite::params!["s_today", 100i64, now_ms])
            .unwrap();
        // yesterday  → offset 1 → 周六(index 5)
        stmt.execute(rusqlite::params!["s_y1", 200i64, now_ms - day_ms])
            .unwrap();
        // 2 days ago → offset 2 → 周五(index 4)
        stmt.execute(rusqlite::params!["s_y2", 400i64, now_ms - 2 * day_ms])
            .unwrap();
        // 3 days ago → offset 3 → 周四(index 3)
        stmt.execute(rusqlite::params!["s_y3", 800i64, now_ms - 3 * day_ms])
            .unwrap();
        // 8 days ago → outside window, should be ignored
        stmt.execute(rusqlite::params!["s_old", 9999i64, now_ms - 8 * day_ms])
            .unwrap();
    }

    set_opencode_db(&path);
    let db_path = resolve_opencode_db().expect("should resolve");
    let sessions = read_all_sessions(&db_path).expect("should read");
    assert_eq!(sessions.len(), 5);

    let agg = aggregate_local(&sessions, now_ms);

    // day_tokens is indexed by offset-from-today, then mapped 1:1 to
    // WEEKDAYS_CN = [周一, 周二, 周三, 周四, 周五, 周六, 周日].
    // offset 0 = today, offset 1 = yesterday, …
    assert_eq!(agg.daily_history[0].tokens, 100.0, "offset 0 → 周一 = today");
    assert_eq!(agg.daily_history[1].tokens, 200.0, "offset 1 → 周二 = yesterday");
    assert_eq!(agg.daily_history[2].tokens, 400.0, "offset 2 → 周三 = 2 days ago");
    assert_eq!(agg.daily_history[3].tokens, 800.0, "offset 3 → 周四 = 3 days ago");
    // remaining days this window (offset 4,5,6) were not written
    assert_eq!(agg.daily_history[4].tokens, 0.0);
    assert_eq!(agg.daily_history[5].tokens, 0.0);
    assert_eq!(agg.daily_history[6].tokens, 0.0);

    // total_tokens is sum of ALL sessions (daily window only affects history buckets)
    assert_eq!(agg.total_tokens, 100 + 200 + 400 + 800 + 9999);
}

// =====================================================================
//  6. model JSON 解析
// =====================================================================
#[test]
fn model_json_parses_correctly() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_empty_session_db(&dir);

    {
        let conn = Connection::open(&path).unwrap();
        let mut stmt = conn
            .prepare("INSERT INTO session (id, model, cost, tokens_input, tokens_output, time_created)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .unwrap();

        stmt.execute(rusqlite::params![
            "s1",
            r#"{"id":"gpt-4","providerID":"openai"}"#,
            0.1f64,
            100i64, 50, 1000i64
        ])
        .unwrap();
        stmt.execute(rusqlite::params![
            "s2",
            r#"{"id":"claude-3","providerID":"anthropic"}"#,
            0.2f64,
            200i64, 100, 2000i64
        ])
        .unwrap();

        // no variant (and no providerID) — must be tolerated
        stmt.execute(rusqlite::params![
            "s3",
            r#"{"id":"deepseek-v3"}"#,
            0.05f64,
            50i64, 25, 3000i64
        ])
        .unwrap();
    }

    set_opencode_db(&path);
    let db_path = resolve_opencode_db().expect("should resolve");
    let sessions = read_all_sessions(&db_path).expect("should read");

    let s1 = sessions.iter().find(|s| s.session_id == "s1").unwrap();
    assert_eq!(s1.model_id, "gpt-4");
    assert_eq!(s1.provider_id, "openai");

    let s2 = sessions.iter().find(|s| s.session_id == "s2").unwrap();
    assert_eq!(s2.model_id, "claude-3");
    assert_eq!(s2.provider_id, "anthropic");

    let s3 = sessions.iter().find(|s| s.session_id == "s3").unwrap();
    assert_eq!(s3.model_id, "deepseek-v3");
    assert_eq!(s3.provider_id, ""); // missing in JSON — empty string
}

// =====================================================================
//  7. model 为 NULL 的 session（当前代码用 String 取 NULL 会失败）
// =====================================================================
#[test]
fn null_model_does_not_crash_and_tokens_are_counted() {
    let _room = EnvCleanRoom::enter();
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_empty_session_db(&dir);

    {
        let conn = Connection::open(&path).unwrap();
        // model = NULL
        conn.execute(
            "INSERT INTO session (id, model, cost, tokens_input, tokens_output, time_created)
             VALUES ('s_null', NULL, 0.1, 500, 300, 1000)",
            [],
        )
        .unwrap();
        // normal row
        conn.execute(
            "INSERT INTO session (id, model, cost, tokens_input, tokens_output, time_created)
             VALUES ('s_normal', '{\"id\":\"gpt-4\",\"providerID\":\"openai\"}', 0.2, 100, 50, 2000)",
            [],
        )
        .unwrap();
    }

    set_opencode_db(&path);
    let db_path = resolve_opencode_db().expect("should resolve");
    let sessions = read_all_sessions(&db_path)
        .expect("NULL model should be tolerated (currently fails — code uses String, not Option<String>)");

    assert_eq!(sessions.len(), 2);
    let null_s = sessions.iter().find(|s| s.session_id == "s_null").unwrap();
    assert_eq!(null_s.model_id, "");
    assert_eq!(null_s.tokens_input, 500);
    assert_eq!(null_s.tokens_output, 300);

    let agg = aggregate_local(&sessions, 100_000);
    assert_eq!(agg.total_tokens, 500 + 300 + 100 + 50);
}
