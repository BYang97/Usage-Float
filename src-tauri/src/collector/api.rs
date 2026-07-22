// ============================================================
// opencode.ai API 客户端 - 取真实配额与账户信息
//
// 端点调研结果:
//   Base URL: https://console.opencode.ai
//   Auth:     Cookie: auth=<Fe26.2...>
//   GET /api/budgets/org     -> OrgSpendCheck { limitMicroCents, spentMicroCents, exceeded, resetsAt }
//   GET /api/billing/status  -> BillingStatus { billingMode, balanceMicroCents, ... }
//   GET /api/billing/account -> BillingAccount { orgId, creditLimitMicroCents, ... }
//
// 调研过程见 docs/contract.md §2.
// ============================================================

use crate::collector::error::CollectorError;
use crate::collector::model::{ApiAccount, ApiQuota, ApiWindow};
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://console.opencode.ai";
const REQUEST_TIMEOUT_SECS: u64 = 10;

/// opencode.ai API 客户端。
pub struct OpenCodeApiClient {
    cookie: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenCodeApiClient {
    /// 新建客户端(默认官方域 console.opencode.ai,10s 超时)。
    /// cookie 为空时后续调用返回 Err(NoCookie)。
    pub fn new(cookie: String) -> Self {
        Self::with_options(cookie, DEFAULT_BASE_URL.to_string(), REQUEST_TIMEOUT_SECS)
    }

    /// 指定 base_url 构造(测试注入 mock server 用),超时用默认。
    pub fn with_base_url(cookie: String, base_url: String) -> Self {
        Self::with_options(cookie, base_url, REQUEST_TIMEOUT_SECS)
    }

    /// 全参数构造(测试可控制 timeout 以加速超时场景)。
    pub fn with_options(cookie: String, base_url: String, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("reqwest Client 初始化不应失败");
        Self { cookie, base_url, client }
    }

    /// 取真实配额(预算/已用/重置时间)。
    ///
    /// 调用 `GET /api/budgets/org`,返回组织级预算信息。
    /// TODO: 5h/weekly/monthly 三窗口的精确映射待确认。
    ///   当前返回单一窗口数据(org budget),三窗口均填入相同值,
    ///   待获取更细粒度窗口端点后替换。
    pub async fn fetch_quota(&self) -> Result<ApiQuota, CollectorError> {
        if self.cookie.is_empty() {
            return Err(CollectorError::NoCookie);
        }

        let url = format!("{}/api/budgets/org", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Cookie", format!("auth={}", self.cookie))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    CollectorError::ApiError("请求超时".to_string())
                } else if e.is_connect() {
                    CollectorError::ApiError(format!("网络连接失败: {}", e))
                } else {
                    CollectorError::ApiError(e.to_string())
                }
            })?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CollectorError::Unauthorized);
        }
        if !status.is_success() {
            return Err(CollectorError::ApiError(format!(
                "API 返回 HTTP {}",
                status.as_u16()
            )));
        }

        let budget: OrgSpendCheck = resp.json().await.map_err(|e| {
            CollectorError::ParseFailed(format!("解析预算响应失败: {}", e))
        })?;

        // 将单一窗口映射到三个窗口
        // NOTE: spentMicroCents 是金额(非 token),此处除以 100_000 仅作归一化,
        // 百分比(used/limit)正确,绝对值语义待 TODO 替换为真实 token 端点。
        let used_tokens = budget.spent_micro_cents / 100_000;
        let limit_tokens = budget
            .limit_micro_cents
            .map(|l| l / 100_000);

        let window = ApiWindow {
            used: Some(used_tokens),
            limit: limit_tokens,
            reset_at: budget.resets_at.clone(),
        };

        // TODO: 待获取更具体的 5h/weekly/monthly 窗口字段后替换
        Ok(ApiQuota {
            five_hour: window.clone(),
            weekly: window.clone(),
            monthly: window,
        })
    }

    /// 取账户信息(plan / status / 到期)。
    ///
    /// 先调 `GET /api/billing/status` 取 plan+status,
    /// 再调 `GET /api/billing/seat-billing` 取订阅到期日(renewalAt / period endsAt)。
    /// 后者可失败,不影响 plan/status。
    pub async fn fetch_account(&self) -> Result<ApiAccount, CollectorError> {
        if self.cookie.is_empty() {
            return Err(CollectorError::NoCookie);
        }

        // 1. billing status (plan + status)
        let billing = self.get_billing_status().await?;

        // 2. 尝试 seat-billing 获取到期日
        let expire_date = self.get_expire_date().await.unwrap_or(None);

        Ok(ApiAccount {
            plan: Some(billing.billing_mode),
            status: Some(billing.managed_inference_status),
            expire_date,
        })
    }

    /// GET /api/billing/status
    async fn get_billing_status(&self) -> Result<BillingStatusResponse, CollectorError> {
        let url = format!("{}/api/billing/status", self.base_url);
        let resp = self.send_get(&url).await?;
        resp.json().await.map_err(|e| {
            CollectorError::ParseFailed(format!("解析账单状态失败: {}", e))
        })
    }

    /// GET /api/billing/seat-billing -> 取 subscription.renewalAt 或 period.endsAt。
    async fn get_expire_date(&self) -> Result<Option<String>, CollectorError> {
        let url = format!("{}/api/billing/seat-billing", self.base_url);
        let resp = match self.send_get(&url).await {
            Ok(r) => r,
            Err(_) => return Ok(None), // seat-billing 可能无订阅(免费用户)
        };

        let overview: SeatOverview = match resp.json().await {
            Ok(o) => o,
            Err(_) => return Ok(None),
        };

        // 优先 subscription.renewalAt,其次 period.endsAt
        if let Some(sub) = overview.subscription {
            if let Some(r) = sub.renewal_at {
                if !r.is_empty() {
                    return Ok(Some(r));
                }
            }
        }
        if let Some(period) = overview.current_period {
            if let Some(e) = period.ends_at {
                if !e.is_empty() {
                    return Ok(Some(e));
                }
            }
        }
        Ok(None)
    }

    /// 通用 GET 请求(带 auth cookie + 错误处理)。
    async fn send_get(&self, url: &str) -> Result<reqwest::Response, CollectorError> {
        let resp = self
            .client
            .get(url)
            .header("Cookie", format!("auth={}", self.cookie))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    CollectorError::ApiError("请求超时".to_string())
                } else if e.is_connect() {
                    CollectorError::ApiError(format!("网络连接失败: {}", e))
                } else {
                    CollectorError::ApiError(e.to_string())
                }
            })?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CollectorError::Unauthorized);
        }
        if !status.is_success() {
            return Err(CollectorError::ApiError(format!(
                "API 返回 HTTP {}",
                status.as_u16()
            )));
        }
        Ok(resp)
    }
}

