//! 系统代理读取集成测试。
//! 真实注册表,可能 Some 或 None,但不 panic。

use app_lib::proxy::read_system_proxy;

#[test]
fn test_read_system_proxy_no_panic() {
    let proxy = read_system_proxy();
    if let Some(url) = proxy {
        assert!(
            url.starts_with("http://"),
            "代理 URL 应以 http:// 开头,得到: {url}"
        );
    }
}
