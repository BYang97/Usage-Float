//! Windows 系统代理读取。
//! reqwest 不读 Windows 注册表代理,这里主动读,供 OpenCodeApiClient 配代理。

/// 读 Windows 注册表系统代理。
/// 返回 Some("http://host:port") 或 None(未启用 / 读取失败 / 非 Windows)。
///
/// 注册表路径:HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings
///   ProxyEnable (DWORD) == 1 且 ProxyServer (String) 存在 -> http://{ProxyServer}
#[cfg(target_os = "windows")]
pub fn read_system_proxy() -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

    let key = hkcu.open_subkey(path).ok()?;

    // ProxyEnable: DWORD (u32)
    let enabled: u32 = key.get_value("ProxyEnable").ok()?;
    if enabled != 1 {
        return None;
    }

    // ProxyServer: String, e.g. "127.0.0.1:6789"
    let server: String = key.get_value("ProxyServer").ok()?;
    if server.is_empty() {
        return None;
    }

    Some(format!("http://{}", server))
}

/// 非 Windows 平台返回 None。
#[cfg(not(target_os = "windows"))]
pub fn read_system_proxy() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_system_proxy_no_panic() {
        // 真实注册表,可能 Some 或 None,但不 panic。
        let proxy = read_system_proxy();
        if let Some(url) = proxy {
            assert!(
                url.starts_with("http://"),
                "代理 URL 应以 http:// 开头,得到: {url}"
            );
        }
    }
}