// ============================================================
// API 响应结构体
// ============================================================

/// `GET /api/budgets/org` 响应。
/// 对应 Console 源码中的 OrgSpendCheck / OrgBudgetRule。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrgSpendCheck {
    /// 预算上限(micro-cents)。
    #[serde(default)]
    limit_micro_cents: Option<i64>,
    /// 已花费(micro-cents)。
    #[serde(default)]
    spent_micro_cents: i64,
    /// 是否超限(API 响应字段,当前仅记录供后续使用)。
    #[allow(dead_code)]
    #[serde(default)]
    exceeded: bool,
    /// 重置时间(ISO 8601 字符串)。
    #[serde(default)]
    resets_at: Option<String>,
}

/// `GET /api/billing/status` 响应。
/// 对应 Console 源码中的 BillingStatus。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BillingStatusResponse {
    /// 计费模式: "pay-as-you-go" | "invoiceable"
    #[serde(default)]
    billing_mode: String,
    /// 托管推理状态: active | plan-required | plan-suspended | credit-exhausted | invoice-overdue
    #[serde(default)]
    managed_inference_status: String,
    /// 余额(micro-cents,API 响应字段)。
    #[allow(dead_code)]
    #[serde(default)]
    balance_micro_cents: Option<i64>,
    /// 可用额度(micro-cents,API 响应字段)。
    #[allow(dead_code)]
    #[serde(default)]
    available_micro_cents: Option<i64>,
}

/// `GET /api/billing/seat-billing` 响应(SeatOverview)。
/// 对应 Console 源码中的 SeatOverview,含订阅到期日。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeatOverview {
    #[serde(default)]
    subscription: Option<SeatSubscription>,
    #[serde(default)]
    current_period: Option<SeatPeriod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeatSubscription {
    /// 续订时间(ISO 8601)。
    #[serde(default)]
    renewal_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeatPeriod {
    /// 周期结束时间(ISO 8601)。
    #[serde(default)]
    ends_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 OrgSpendCheck 反序列化。
    #[test]
    fn deserialize_org_spend_check() {
        let json = r#"{
            "limitMicroCents": 500000000,
            "spentMicroCents": 350000000,
            "exceeded": false,
            "resetsAt": "2026-07-22T00:00:00Z"
        }"#;
        let check: OrgSpendCheck = serde_json::from_str(json).unwrap();
        assert_eq!(check.limit_micro_cents, Some(500000000));
        assert_eq!(check.spent_micro_cents, 350000000);
        assert!(!check.exceeded);
        assert_eq!(
            check.resets_at,
            Some("2026-07-22T00:00:00Z".to_string())
        );
    }

    /// 测试 BillingStatusResponse 反序列化。
    #[test]
    fn deserialize_billing_status() {
        let json = r#"{
            "billingMode": "pay-as-you-go",
            "managedInferenceStatus": "active",
            "balanceMicroCents": 100000000,
            "availableMicroCents": 50000000
        }"#;
        let status: BillingStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(status.billing_mode, "pay-as-you-go");
        assert_eq!(status.managed_inference_status, "active");
        assert_eq!(status.balance_micro_cents, Some(100000000));
    }

    /// 带空值字段的 JSON 也能反序列化。
    #[test]
    fn deserialize_partial_data() {
        let json = r#"{
            "spentMicroCents": 100,
            "managedInferenceStatus": "active"
        }"#;

        // 作为 OrgSpendCheck（有默认值字段）
        let check: OrgSpendCheck = serde_json::from_str(json).unwrap();
        assert_eq!(check.spent_micro_cents, 100);
        assert_eq!(check.limit_micro_cents, None);
        assert_eq!(check.resets_at, None);
    }
}
