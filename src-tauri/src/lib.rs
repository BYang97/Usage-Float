pub mod collector;
mod database;
mod mock;
mod models;

use collector::model::{ApiAccount, ApiQuota};
use models::UsageData;
use tauri::Manager;

// ============================================================
// 用量数据命令 — 组合本地 collector + API (批次 2) + mock 兜底
// ============================================================

/// 返回用量数据。
///
/// 组合三个数据源:
///   1. 本地 SQLite 采集(token 累计 + 历史 + 模型分布)
///   2. opencode.ai API(配额 + 账户信息,需 auth cookie)
///   3. mock 兜底(任何采集失败时)
#[tauri::command]
async fn get_usage_data(app_handle: tauri::AppHandle) -> Result<UsageData, String> {
    // 1. 本地 SQLite 采集
    let (local, has_local_data) = match resolve_local_data() {
        Some(l) => (l, true),
        None => (collector::model::LocalAggregate {
            total_tokens: 0,
            total_cost: 0.0,
            daily_history: vec![],
            models: vec![],
        }, false),
    };

    // 2. 尝试 API 配额(需 cookie)
    let (api_quota, api_account) = match fetch_api_quota(&app_handle).await {
        Ok((q, a)) => (Some(q), Some(a)),
        Err(_) => (None, None),
    };

    // 3. 合并 → UsageData
    if has_local_data || api_quota.is_some() {
        Ok(map_local_to_usage_data(local, api_quota, api_account))
    } else {
        Ok(mock::mock_usage_data())
    }
}

/// 尝试本地 SQLite 采集,失败返回 None。
fn resolve_local_data() -> Option<collector::model::LocalAggregate> {
    let db_path = collector::opencode::resolve_opencode_db().ok()?;
    let sessions = collector::opencode::read_all_sessions(&db_path).ok()?;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    Some(collector::opencode::aggregate_local(&sessions, now_ms))
}

/// 尝试从 API 取配额与账户信息(带缓存)。
///
/// 优先读本地 quota/account 表缓存(5 分钟 TTL),
/// 缓存未命中或无 cookie 时返回 Err。
async fn fetch_api_quota(
    app_handle: &tauri::AppHandle,
) -> Result<(ApiQuota, ApiAccount), String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取 app 数据目录失败: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;

    // 1. 试读缓存
    if let Ok(Some((used, limit, reset_at))) = database::get_quota_cache(&conn, "five_hour") {
        if let Ok(Some((plan, status, expire_date))) = database::get_account_cache(&conn) {
            let window = collector::model::ApiWindow {
                used: Some(used),
                limit,
                reset_at,
            };
            return Ok((
                collector::model::ApiQuota {
                    five_hour: window.clone(),
                    weekly: window.clone(),
                    monthly: window,
                },
                collector::model::ApiAccount {
                    plan: Some(plan),
                    status: Some(status),
                    expire_date,
                },
            ));
        }
    }

    // 2. 缓存未命中 → 读 cookie + 调 API
    let cookie = database::get_opencode_cookie(&conn)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    if cookie.is_empty() {
        return Err("未设置 auth cookie".to_string());
    }

    let client = collector::api::OpenCodeApiClient::new(cookie);

    let quota = client.fetch_quota().await
        .map_err(|e| format!("配额查询失败: {}", e))?;
    let account = client.fetch_account().await
        .map_err(|e| format!("账户查询失败: {}", e))?;

    // 3. 写入缓存
    for (win_key, win) in [
        ("five_hour", &quota.five_hour),
        ("weekly", &quota.weekly),
        ("monthly", &quota.monthly),
    ] {
        let _ = database::set_quota_cache(
            &conn,
            win_key,
            win.used,
            win.limit,
            win.reset_at.as_deref(),
        );
    }
    if let (Some(plan), Some(status)) = (&account.plan, &account.status) {
        let _ = database::set_account_cache(&conn, plan, status, account.expire_date.as_deref());
    }

    Ok((quota, account))
}

/// 将 LocalAggregate + 可选 API 数据映射为前端 UsageData。
fn map_local_to_usage_data(
    local: collector::model::LocalAggregate,
    api_quota: Option<collector::model::ApiQuota>,
    api_account: Option<collector::model::ApiAccount>,
) -> UsageData {
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

    // 配额:优先从 API 取,失败时用 mock 占位
    let mock_data = mock::mock_usage_data();
    let quota = build_quota_info(&api_quota, &mock_data);
    let account = build_account_info(&api_account, &mock_data);

    // 只要有任一数据源有效,就构造真实 UsageData
    if total > 0 || !history_records.is_empty() || api_quota.is_some() {
        UsageData {
            account,
            quota,
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
        mock_data
    }
}

/// 从 API 或 mock 构建账户信息。
fn build_account_info(
    api_account: &Option<ApiAccount>,
    mock_data: &UsageData,
) -> models::AccountInfo {
    match api_account {
        Some(api) => {
            let status = match api.status.as_deref() {
                Some("active" | "plan-required") => models::PlanStatus::Active,
                Some("suspended" | "credit-exhausted") => models::PlanStatus::Expired,
                _ => models::PlanStatus::Active,
            };
            models::AccountInfo {
                plan: api.plan.clone().unwrap_or_else(|| mock_data.account.plan.clone()),
                status,
                expire_date: api
                    .expire_date
                    .clone()
                    .unwrap_or_else(|| mock_data.account.expire_date.clone()),
            }
        }
        None => mock_data.account.clone(),
    }
}

/// 从 API 窗口计算配额百分比,或回退 mock。
fn build_quota_info(
    api_quota: &Option<ApiQuota>,
    mock_data: &UsageData,
) -> models::QuotaInfo {
    match api_quota {
        Some(q) => models::QuotaInfo {
            five_hour_percent: calc_percent(&q.five_hour),
            five_hour_reset: q
                .five_hour
                .reset_at
                .as_deref()
                .unwrap_or("—")
                .to_string(),
            weekly_percent: calc_percent(&q.weekly),
            weekly_reset: q
                .weekly
                .reset_at
                .as_deref()
                .unwrap_or("—")
                .to_string(),
            monthly_percent: calc_percent(&q.monthly),
        },
        None => models::QuotaInfo {
            five_hour_percent: mock_data.quota.five_hour_percent,
            five_hour_reset: mock_data.quota.five_hour_reset.clone(),
            weekly_percent: mock_data.quota.weekly_percent,
            weekly_reset: mock_data.quota.weekly_reset.clone(),
            monthly_percent: mock_data.quota.monthly_percent,
        },
    }
}

/// 从 ApiWindow 计算已用百分比。
fn calc_percent(window: &collector::model::ApiWindow) -> f64 {
    match (window.used, window.limit) {
        (Some(u), Some(l)) if l > 0 => (u as f64 / l as f64) * 100.0,
        _ => 0.0,
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
