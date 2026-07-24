use std::path::PathBuf;

use rusqlite::{Connection, OpenFlags};

use crate::collector::error::CollectorError;
use crate::collector::model::*;

const WEEKDAYS_CN: &[&str] = &["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

/// 按第 1 节优先级解析 OpenCode 本地 db 路径。
///
/// 优先级:
///   1. 环境变量 `OPENCODE_DB`(绝对路径)
///   2. `$XDG_DATA_HOME/opencode/`(Windows: `%USERPROFILE%\.local\share\opencode`)
///   3. `$HOME/.local/share/opencode/`
///   在上述目录中查找 `opencode.db`。
pub fn resolve_opencode_db() -> Result<PathBuf, CollectorError> {
    // 优先级 1: 环境变量 OPENCODE_DB(绝对路径)
    if let Ok(path) = std::env::var("OPENCODE_DB") {
        let p = PathBuf::from(&path);
        if p.is_absolute() && p.exists() {
            return Ok(p);
        }
        // TODO: 若 OPENCODE_DB 为相对路径,需相对 data 目录解析,待确认后实现。
    }

    // 优先级 2: 候选 data 目录,逐个验证存在性(首个有效者用)。
    // Windows 上 HOME 可能是 git-bash 的 unix 风格路径(如 /c/Users/...),
    // Rust 视为无效路径,is_dir() 返回 false,故须继续尝试 USERPROFILE(Windows 原生路径)。
    let candidates: Vec<PathBuf> = [
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(|x| PathBuf::from(x).join("opencode")),
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".local").join("share").join("opencode")),
        std::env::var("USERPROFILE")
            .ok()
            .map(|u| PathBuf::from(u).join(".local").join("share").join("opencode")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for data_dir in candidates {
        if !data_dir.is_dir() {
            continue;
        }
        let db_path = data_dir.join("opencode.db");
        if db_path.exists() {
            return Ok(db_path);
        }
    }

    Err(CollectorError::NotFound)
}

/// 以只读模式打开 OpenCode db,读取 session 表所有行,解析成 RawSessionUsage。
pub fn read_all_sessions(db_path: &PathBuf) -> Result<Vec<RawSessionUsage>, CollectorError> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| CollectorError::OpenFailed(e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, model, cost, tokens_input, tokens_output, \
             tokens_reasoning, tokens_cache_read, tokens_cache_write, time_created \
             FROM session",
        )
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            // model 列可能为 NULL(老数据/异常 session),用 Option 容错。
            let model_json: Option<String> = row.get(1).unwrap_or(None);
            let cost: f64 = row.get(2)?;
            let tokens_input: i64 = row.get(3)?;
            let tokens_output: i64 = row.get(4)?;
            let tokens_reasoning: i64 = row.get(5)?;
            let tokens_cache_read: i64 = row.get(6)?;
            let tokens_cache_write: i64 = row.get(7)?;
            let time_created: i64 = row.get(8)?;

            let model_val: serde_json::Value = match model_json {
                Some(j) => serde_json::from_str(&j).unwrap_or(serde_json::Value::Null),
                None => serde_json::Value::Null,
            };

            let model_id = model_val["id"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let provider_id = model_val["providerID"]
                .as_str()
                .unwrap_or("")
                .to_string();

            Ok(RawSessionUsage {
                session_id: id,
                model_id,
                provider_id,
                cost,
                tokens_input,
                tokens_output,
                tokens_reasoning,
                tokens_cache_read,
                tokens_cache_write,
                time_created,
            })
        })
        .map_err(|e| CollectorError::QueryFailed(e.to_string()))?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|e| CollectorError::ParseFailed(e.to_string()))?);
    }
    Ok(sessions)
}

/// 本地用量聚合:按 model 分组 + 逐日历史(近 7 天)。
pub fn aggregate_local(raw: &[RawSessionUsage], now_ms: i64) -> LocalAggregate {
    let sum_tokens = |s: &RawSessionUsage| -> i64 {
        s.tokens_input
            .saturating_add(s.tokens_output)
            .saturating_add(s.tokens_reasoning)
            .saturating_add(s.tokens_cache_read)
            .saturating_add(s.tokens_cache_write)
    };

    let total_tokens: i64 = raw.iter().map(|s| sum_tokens(s)).sum();
    let total_cost: f64 = raw.iter().map(|s| s.cost).sum();

    let day_ms: i64 = 86_400_000;
    let cutoff_7d = now_ms - 7 * day_ms;
    let cutoff_30d = now_ms - 30 * day_ms;

    let tokens_7d: i64 = raw.iter()
        .filter(|s| s.time_created >= cutoff_7d)
        .map(|s| sum_tokens(s))
        .sum();

    let tokens_30d: i64 = raw.iter()
        .filter(|s| s.time_created >= cutoff_30d)
        .map(|s| sum_tokens(s))
        .sum();

    // --- daily_history: 近 7 天,按周一到周日中文标签 ---
    let today_idx = now_ms / day_ms;
    let mut day_tokens = [0f64; 7];
    for s in raw {
        let d = s.time_created / day_ms;
        let offset = today_idx - d;
        if offset >= 0 && (offset as usize) < 7 {
            let tokens = sum_tokens(s) as f64;
            day_tokens[offset as usize] += tokens;
        }
    }
    let daily_history: Vec<DayBucket> = day_tokens
        .iter()
        .enumerate()
        .map(|(i, &t)| DayBucket {
            date: WEEKDAYS_CN[i].to_string(),
            tokens: t,
        })
        .collect();

    // --- models: 按 model_id 分组 ---
    let mut model_tokens: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for s in raw {
        let model_name = if s.model_id.is_empty() {
            "未知".to_string()
        } else {
            s.model_id.clone()
        };
        *model_tokens.entry(model_name).or_insert(0) += sum_tokens(s);
    }

    let models: Vec<ModelBreakdown> = model_tokens
        .into_iter()
        .map(|(name, tokens)| {
            let pct = if total_tokens > 0 {
                (tokens as f64 / total_tokens as f64) * 100.0
            } else {
                0.0
            };
            let lower = name.to_lowercase();
            let color = if lower.contains("gpt") {
                "#3B82F6"
            } else if lower.contains("claude") {
                "#F97316"
            } else {
                "#06B6D4"
            };
            ModelBreakdown {
                name,
                percentage: pct,
                color: color.to_string(),
            }
        })
        .collect();

    LocalAggregate {
        total_tokens,
        tokens_7d,
        tokens_30d,
        total_cost,
        daily_history,
        models,
    }
}
