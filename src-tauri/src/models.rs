use serde::Serialize;

// ============================================================
// 数据模型 — 与 web/src/types/index.ts 的 UsageData 对齐。
// 这是前端唯一会通过 invoke 读取的结构,变更须与前端契约同步。
// 字段命名用 camelCase,匹配 TS 侧;serderename 在结构体上统一设置。
// ============================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRecord {
  pub date: String,
  pub tokens: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelUsageData {
  pub name: String,
  pub percentage: f64,
  pub color: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)] // Expired/Error 在 Phase 1 mock 未使用,Phase 2 collector 会产出。
pub enum PlanStatus {
  Active,
  Expired,
  Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
  pub plan: String,
  pub status: PlanStatus,
  pub expire_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaInfo {
  pub five_hour_percent: f64,
  pub five_hour_reset: String,
  pub weekly_percent: f64,
  pub weekly_reset: String,
  pub monthly_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
  pub token_today: String,
  pub token_7d: String,
  pub token_30d: String,
  pub token_history: Vec<TokenRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageData {
  pub account: AccountInfo,
  pub quota: QuotaInfo,
  pub tokens: TokenInfo,
  pub models: Vec<ModelUsageData>,
}
