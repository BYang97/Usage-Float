//! 端到端验证:用真 cookie + workspace_id 调 opencode.ai,验证配额获取整条链。
//! 用法(设环境变量后跑):
//!   PowerShell: $env:OPENCODE_COOKIE="Fe26.2..."; $env:OPENCODE_WORKSPACE_ID="wrk_..."
//!   cargo run --example e2e_verify

use app_lib::collector::api::OpenCodeApiClient;

#[tokio::main]
async fn main() {
    let cookie = std::env::var("OPENCODE_COOKIE")
        .expect("请设 OPENCODE_COOKIE 环境变量");
    let workspace_id = std::env::var("OPENCODE_WORKSPACE_ID")
        .expect("请设 OPENCODE_WORKSPACE_ID 环境变量");

    println!("workspace: {}", workspace_id);
    println!("cookie 长度: {}", cookie.len());
    println!("(OpenCodeApiClient::new 会读系统代理 proxy.rs)\n");

    let client = OpenCodeApiClient::new(cookie, workspace_id);
    match client.fetch_quota().await {
        Ok(q) => {
            println!("=== 配额获取成功 ===");
            println!("rolling(5h): {}%, 重置 {}s", q.five_hour.usage_percent, q.five_hour.reset_in_sec);
            println!("weekly:      {}%, 重置 {}s", q.weekly.usage_percent, q.weekly.reset_in_sec);
            println!("monthly:     {}%, 重置 {}s", q.monthly.usage_percent, q.monthly.reset_in_sec);
            println!("plan: {:?}", q.plan);
        }
        Err(e) => eprintln!("fetch_quota 失败: {:?}", e),
    }
}
