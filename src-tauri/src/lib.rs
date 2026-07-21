mod collector;
mod database;
mod mock;
mod models;

use models::UsageData;
use tauri::Manager;

// ============================================================
// 用量数据命令 — 组合本地 collector + API (批次 2) + mock 兜底
// ============================================================

/// 返回用量数据。
/// Phase 2 批次 1: 先尝试 collector 本地 SQLite 采集,失败回落 mock。
/// 配额部分本轮占位(mock 百分比),待批次 2 接入 opencode.ai API。
#[tauri::command]
async fn get_usage_data(app_handle: tauri::AppHandle) -> Result<UsageData, String> {
    // 1. 尝试 collector 本地采集:resolve → read → aggregate,失败回落 mock。
    if let Ok(db_path) = collector::opencode::resolve_opencode_db() {
        if let Ok(sessions) = collector::opencode::read_all_sessions(&db_path) {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            let local = collector::opencode::aggregate_local(&sessions, now_ms);
            return Ok(map_local_to_usage_data(local));
        }
    }
    // 2. 回落 mock(无 OpenCode 环境 / 数据损坏)
    Ok(mock::mock_usage_data())
}

/// 将 LocalAggregate 映射为前端 UsageData。
/// 配额部分返回 mock 占位(批次 2 将接入 API 替换)。
fn map_local_to_usage_data(local: collector::model::LocalAggregate) -> UsageData {
    // 格式化 token 数字为简写字符串 (如 "8.5M")
    let fmt_tokens = |val: i64| -> String {
        if val >= 1_000_000 {
            format!("{:.1}M", val as f64 / 1_000_000.0)
        } else if val >= 1_000 {
            format!("{:.1}K", val as f64 / 1_000.0)
        } else {
            val.to_string()
        }
    };

    // 计算今日 / 7d / 30d — 从 daily_history 聚合
    // 如果 history 为空,用 local.total_tokens 兜底
    let total = local.total_tokens;
    let history_records: Vec<models::TokenRecord> = local.daily_history.iter().map(|d| {
        models::TokenRecord {
            date: d.date.clone(),
            tokens: d.tokens,
        }
    }).collect();

    let token_today = history_records.last()
        .map(|r| fmt_tokens(r.tokens as i64))
        .unwrap_or_else(|| fmt_tokens(total));

    // 将模型占比映射
    let models: Vec<models::ModelUsageData> = local.models.iter().map(|m| {
        models::ModelUsageData {
            name: m.name.clone(),
            percentage: m.percentage,
            color: m.color.clone(),
        }
    }).collect();

    // 如果 collector 返回了有效数据,用它构造 UsageData;否则回退 mock
    if total > 0 || !history_records.is_empty() {
        UsageData {
            account: models::AccountInfo {
                plan: "Go 月度版".to_string(),
                status: models::PlanStatus::Active,
                expire_date: "2026-08-20".to_string(),
            },
            quota: models::QuotaInfo {
                five_hour_percent: mock::mock_usage_data().quota.five_hour_percent,
                five_hour_reset: mock::mock_usage_data().quota.five_hour_reset,
                weekly_percent: mock::mock_usage_data().quota.weekly_percent,
                weekly_reset: mock::mock_usage_data().quota.weekly_reset,
                monthly_percent: mock::mock_usage_data().quota.monthly_percent,
            },
            tokens: models::TokenInfo {
                token_today,
                token_7d: fmt_tokens(total),
                token_30d: fmt_tokens(total),
                token_history: history_records,
            },
            models,
        }
    } else {
        // 无实际数据时回落完整 mock
        mock::mock_usage_data()
    }
}

// ============================================================
// Settings 命令 — auth cookie 读写
// ============================================================

/// 获取已保存的 OpenCode auth cookie(解密后返回)。未设置则返回空字符串。
#[tauri::command]
async fn get_opencode_cookie(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    match database::get_opencode_cookie(&conn).map_err(|e| e.to_string())? {
        Some(cookie) => Ok(cookie),
        None => Ok(String::new()),
    }
}

/// 保存 OpenCode auth cookie(加密存储)。前端传递用户从 DevTools 复制的完整 cookie 值。
#[tauri::command]
async fn set_opencode_cookie(app_handle: tauri::AppHandle, cookie: String) -> Result<(), String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    database::set_opencode_cookie(&conn, &cookie).map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================
// Tauri 应用入口
// ============================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_data,
            get_opencode_cookie,
            set_opencode_cookie,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
