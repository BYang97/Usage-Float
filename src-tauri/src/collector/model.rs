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

// ===== opencode.ai API 侧占位结构(第 2 节) =====
// TODO: 待端点确认后回填字段与反序列化逻辑。

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiQuota {
    pub five_hour: ApiWindow,
    pub weekly: ApiWindow,
    pub monthly: ApiWindow,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiWindow {
    pub used: Option<i64>,
    pub limit: Option<i64>,
    pub reset_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAccount {
    pub plan: Option<String>,
    pub status: Option<String>,
    pub expire_date: Option<String>,
}
