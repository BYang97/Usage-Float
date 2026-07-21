// ============================================================
// opencode.ai API 客户端
// 批次 1: 端点待[opencode.ai API 调研]回填后实现。
// 批次 2: 按调研结果实现 fetch_quota / fetch_account。
// ============================================================
// TODO: 以下内容待调研确认后移除此注释块:
//   - 真实端点 URL(如 /api/quota, /api/account)
//   - 请求/响应格式
//   - 认证方式(Cookie auth=<Fe26.2**...>)
//   - 超时设置(建议 10s)
//   - 降级策略(失败读缓存,读不到则返回 None)
//
// pub struct OpenCodeApiClient { cookie: String }
// impl OpenCodeApiClient {
//     pub fn new(cookie: String) -> Self;
//     pub async fn fetch_quota(&self) -> Result<ApiQuota, CollectorError>;
//     pub async fn fetch_account(&self) -> Result<ApiAccount, CollectorError>;
// }
