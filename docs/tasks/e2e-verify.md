# 任务:e2e-verify - 端到端验证 example

## 目标

新建 `src-tauri/examples/e2e_verify.rs`,从环境变量读 cookie + workspace_id,用 `OpenCodeApiClient` 调真 opencode.ai,打印真实配额。验证 app 代理(`proxy.rs` 读系统代理)+ `reqwest::Proxy` + `fetch_quota` + HTML 解析整条链。

## 改动

### `src-tauri/examples/e2e_verify.rs` - 新建

```rust
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
```

## 验证(omp 只需编译通过)

```bash
cd src-tauri && cargo check --example e2e_verify
```

## 注意

- example 从环境变量读 cookie/workspace_id,**不硬编码**(敏感凭证不入代码)
- `OpenCodeApiClient::new` 会读系统代理(`proxy.rs`),验证代理配置
- example 可入 git(不含凭证),作调试工具
- 真跑验证由 claude code 执行(设环境变量 + cargo run)
