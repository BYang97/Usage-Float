use crate::models::{
  AccountInfo, ModelUsageData, PlanStatus, QuotaInfo, TokenInfo, TokenRecord, UsageData,
};

/// Phase 1 占位实现:返回与前端 `web/src/data/mock.ts` 一致的 mock 数据。
/// Phase 2 起由 collector/database 取代;在此之前保证 invoke 通路联通、UI 行为不变。
pub fn mock_usage_data() -> UsageData {
  UsageData {
    account: AccountInfo {
      plan: "Go 月度版".to_string(),
      status: PlanStatus::Active,
      expire_date: "2026-08-20".to_string(),
    },
    quota: QuotaInfo {
      five_hour_percent: 82.0,
      five_hour_reset: "01:42:30".to_string(),
      weekly_percent: 63.0,
      weekly_reset: "周五 09:00".to_string(),
      monthly_percent: 45.0,
    },
    tokens: TokenInfo {
      token_today: "8.5M".to_string(),
      token_7d: "42M".to_string(),
      token_30d: "180M".to_string(),
      token_history: vec![
        TokenRecord { date: "周一".to_string(), tokens: 6.2 },
        TokenRecord { date: "周二".to_string(), tokens: 8.1 },
        TokenRecord { date: "周三".to_string(), tokens: 5.4 },
        TokenRecord { date: "周四".to_string(), tokens: 7.8 },
        TokenRecord { date: "周五".to_string(), tokens: 4.2 },
        TokenRecord { date: "周六".to_string(), tokens: 3.5 },
        TokenRecord { date: "周日".to_string(), tokens: 6.9 },
      ],
    },
    models: vec![
      ModelUsageData { name: "GPT 系列".to_string(), percentage: 60.0, color: "#4a9eff".to_string() },
      ModelUsageData { name: "Claude 系列".to_string(), percentage: 40.0, color: "#d97706".to_string() },
    ],
  }
}
