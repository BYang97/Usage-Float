use serde::Serialize;

/// 单个 session 原始用量(session 表一行)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSessionUsage {
    pub session_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub cost: f64,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub tokens_reasoning: i64,
    pub tokens_cache_read: i64,
    pub tokens_cache_write: i64,
    pub time_created: i64,
}

/// 本地聚合结果(token 来自 SQLite)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalAggregate {
    pub total_tokens: i64,
    pub tokens_7d: i64,
    pub tokens_30d: i64,
    pub total_cost: f64,
    pub daily_history: Vec<DayBucket>,
    pub models: Vec<ModelBreakdown>,
}

/// 单日用量桶(折线图,单位 M token)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayBucket {
    pub date: String,
    pub tokens: f64,
}

/// 按 model 分组用量。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelBreakdown {
    pub name: String,
    pub percentage: f64,
    pub color: String,
}

// ===== opencode.ai Go 套餐配额(从 Go 页面 HTML 解析) =====
// 端点:opencode.ai/workspace/{workspaceId}/go(HTML 页面)
// 数据在 React Server Component flight 序列化里:rollingUsage:$R[N]={usagePercent,resetInSec}

/// 单个配额窗口(rolling/weekly/monthly)。
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiWindow {
    pub usage_percent: f64,
    pub reset_in_sec: i64,
}

/// Go 套餐配额(三窗口 + plan)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiQuota {
    pub five_hour: ApiWindow, // rolling 窗口
    pub weekly: ApiWindow,
    pub monthly: ApiWindow,
    pub plan: Option<String>,
}

/// 账户信息。当前仅 plan 从 Go 页面提取,status/expire 暂缺(端点未确认)。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAccount {
    pub plan: Option<String>,
    pub status: Option<String>,
    pub expire_date: Option<String>,
}

// ===== opencode.ai /_server RPC 用量历史(从 React Flight 数据解析) =====

/// 单条用量历史记录(从 usg_xxx 记录解析)。
#[derive(Debug, Clone, Serialize)]
pub struct UsageHistoryItem {
    pub id: String,
    pub time_created: String,
    pub model: String,
    pub provider: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost: f64,
    pub key_id: Option<String>,
    pub session_id: Option<String>,
}
