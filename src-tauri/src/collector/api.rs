// ============================================================
// opencode.ai Go 套餐配额客户端
//
// 抓 opencode.ai/workspace/{workspaceId}/go 页面 HTML,正则解析
// React Server Component flight 数据(rollingUsage/weeklyUsage/monthlyUsage/plan)。
// 参考实现: https://github.com/Ruinique/opencode-go-dashboard
// ============================================================

use crate::collector::error::CollectorError;
use crate::collector::model::{ApiQuota, ApiWindow};
use regex::Regex;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://opencode.ai";
const REQUEST_TIMEOUT_SECS: u64 = 15;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

/// opencode.ai Go 套餐配额客户端。
pub struct OpenCodeApiClient {
    cookie: String,
    workspace_id: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenCodeApiClient {
    /// 新建客户端(默认官方域 opencode.ai,15s 超时)。
    /// cookie 为空时 fetch_quota 返回 Err(NoCookie);workspace_id 为空返回 Err(NotFound)。
    pub fn new(cookie: String, workspace_id: String) -> Self {
        Self::with_options(
            cookie,
            workspace_id,
            DEFAULT_BASE_URL.to_string(),
            REQUEST_TIMEOUT_SECS,
        )
    }

    /// 指定 base_url 构造(测试注入 mock server 用),超时用默认。
    pub fn with_base_url(cookie: String, workspace_id: String, base_url: String) -> Self {
        Self::with_options(cookie, workspace_id, base_url, REQUEST_TIMEOUT_SECS)
    }

    /// 全参数构造(测试可控制 timeout 以加速超时场景)。
    pub fn with_options(
        cookie: String,
        workspace_id: String,
        base_url: String,
        timeout_secs: u64,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(USER_AGENT)
            .build()
            .expect("reqwest Client 初始化不应失败");
        Self {
            cookie,
            workspace_id,
            base_url,
            client,
        }
    }

    /// 抓 Go 套餐页面,正则解析 rolling/weekly/monthly 配额 + plan。
    ///
    /// 页面 URL: `{base_url}/workspace/{workspace_id}/go`
    /// 响应是 HTML(React Server Component flight),含:
    ///   rollingUsage:$R[N]={status:"ok",usagePercent:X,resetInSec:Y}
    ///   weeklyUsage:$R[N]={...}
    ///   monthlyUsage:$R[N]={...}
    ///   plan:$R[N]="go-monthly"
    pub async fn fetch_quota(&self) -> Result<ApiQuota, CollectorError> {
        if self.cookie.is_empty() {
            return Err(CollectorError::NoCookie);
        }
        if self.workspace_id.is_empty() {
            return Err(CollectorError::NotFound);
        }

        let url = format!("{}/workspace/{}/go", self.base_url, self.workspace_id);
        let resp = self
            .client
            .get(&url)
            .header("Cookie", format!("auth={}", self.cookie))
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
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

        let html = resp
            .text()
            .await
            .map_err(|e| CollectorError::ParseFailed(format!("读取响应失败: {}", e)))?;

        // 登录态检查:重定向到 sign-in 且无 rollingUsage = cookie 过期
        if html.contains("/sign-in") && !html.contains("rollingUsage") {
            return Err(CollectorError::Unauthorized);
        }

        let rolling = parse_usage(&html, "rollingUsage")?;
        let weekly = parse_usage(&html, "weeklyUsage")?;
        let monthly = parse_usage(&html, "monthlyUsage")?;
        let plan = parse_plan(&html);

        Ok(ApiQuota {
            five_hour: rolling,
            weekly,
            monthly,
            plan,
        })
    }
}

/// 从 HTML 解析指定窗口(rollingUsage/weeklyUsage/monthlyUsage)的 usagePercent + resetInSec。
fn parse_usage(html: &str, key: &str) -> Result<ApiWindow, CollectorError> {
    // 匹配 key:$R[N]={...},提取 {...}
    let pattern = format!("{}:{}", key, r"\$R\[\d+\]=(\{[^}]+\})");
    let obj_re = Regex::new(&pattern)
        .map_err(|e| CollectorError::ParseFailed(format!("正则编译失败: {}", e)))?;
    let obj = obj_re
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| CollectorError::ParseFailed(format!("未找到 {}", key)))?;

    let usage_percent = parse_field_f64(obj, "usagePercent")
        .ok_or_else(|| CollectorError::ParseFailed(format!("{} 无 usagePercent", key)))?;
    let reset_in_sec = parse_field_i64(obj, "resetInSec")
        .ok_or_else(|| CollectorError::ParseFailed(format!("{} 无 resetInSec", key)))?;

    Ok(ApiWindow {
        usage_percent,
        reset_in_sec,
    })
}

/// 从 HTML 解析 plan(plan:$R[N]="go-monthly")。
fn parse_plan(html: &str) -> Option<String> {
    let re = Regex::new(r#"plan:\$R\[\d+\]="([^"]+)""#).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn parse_field_f64(obj: &str, field: &str) -> Option<f64> {
    let pattern = format!("{}:{}", field, r"(\d+(?:\.\d+)?)");
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
}

fn parse_field_i64(obj: &str, field: &str) -> Option<i64> {
    let pattern = format!("{}:{}", field, r"(\d+)");
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<i64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_usage_extracts_rolling() {
        let html = r#"rollingUsage:$R[31]={status:"ok",resetInSec:7828,usagePercent:1}"#;
        let w = parse_usage(html, "rollingUsage").unwrap();
        assert_eq!(w.usage_percent, 1.0);
        assert_eq!(w.reset_in_sec, 7828);
    }

    #[test]
    fn parse_usage_extracts_monthly() {
        let html = r#"monthlyUsage:$R[33]={status:"ok",resetInSec:481129,usagePercent:87}"#;
        let w = parse_usage(html, "monthlyUsage").unwrap();
        assert_eq!(w.usage_percent, 87.0);
        assert_eq!(w.reset_in_sec, 481129);
    }

    #[test]
    fn parse_usage_extracts_weekly_zero() {
        let html = r#"weeklyUsage:$R[32]={status:"ok",resetInSec:375487,usagePercent:0}"#;
        let w = parse_usage(html, "weeklyUsage").unwrap();
        assert_eq!(w.usage_percent, 0.0);
        assert_eq!(w.reset_in_sec, 375487);
    }

    #[test]
    fn parse_plan_extracts() {
        let html = r#"plan:$R[5]="go-monthly""#;
        assert_eq!(parse_plan(html), Some("go-monthly".to_string()));
    }

    #[test]
    fn parse_usage_missing_returns_err() {
        let html = "no usage here";
        assert!(parse_usage(html, "rollingUsage").is_err());
    }

    #[test]
    fn parse_usage_from_real_page_fragment() {
        // 模拟真实页面片段(含多个 $R)
        let html = r#"<html>...rollingUsage:$R[31]={status:"ok",resetInSec:7828,usagePercent:1}...weeklyUsage:$R[32]={status:"ok",resetInSec:375487,usagePercent:0}...monthlyUsage:$R[33]={status:"ok",resetInSec:481129,usagePercent:87}...plan:$R[5]="go-monthly"..."#;
        let rolling = parse_usage(html, "rollingUsage").unwrap();
        let weekly = parse_usage(html, "weeklyUsage").unwrap();
        let monthly = parse_usage(html, "monthlyUsage").unwrap();
        assert_eq!(rolling.usage_percent, 1.0);
        assert_eq!(weekly.usage_percent, 0.0);
        assert_eq!(monthly.usage_percent, 87.0);
        assert_eq!(parse_plan(html), Some("go-monthly".to_string()));
    }
}
