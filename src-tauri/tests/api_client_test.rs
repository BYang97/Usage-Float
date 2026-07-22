//! opencode.ai API 客户端集成测试。
//!
//! 用 httpmock 建 mock HTTP server,覆盖契约 docs/contract.md 第 8 节 API 场景:
//!   - 有 cookie + API 正常 -> 解析成功
//!   - 无 cookie -> NoCookie(不发请求)
//!   - cookie 失效(401) -> Unauthorized
//!   - API 5xx -> ApiError
//!   - API 超时 -> ApiError
//!   - fetch_account 正常(plan/status/到期)
//!   - fetch_account seat-billing 失败降级(plan/status 仍可用)

use app_lib::collector::api::OpenCodeApiClient;
use app_lib::collector::error::CollectorError;
use httpmock::{Method, MockServer};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn fetch_quota_normal_returns_parsed_quota() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(Method::GET)
            .path("/api/budgets/org")
            .header("Cookie", "auth=test-cookie");
        then.status(200).json_body(json!({
            "limitMicroCents": 500000000,
            "spentMicroCents": 250000000,
            "exceeded": false,
            "resetsAt": "2026-07-22T00:00:00Z"
        }));
    });

    let client = OpenCodeApiClient::with_base_url("test-cookie".into(), server.base_url());
    let quota = client.fetch_quota().await.expect("fetch_quota 应成功");

    // 250000000 / 100000 = 2500;500000000 / 100000 = 5000
    assert_eq!(quota.five_hour.used, Some(2500));
    assert_eq!(quota.five_hour.limit, Some(5000));
    assert_eq!(
        quota.five_hour.reset_at.as_deref(),
        Some("2026-07-22T00:00:00Z")
    );
    // 已知 TODO:三窗口共享单一 org budget 值
    assert_eq!(quota.weekly.used, Some(2500));
    assert_eq!(quota.monthly.used, Some(2500));
    mock.assert_hits(1);
}

#[tokio::test]
async fn fetch_quota_no_cookie_returns_no_cookie_without_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(Method::GET).path("/api/budgets/org");
        then.status(200).json_body(json!({"spentMicroCents": 0}));
    });

    let client = OpenCodeApiClient::with_base_url(String::new(), server.base_url());
    let err = client.fetch_quota().await.unwrap_err();
    assert!(
        matches!(err, CollectorError::NoCookie),
        "应是 NoCookie,实际: {:?}",
        err
    );
    mock.assert_hits(0); // cookie 为空时不该发请求
}

#[tokio::test]
async fn fetch_quota_401_returns_unauthorized() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/budgets/org");
        then.status(401);
    });

    let client = OpenCodeApiClient::with_base_url("expired".into(), server.base_url());
    let err = client.fetch_quota().await.unwrap_err();
    assert!(
        matches!(err, CollectorError::Unauthorized),
        "应是 Unauthorized,实际: {:?}",
        err
    );
}

#[tokio::test]
async fn fetch_quota_500_returns_api_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/budgets/org");
        then.status(500);
    });

    let client = OpenCodeApiClient::with_base_url("c".into(), server.base_url());
    let err = client.fetch_quota().await.unwrap_err();
    assert!(
        matches!(err, CollectorError::ApiError(_)),
        "应是 ApiError,实际: {:?}",
        err
    );
}

#[tokio::test]
async fn fetch_quota_timeout_returns_api_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/budgets/org");
        // mock 延迟 3s,client timeout 1s -> 超时
        then.status(200).delay(Duration::from_secs(3));
    });

    let client = OpenCodeApiClient::with_options("c".into(), server.base_url(), 1);
    let err = client.fetch_quota().await.unwrap_err();
    assert!(
        matches!(err, CollectorError::ApiError(_)),
        "应是 ApiError(超时),实际: {:?}",
        err
    );
}

#[tokio::test]
async fn fetch_account_normal_returns_plan_status_expire() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/billing/status");
        then.status(200).json_body(json!({
            "billingMode": "pay-as-you-go",
            "managedInferenceStatus": "active"
        }));
    });
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/billing/seat-billing");
        then.status(200).json_body(json!({
            "subscription": { "renewalAt": "2026-08-20T00:00:00Z" },
            "currentPeriod": { "endsAt": "2026-08-01T00:00:00Z" }
        }));
    });

    let client = OpenCodeApiClient::with_base_url("c".into(), server.base_url());
    let account = client.fetch_account().await.expect("fetch_account 应成功");
    assert_eq!(account.plan.as_deref(), Some("pay-as-you-go"));
    assert_eq!(account.status.as_deref(), Some("active"));
    // renewalAt 优先于 endsAt
    assert_eq!(account.expire_date.as_deref(), Some("2026-08-20T00:00:00Z"));
}

#[tokio::test]
async fn fetch_account_seat_billing_fails_still_returns_plan() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/billing/status");
        then.status(200).json_body(json!({
            "billingMode": "invoiceable",
            "managedInferenceStatus": "active"
        }));
    });
    server.mock(|when, then| {
        when.method(Method::GET).path("/api/billing/seat-billing");
        then.status(404); // 免费用户可能无订阅
    });

    let client = OpenCodeApiClient::with_base_url("c".into(), server.base_url());
    let account = client
        .fetch_account()
        .await
        .expect("seat-billing 失败不应阻断 account");
    assert_eq!(account.plan.as_deref(), Some("invoiceable"));
    assert_eq!(account.status.as_deref(), Some("active"));
    assert_eq!(account.expire_date, None); // 无到期日
}
