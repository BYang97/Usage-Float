# 任务:app-proxy-wire - OpenCodeApiClient 配代理

## 前置

- `app-proxy-read` 已完成(`src-tauri/src/proxy.rs` 的 `read_system_proxy` 可用)。

## 目标

`OpenCodeApiClient` 在构造 reqwest::Client 时读系统代理并配置(`reqwest::Proxy::all`),让 `fetch_quota` 走系统代理。

## 改动

### 1. `src-tauri/src/collector/api.rs` - `with_options` 加代理

当前 `with_options`:
```rust
pub fn with_options(cookie, workspace_id, base_url, timeout_secs) -> Self {
    let client = reqwest::Client::builder()
        .timeout(...)
        .user_agent(...)
        .build().expect(...);
    ...
}
```

改为:builder 阶段读系统代理并配:
```rust
let mut builder = reqwest::Client::builder()
    .timeout(Duration::from_secs(timeout_secs))
    .user_agent(USER_AGENT);

// 读 Windows 系统代理(reqwest 不读注册表,这里主动读)
if let Some(proxy_url) = crate::proxy::read_system_proxy() {
    if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
        builder = builder.proxy(proxy);
    }
}

let client = builder.build().expect("reqwest Client 初始化不应失败");
```

### 2. 测试 - `tests/api_client_test.rs`

- 现有 7 个测试应继续通过(代理可选,读系统代理可能 None,mock server 本地不受影响)。
- 不需要新加代理测试(reqwest::Proxy 难 mock,且代理逻辑简单)。

## 验证

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --all-targets   # 21 测试全过
```

## 注意

- 代理读系统注册表(`with_options` 内调 `crate::proxy::read_system_proxy()`),生产 `new()` 走 `with_options` 自动配。
- 代理配置失败不 panic(`if let Ok(proxy)` 跳过)。
- `with_base_url`/`with_options`(测试用)也走同逻辑,但 mock server 是本地 127.0.0.1,reqwest::Proxy::all 会把本地请求也走代理?-- 注意:httpmock 的 `server.base_url()` 是 `http://127.0.0.1:port`,reqwest::Proxy::all 对 http://127.0.0.1 可能不走代理(默认 no_proxy 本地)。如果测试因代理失败,加 `reqwest::Proxy::http`/`https` 而非 `all`,或测试时设 `NO_PROXY=127.0.0.1`。
- 若现有测试因代理改动失败,调整代理配置(只对 https 走代理,或排除本地)。
