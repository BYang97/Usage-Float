# 任务:app-proxy-read - 读 Windows 系统代理

## 目标

新建 `proxy.rs`,读 Windows 注册表系统代理(`ProxyEnable`/`ProxyServer`),返回代理 URL(如 `http://127.0.0.1:6789`)。供 `OpenCodeApiClient` 配代理用。

## 背景

- `reqwest`(system-proxy feature)只读 `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` 环境变量,**不读 Windows 注册表系统代理**。
- 用户机器有系统代理(`127.0.0.1:6789`,MITM 型),app 运行时若没设环境变量,`fetch_quota` 连不上(TLS 握手失败)。
- 需要 app 主动读系统代理,配给 reqwest。

## 改动

### 1. `src-tauri/Cargo.toml` - 加 winreg

```toml
# winreg: 读 Windows 注册表(系统代理设置)
winreg = "0.5"
```

### 2. `src-tauri/src/proxy.rs` - 新建

```rust
//! Windows 系统代理读取。
//! reqwest 不读 Windows 注册表代理,这里主动读,供 OpenCodeApiClient 配代理。

/// 读 Windows 注册表系统代理。
/// 返回 Some("http://host:port") 或 None(未启用 / 读取失败 / 非 Windows)。
///
/// 注册表路径:HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings
///   ProxyEnable (DWORD) == 1 且 ProxyServer (String) 存在 -> http://{ProxyServer}
pub fn read_system_proxy() -> Option<String> {
    // 用 winreg crate 读 HKEY_CURRENT_USER\...\Internet Settings
    // ProxyEnable == 1 且 ProxyServer 非空 -> 返回 format!("http://{}", proxy_server)
    // 否则 None
    // 非 Windows(cfg(target_os = "windows"))-> None
    todo!()
}
```

### 3. `src-tauri/src/lib.rs` - 声明模块

在 `mod database;` 附近加:
```rust
mod proxy;
```

### 4. 测试 - `src-tauri/tests/proxy_test.rs`(新建)或 `proxy.rs` 内 `#[cfg(test)]`

- 测试 `read_system_proxy()` 返回 `Option<String>`,不 panic(真实注册表,可能 Some 或 None)
- 如果返回 Some,验证格式是 `http://...`

## 验证

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --test proxy_test
```

## 注意

- 只读注册表,不写。
- `ProxyEnable=0` 或无 `ProxyServer` -> `None`。
- 非 Windows 平台 -> `None`(`#[cfg(not(target_os = "windows"))]` 返回 None)。
- winreg 仅 Windows,用 `#[cfg(target_os = "windows")]` 守卫,非 Windows 提供空实现。
