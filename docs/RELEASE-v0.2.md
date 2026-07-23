# Release v0.2

OpenCode Usage Float v0.2 - 在 v0.1 本地 SQLite 用量基础上,集成 opencode.ai Go 套餐真实配额(5h/周/月百分比 + 重置倒计时)。

## 构建

```bash
cd src-tauri && cargo build --release --offline   # 产 exe(离线,依赖已缓存)
# 或完整 bundle(需联网下载 WiX 产 msi):
npx @tauri-apps/cli build
```

产物:`src-tauri/target/release/app.exe`(约 15.5 MB)

## 校验

- **SHA256**:`d0a9b819bd6a2dfbcd7036007a8e5b6c6dee920ab1a5eadd7e7204e156a43710`(本机构建,同源同配置可复现)

## 能力(v0.1 基础 + v0.2 新增)

**v0.1 能力:**
- 读取本地 OpenCode SQLite(`~/.local/share/opencode/opencode.db`),只读
- 仪表盘:累计 token、近 7 天历史折线、按模型分组占比
- OpenCode Go 认证面板(auth cookie 粘贴,本地 ring AEAD 加密存储)
- 完全离线(token 数据)

**v0.2 新增:**
- **真实配额**:从 opencode.ai Go 页面(HTML)解析 rolling/weekly/monthly `usagePercent` + `resetInSec`(非 mock)
- **workspaceId**:设置面板新增 workspaceId 输入(`wrk_xxx`,从 opencode.ai 工作区 URL 获取)
- **app 代理**:读 Windows 系统代理(reqwest rustls-tls + 注册表代理),支持 MITM 代理环境
- **配额缓存**:5min TTL,避免频繁打 opencode.ai
- 端到端验证通过(e2e_verify 真跑:rolling 1% / weekly 1% / monthly 88%;或ca computer UI 端到端:cookie 填充 -> 保存 -> 首页显示真实配额 88%)

## 测试

`cargo test --all-targets`:**27 passed;0 failed**
- 6 api 单元(正则解析 React flight 数据)
- 7 httpmock 集成(配额场景:正常/无cookie/无workspace/401/5xx/超时/sign-in)
- 8 collector(本地 SQLite 采集)
- 5 cache(缓存 TTL + account)
- 1 proxy(系统代理读取)

**端到端验证**:`cargo run --example e2e_verify`(设环境变量 `OPENCODE_COOKIE` + `OPENCODE_WORKSPACE_ID`),真跑 opencode.ai 验证配额获取整条链(proxy.rs 代理 -> reqwest::Proxy -> fetch_quota -> HTML 解析)。

**或ca computer UI 端到端**:或ca computer 驱动 Tauri webview,set-value 填 cookie+workspaceId -> 保存 -> 首页显示真实配额(monthly 88%)。修复前端 invoke 接线(TauriProvider 改调 `get_usage_data`,之前调不存在的命令导致全 fallback 0%)+ cookie 控件(`type=text` + `WebkitTextSecurity:'disc'` 视觉遮罩 + 非受控 ref,绕过或ca computer 对 password 字段/受控 onChange 的限制)。

## 已知限制

- **plan 解析 None**:`parse_plan` 正则没匹配真实 HTML 的 plan 字段(格式待确认),plan 暂缺。配额三窗口正常。
- **account status/expire 缺**:Go 页面只提供 plan(且当前 None),无 status/expire。用 mock 兜底。要 status/expire 需找别的端点。
- **msi bundle**:tauri build 的 msi 阶段需联网下载 WiX(同 v0.1);exe 可离线产出。
- 仅 Windows 验证(macOS/Linux 路径逻辑已写,未实测)。

## 技术变更(相对 v0.1)

- **端点**:`console.opencode.ai` JSON API(废弃,全 401) -> `opencode.ai/workspace/{ws}/go` 页面 HTML + 正则解析 React Server Component flight 数据
- **reqwest**:换 rustls-tls(避开 Windows schannel 在代理 MITM 下 TLS 握手失败)
- **新增**:`proxy.rs`(读系统代理)+ workspaceId + 配额缓存改 `percent`/`reset_in_sec`
- **参考**:https://github.com/Ruinique/opencode-go-dashboard

## 路线图

- **v0.3**:account status/expire + plan 解析 + 系统托盘 + 开机启动 + 悬浮球 AlwaysOnTop + Windows 通知
