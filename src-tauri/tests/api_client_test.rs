//! opencode.ai Go 套餐配额客户端集成测试。
//!
//! 用 httpmock mock Go 页面 HTML,覆盖:
//!   - 正常解析三窗口 + plan
//!   - 无 cookie / 无 workspace_id
//!   - 401 / 5xx / 超时
//!   - cookie 过期(重定向 sign-in)

use app_lib::collector::api::OpenCodeApiClient;
use app_lib::collector::error::CollectorError;
use httpmock::{Method, MockServer};
use std::time::Duration;

const WS: &str = "wrk_test123";
const PATH: &str = "/workspace/wrk_test123/go";
const COOKIE: &str = "Fe26.2**test";

/// 真实页面片段(含 rolling/weekly/monthly + plan)。
const HTML_OK: &str = r#"<html>...rollingUsage:$R[31]={status:"ok",resetInSec:7828,usagePercent:1}...weeklyUsage:$R[32]={status:"ok",resetInSec:375487,usagePercent:0}...monthlyUsage:$R[33]={status:"ok",resetInSec:481129,usagePercent:87}...plan:$R[5]="go-monthly"...</html>"#;

#[tokio::test]
async fn fetch_quota_normal_parses_three_windows_and_plan() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET)
            .path(PATH)
            .header("Cookie", "auth=Fe26.2**test");
        then.status(200)
            .header("content-type", "text/html")
            .body(HTML_OK);
    });

    let client = OpenCodeApiClient::with_base_url(
        COOKIE.to_string(),
        WS.to_string(),
        server.base_url(),
    );
    let q = client.fetch_quota().await.expect("fetch_quota 应成功");
    assert_eq!(q.five_hour.usage_percent, 1.0);
    assert_eq!(q.five_hour.reset_in_sec, 7828);
    assert_eq!(q.weekly.usage_percent, 0.0);
    assert_eq!(q.weekly.reset_in_sec, 375487);
    assert_eq!(q.monthly.usage_percent, 87.0);
    assert_eq!(q.monthly.reset_in_sec, 481129);
    assert_eq!(q.plan, Some("go-monthly".to_string()));
}

#[tokio::test]
async fn fetch_quota_no_cookie_returns_no_cookie_without_request() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(Method::GET).path(PATH);
        then.status(200).body(HTML_OK);
    });

    let client = OpenCodeApiClient::with_base_url(
        String::new(),
        WS.to_string(),
        server.base_url(),
    );
    let err = client.fetch_quota().await.unwrap_err();
    assert!(
        matches!(err, CollectorError::NoCookie),
        "应是 NoCookie,实际: {:?}",
        err
    );
    mock.assert_hits(0);
}

#[tokio::test]
async fn fetch_quota_no_workspace_returns_not_found() {
    let server = MockServer::start();
    let client = OpenCodeApiClient::with_base_url(
        COOKIE.to_string(),
        String::new(),
        server.base_url(),
    );
    let err = client.fetch_quota().await.unwrap_err();
    assert!(
        matches!(err, CollectorError::NotFound),
        "应是 NotFound,实际: {:?}",
        err
    );
}

#[tokio::test]
async fn fetch_quota_401_returns_unauthorized() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path(PATH);
        then.status(401);
    });
    let client = OpenCodeApiClient::with_base_url(
        COOKIE.to_string(),
        WS.to_string(),
        server.base_url(),
    );
    let err = client.fetch_quota().await.unwrap_err();
    assert!(matches!(err, CollectorError::Unauthorized));
}

#[tokio::test]
async fn fetch_quota_500_returns_api_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path(PATH);
        then.status(500);
    });
    let client = OpenCodeApiClient::with_base_url(
        COOKIE.to_string(),
        WS.to_string(),
        server.base_url(),
    );
    let err = client.fetch_quota().await.unwrap_err();
    assert!(matches!(err, CollectorError::ApiError(_)));
}

#[tokio::test]
async fn fetch_quota_timeout_returns_api_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path(PATH);
        then.status(200).delay(Duration::from_secs(3));
    });
    let client = OpenCodeApiClient::with_options(
        COOKIE.to_string(),
        WS.to_string(),
        server.base_url(),
        1,
    );
    let err = client.fetch_quota().await.unwrap_err();
    assert!(matches!(err, CollectorError::ApiError(_)));
}

#[tokio::test]
async fn fetch_quota_sign_in_redirect_returns_unauthorized() {
    // cookie 过期:页面重定向到 sign-in,无 rollingUsage
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(Method::GET).path(PATH);
        then.status(200)
            .body(r#"<html>.../sign-in...请重新登录...</html>"#);
    });
    let client = OpenCodeApiClient::with_base_url(
        COOKIE.to_string(),
        WS.to_string(),
        server.base_url(),
    );
    let err = client.fetch_quota().await.unwrap_err();
    assert!(matches!(err, CollectorError::Unauthorized));
}
