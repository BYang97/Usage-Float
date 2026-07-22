pub mod collector;
mod database;
mod mock;
mod models;

use collector::model::{ApiAccount, ApiQuota, ApiWindow};
use models::UsageData;
use tauri::Manager;

// ============================================================
// 用量数据命令 - 组合本地 collector + opencode.ai Go 页面(批次 2 重构)+ mock 兜底
// ============================================================

/// 返回用量数据。
///
/// 组合三个数据源:
///   1. 本地 SQLite 采集(token 累计 + 历史 + 模型分布)
///   2. opencode.ai Go 套餐页面(配额百分比 + 重置秒数 + plan,需 auth cookie + workspace_id)
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

    // 2. 尝试 API 配额(需 cookie + workspace_id)
    let (api_quota, api_account) = match fetch_api_quota(&app_handle).await {
        Ok((q, a)) => (Some(q), Some(a)),
        Err(_) => (None, None),
    };

    // 3. 合并 -> UsageData
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

/// 尝试从 opencode.ai Go 页面取配额(带缓存)。
///
/// 优先读本地 quota/account 表缓存(5 分钟 TTL),
/// 缓存未命中或无 cookie/workspace_id 时返回 Err。
async fn fetch_api_quota(
    app_handle: &tauri::AppHandle,
) -> Result<(ApiQuota, ApiAccount), String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取 app 数据目录失败: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;

    // 1. 试读缓存(quota 三窗口 + plan)
    let cached_rolling = database::get_quota_cache(&conn, "five_hour").map_err(|e| e.to_string())?;
    let cached_weekly = database::get_quota_cache(&conn, "weekly").map_err(|e| e.to_string())?;
    let cached_monthly = database::get_quota_cache(&conn, "monthly").map_err(|e| e.to_string())?;
    let cached_plan = database::get_account_cache(&conn).map_err(|e| e.to_string())?;

    if let (Some((rp, rr)), Some((wp, wr)), Some((mp, mr))) =
        (cached_rolling, cached_weekly, cached_monthly)
    {
        let quota = ApiQuota {
            five_hour: ApiWindow { usage_percent: rp, reset_in_sec: rr },
            weekly: ApiWindow { usage_percent: wp, reset_in_sec: wr },
            monthly: ApiWindow { usage_percent: mp, reset_in_sec: mr },
            plan: cached_plan.clone(),
        };
        let account = ApiAccount {
            plan: cached_plan,
            status: None,
            expire_date: None,
        };
        return Ok((quota, account));
    }

    // 2. 缓存未命中 -> 读 cookie + workspace_id + 调 API
    let cookie = database::get_opencode_cookie(&conn)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let workspace_id = database::get_opencode_workspace_id(&conn)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    if cookie.is_empty() {
        return Err("未设置 auth cookie".to_string());
    }
    if workspace_id.is_empty() {
        return Err("未设置 workspace_id".to_string());
    }

    let client = collector::api::OpenCodeApiClient::new(cookie, workspace_id);
    let quota = client.fetch_quota().await
        .map_err(|e| format!("配额查询失败: {}", e))?;

    // 3. 写入缓存
    let _ = database::set_quota_cache(
        &conn,
        "five_hour",
        quota.five_hour.usage_percent,
        quota.five_hour.reset_in_sec,
    );
    let _ = database::set_quota_cache(
        &conn,
        "weekly",
        quota.weekly.usage_percent,
        quota.weekly.reset_in_sec,
    );
    let _ = database::set_quota_cache(
        &conn,
        "monthly",
        quota.monthly.usage_percent,
        quota.monthly.reset_in_sec,
    );
    if let Some(plan) = &quota.plan {
        let _ = database::set_account_cache(&conn, plan);
    }

    let account = ApiAccount {
        plan: quota.plan.clone(),
        status: None,
        expire_date: None,
    };
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
            // status/expire 暂缺(Go 页面不提供),用 mock 兜底
            models::AccountInfo {
                plan: api
                    .plan
                    .clone()
                    .unwrap_or_else(|| mock_data.account.plan.clone()),
                status: models::PlanStatus::Active,
                expire_date: mock_data.account.expire_date.clone(),
            }
        }
        None => mock_data.account.clone(),
    }
}

/// 从 API 窗口构建配额信息,或回退 mock。
fn build_quota_info(
    api_quota: &Option<ApiQuota>,
    mock_data: &UsageData,
) -> models::QuotaInfo {
    match api_quota {
        Some(q) => models::QuotaInfo {
            five_hour_percent: q.five_hour.usage_percent,
            five_hour_reset: format_reset(q.five_hour.reset_in_sec),
            weekly_percent: q.weekly.usage_percent,
            weekly_reset: format_reset(q.weekly.reset_in_sec),
            monthly_percent: q.monthly.usage_percent,
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

/// 将重置秒数格式化为 "Xh Ym" / "Xm" / "-"。
fn format_reset(sec: i64) -> String {
    if sec <= 0 {
        return "-".to_string();
    }
    let h = sec / 3600;
    let m = (sec % 3600) / 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m", m)
    } else {
        format!("{}s", sec)
    }
}

// ============================================================
// Settings 命令 - auth cookie + workspace_id 读写
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

/// 获取已保存的 OpenCode workspace ID(解密后返回)。未设置则返回空字符串。
#[tauri::command]
async fn get_opencode_workspace_id(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    match database::get_opencode_workspace_id(&conn).map_err(|e| e.to_string())? {
        Some(ws) => Ok(ws),
        None => Ok(String::new()),
    }
}

/// 保存 OpenCode workspace ID(加密存储)。格式 wrk_xxx,从 opencode.ai 工作区 URL 获取。
#[tauri::command]
async fn set_opencode_workspace_id(app_handle: tauri::AppHandle, workspace_id: String) -> Result<(), String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    database::set_opencode_workspace_id(&conn, &workspace_id).map_err(|e| e.to_string())?;
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
            get_opencode_workspace_id,
            set_opencode_workspace_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
