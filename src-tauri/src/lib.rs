pub mod collector;
pub mod database;
mod mock;
mod models;
pub mod proxy;

use collector::model::{ApiAccount, ApiQuota, ApiWindow, UsageHistoryItem};
use models::UsageData;
use database::{Account, AccountForm};
use tauri::Manager;
use serde::Serialize;
use log::{info, debug};

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
    debug!("fetch_api_quota: start");
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
        debug!("fetch_api_quota: cache hit, returning cached data");
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

    debug!("fetch_api_quota: cache miss, calling API");
    let client = collector::api::OpenCodeApiClient::new(cookie, workspace_id);
    let quota = client.fetch_quota().await
        .map_err(|e| format!("配额查询失败: {}", e))?;
    info!("fetch_api_quota: API success, plan={:?}", quota.plan);

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
// 多账号管理命令 - 账号 CRUD + 配额刷新
// ============================================================

/// refresh_one 返回的单账号配额结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageResult {
    pub plan: Option<String>,
    pub five_hour_percent: f64,
    pub five_hour_reset: i64,
    pub weekly_percent: f64,
    pub weekly_reset: i64,
    pub monthly_percent: f64,
    pub monthly_reset: i64,
}

/// refresh_all 返回的带配额账号条目。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWithUsage {
    pub account: Account,
    pub usage: Option<UsageResult>,
}

/// 列出全部账号。
#[tauri::command]
async fn list_accounts(app_handle: tauri::AppHandle) -> Result<Vec<Account>, String> {
    info!("list_accounts");
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    database::list_accounts(&conn).map_err(|e| e.to_string())
}

/// 创建新账号。
#[tauri::command]
async fn create_account(app_handle: tauri::AppHandle, form: AccountForm) -> Result<Account, String> {
    info!("create_account: name={:?}", form.name);
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    database::create_account(&conn, form).map_err(|e| e.to_string())
}

/// 更新指定账号。auth_cookie 留空则保持原值不变。
#[tauri::command]
async fn update_account(
    app_handle: tauri::AppHandle,
    id: String,
    form: AccountForm,
) -> Result<Account, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    database::update_account(&conn, &id, form)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("账号 {} 不存在", id))
}

/// 删除指定账号。账号不存在返回错误。
#[tauri::command]
async fn delete_account(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    let deleted = database::delete_account(&conn, &id).map_err(|e| e.to_string())?;
    if deleted {
        Ok(())
    } else {
        Err(format!("账号 {} 不存在", id))
    }
}

/// 刷新单个账号的配额。按 account_id 取 cookie+workspace_id,调 opencode.ai API。
#[tauri::command]
async fn refresh_one(app_handle: tauri::AppHandle, account_id: String) -> Result<UsageResult, String> {
    info!("refresh_one: account_id={}", account_id);
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    let account = database::get_account(&conn, &account_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("账号 {} 不存在", account_id))?;

    let client = collector::api::OpenCodeApiClient::new(account.auth_cookie, account.workspace_id);
    let quota = client.fetch_quota().await
        .map_err(|e| format!("配额刷新失败: {}", e))?;

    Ok(UsageResult {
        plan: quota.plan,
        five_hour_percent: quota.five_hour.usage_percent,
        five_hour_reset: quota.five_hour.reset_in_sec,
        weekly_percent: quota.weekly.usage_percent,
        weekly_reset: quota.weekly.reset_in_sec,
        monthly_percent: quota.monthly.usage_percent,
        monthly_reset: quota.monthly.reset_in_sec,
    })
}

/// 刷新全部账号的配额。逐个调用 refresh_one,失败账号 usage 为 None。
#[tauri::command]
async fn refresh_all(app_handle: tauri::AppHandle) -> Result<Vec<AccountWithUsage>, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    let accounts = database::list_accounts(&conn).map_err(|e| e.to_string())?;

    let mut results = Vec::with_capacity(accounts.len());
    for acc in accounts {
        let usage = match collector::api::OpenCodeApiClient::new(
            acc.auth_cookie.clone(),
            acc.workspace_id.clone(),
        ).fetch_quota().await {
            Ok(quota) => Some(UsageResult {
                plan: quota.plan,
                five_hour_percent: quota.five_hour.usage_percent,
                five_hour_reset: quota.five_hour.reset_in_sec,
                weekly_percent: quota.weekly.usage_percent,
                weekly_reset: quota.weekly.reset_in_sec,
                monthly_percent: quota.monthly.usage_percent,
                monthly_reset: quota.monthly.reset_in_sec,
            }),
            Err(_) => None,
        };
        results.push(AccountWithUsage { account: acc, usage });
    }
    Ok(results)
}

// ============================================================
// 用量历史命令 - /_server RPC 获取 usg_xxx 记录
// ============================================================

/// 获取指定账号的用量历史(usg_xxx 记录)。
#[tauri::command]
async fn get_usage_history(app_handle: tauri::AppHandle, account_id: String, cursor: i64) -> Result<Vec<UsageHistoryItem>, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let conn = database::open_db(&app_data_dir).map_err(|e| e.to_string())?;
    let account = database::get_account(&conn, &account_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("账号 {} 不存在", account_id))?;

    let client = collector::api::OpenCodeApiClient::new(account.auth_cookie, account.workspace_id);
    let items = client.fetch_usage_history(cursor).await
        .map_err(|e| format!("用量历史查询失败: {}", e))?;

    Ok(items)
}

// ============================================================
// Tauri 应用入口
// ============================================================

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Debug)
                    .targets([
                        tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                        tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: None }),
                        tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    ])
                    .build(),
            )?;

            // 系统托盘:show/hide 主窗口 + 悬浮窗 + 退出
            let _tray = tauri::tray::TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("OpenCode Usage Float")
                .menu(&tauri::menu::Menu::with_items(app, &[
                    &tauri::menu::MenuItem::with_id(app, "show_main", "显示主窗口", true, None::<&str>)?,
                    &tauri::menu::MenuItem::with_id(app, "show_float", "显示悬浮窗", true, None::<&str>)?,
                    &tauri::menu::MenuItem::with_id(app, "hide_float", "隐藏悬浮窗", true, None::<&str>)?,
                    &tauri::menu::PredefinedMenuItem::separator(app)?,
                    &tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
                ])?)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show_main" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "show_float" => {
                            if let Some(w) = app.get_webview_window("float") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "hide_float" => {
                            if let Some(w) = app.get_webview_window("float") {
                                let _ = w.hide();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击托盘图标:切换悬浮窗显示
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("float") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_data,
            get_usage_history,
            get_opencode_cookie,
            set_opencode_cookie,
            get_opencode_workspace_id,
            set_opencode_workspace_id,
            list_accounts,
            create_account,
            update_account,
            delete_account,
            refresh_one,
            refresh_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
