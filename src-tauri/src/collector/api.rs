// ============================================================
// opencode.ai Go 套餐配额客户端
//
// 抓 opencode.ai/workspace/{workspaceId}/go 页面 HTML,正则解析
// React Server Component flight 数据(rollingUsage/weeklyUsage/monthlyUsage/plan)。
// 参考实现: https://github.com/Ruinique/opencode-go-dashboard
// ============================================================

use crate::collector::error::CollectorError;
use crate::collector::model::{ApiQuota, ApiWindow, UsageHistoryItem};
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
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(USER_AGENT);

        // 读系统代理(reqwest 不读 Windows 注册表,这里主动读)
        if let Some(proxy_url) = crate::proxy::read_system_proxy() {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        let client = builder.build().expect("reqwest Client 初始化不应失败");
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
    pub async fn fetch_usage_history(&self, cursor: i64) -> Result<Vec<UsageHistoryItem>, CollectorError> {
        if self.cookie.is_empty() {
            return Err(CollectorError::NoCookie);
        }
        if self.workspace_id.is_empty() {
            return Err(CollectorError::NotFound);
        }

        let url = format!("{}/_server", self.base_url);

        let body = serde_json::json!({
            "t": {
                "t": 9,
                "i": 0,
                "l": 2,
                "a": [
                    {"t": 1, "s": self.workspace_id},
                    {"t": 0, "s": cursor.to_string()}
                ],
                "o": 0
            },
            "f": 31,
            "m": []
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Cookie", format!("auth={}", self.cookie))
            .header("Origin", "opencode.ai")
            .header("Referer", "opencode.ai")
            .header("x-server-instance", "server-fn:2")
            .header("x-server-id", "bfd684bfc2e4eed05cd0b518f5e4eafd3f3376e3938abb9e536e7c03df831e5c")
            .json(&body)
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

        let text = resp
            .text()
            .await
            .map_err(|e| CollectorError::ParseFailed(format!("读取响应失败: {}", e)))?;

        parse_usage_history_items(&text)
    }
}

/// 从 React Flight 响应中解析 usg_xxx 记录列表。
fn parse_usage_history_items(response: &str) -> Result<Vec<UsageHistoryItem>, CollectorError> {
    let mut items = Vec::new();
    let bytes = response.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let anchor = b"id:\"usg_";

    while pos + anchor.len() <= len {
        // 找下一个 "id:\"usg_" 锚点
        if let Some(offset) = bytes[pos..].windows(anchor.len()).position(|w| w == anchor) {
            let abs_start = pos + offset;

            // 往回找对象起始 {
            let brace_start = bytes[..abs_start]
                .iter()
                .rposition(|&b| b == b'{')
                .ok_or_else(|| CollectorError::ParseFailed("未找到 usg 对象起始".to_string()))?;

            // 往前匹配完整对象 }
            let mut depth: i32 = 0;
            let mut brace_end = abs_start;
            for i in brace_start..len {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            brace_end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth != 0 {
                return Err(CollectorError::ParseFailed(
                    "usg 对象括号不匹配".to_string(),
                ));
            }

            let obj = &response[brace_start..=brace_end];

            let id = parse_rsc_str(obj, "id").ok_or_else(|| {
                CollectorError::ParseFailed("usg 记录缺少 id".to_string())
            })?;
            let time_created = parse_rsc_date(obj, "timeCreated").ok_or_else(|| {
                CollectorError::ParseFailed("usg 记录缺少 timeCreated".to_string())
            })?;
            let model = parse_rsc_str(obj, "model").unwrap_or_default();
            let provider = parse_rsc_str(obj, "provider").unwrap_or_default();
            let input_tokens = parse_rsc_i64(obj, "inputTokens").unwrap_or(0);
            let output_tokens = parse_rsc_i64(obj, "outputTokens").unwrap_or(0);
            let reasoning_tokens = parse_rsc_i64(obj, "reasoningTokens").unwrap_or(0);
            let cache_read_tokens = parse_rsc_i64(obj, "cacheReadTokens").unwrap_or(0);
            let cost = parse_rsc_f64(obj, "cost").unwrap_or(0.0);
            let key_id = parse_rsc_str(obj, "keyID");
            let session_id = parse_rsc_str(obj, "sessionID");

            items.push(UsageHistoryItem {
                id,
                time_created,
                model,
                provider,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                cache_read_tokens,
                cost,
                key_id,
                session_id,
            });

            pos = brace_end + 1;
        } else {
            break;
        }
    }

    Ok(items)
}

/// 从 RSC 对象中提取字符串字段(field:"value")。
fn parse_rsc_str<'a>(obj: &'a str, field: &str) -> Option<String> {
    let pattern = format!(r#"{}:"([^"]*)""#, regex::escape(field));
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj)?.get(1).map(|m| m.as_str().to_string())
}

/// 从 RSC 对象提取日期字段(field:$R[N]=new Date("...") 或 field:new Date("..."))。
fn parse_rsc_date(obj: &str, field: &str) -> Option<String> {
    let pattern = format!(r#"{}:(?:\$R\[\d+\]=)?new Date\("([^"]+)"\)"#, regex::escape(field));
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj)?.get(1).map(|m| m.as_str().to_string())
}

/// 从 RSC 对象中提取整数 field。
fn parse_rsc_i64(obj: &str, field: &str) -> Option<i64> {
    let pattern = format!(r"{}:(-?\d+)", regex::escape(field));
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj)?.get(1)?.as_str().parse().ok()
}

/// 从 RSC 对象中提取浮点数 field。
fn parse_rsc_f64(obj: &str, field: &str) -> Option<f64> {
    let pattern = format!(r"{}:(-?\d+(?:\.\d+)?)", regex::escape(field));
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj)?.get(1)?.as_str().parse().ok()
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

/// 从 HTML 解析 plan。
/// 优先级:
/// 1. subscriptionPlan:"xxx" (有值时,如 "go-monthly")
/// 2. plan:$R[N]="xxx" (旧格式/参考项目)
/// 3. useBalance:!0 (lite 余额模式) -> "Lite"
fn parse_plan(html: &str) -> Option<String> {
    // 1. subscriptionPlan:"xxx" (真实页面格式,有值时)
    if let Some(c) = Regex::new(r#"subscriptionPlan:"([^"]+)""#).ok()?.captures(html) {
        return Some(c[1].to_string());
    }
    // 2. plan:$R[N]="xxx" (兼容旧格式/参考项目)
    if let Some(c) = Regex::new(r#"plan:\$R\[\d+\]="([^"]+)""#).ok()?.captures(html) {
        return Some(c[1].to_string());
    }
    // 3. lite 余额模式
    if html.contains("useBalance:!0") {
        return Some("Lite".to_string());
    }
    None
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
    fn parse_plan_extracts_old_format() {
        let html = r#"plan:$R[5]="go-monthly""#;
        assert_eq!(parse_plan(html), Some("go-monthly".to_string()));
    }

    #[test]
    fn parse_plan_extracts_subscription_plan() {
        let html = r#"subscriptionPlan:"go-monthly""#;
        assert_eq!(parse_plan(html), Some("go-monthly".to_string()));
    }

    #[test]
    fn parse_plan_extracts_lite_mode() {
        let html = r#"subscriptionPlan:null,...useBalance:!0"#;
        assert_eq!(parse_plan(html), Some("Lite".to_string()));
    }

    #[test]
    fn parse_plan_extracts_priority_subscription_over_lite() {
        let html = r#"subscriptionPlan:"go-monthly",useBalance:!0"#;
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
