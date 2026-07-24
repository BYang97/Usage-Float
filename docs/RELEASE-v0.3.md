# Release v0.3

OpenCode Usage Float v0.3 - 多账号管理 + History/Models 页 + 日志系统 + 悬浮窗独立窗口 + 系统托盘。

## 构建

```bash
cd src-tauri && cargo build --release --offline
```

产物:`src-tauri/target/release/app.exe`

## 校验

- **SHA256**:`27902da5b5ffe832db7ca715ae84dc736d4d6b85bd8a62f071a5923612e1a5c0`(本机构建,同源同配置可复现)

## 能力(v0.2 基础 + v0.3 新增)

**v0.1 能力:** 本地 SQLite 用量 + 仪表盘(token 累计/历史/模型分布)
**v0.2 能力:** opencode.ai Go 真实配额(rolling/weekly/monthly)+ app 代理(rustls-tls)+ 配额缓存(5min TTL)

**v0.3 新增:**
- **多账号管理**:AccountDialog(添加/编辑)+ AccountTable(列表+刷新/编辑/删除)+ accounts 表 CRUD(ring AEAD 加密)+ migrate_from_settings(旧单账号迁移)
- **History/Models 页**:使用记录(token 今日/7天/30天 + 历史折线)+ 模型统计(占比条形图),替代 Dashboard 占位
- **日志系统**:tauri_plugin_log 文件 target(app_data_dir/logs)+ Stdout + Webview,level Debug;前端 logger.ts 封装;关键路径 log(fetch_api_quota/list_accounts/create_account/refresh_one)
- **悬浮窗独立 OS 窗口**:无边框+透明+置顶+skipTaskbar,data-tauri-drag-region 拖到桌面任意位置
- **系统托盘**:show/hide 主窗口+悬浮窗 + 退出 + 左键切换悬浮窗
- **parse_plan**:subscriptionPlan + lite 余额模式 -> "Lite"(真实 HTML 格式)
- **或ca computer UI 端到端验证**:添加账号/刷新/导航/悬浮窗显示隐藏/日志

## 测试

`cargo test --all-targets`:**41 passed;0 failed**
- 10 database(accounts CRUD + migration)
- 9 api(正则解析 + parse_plan subscriptionPlan/lite)
- 8 collector(本地 SQLite 采集)
- 5 cache(缓存 TTL + account)
- 1 proxy(系统代理)

`tsc --noEmit`:零错误

**或ca computer e2e**:History/Models 导航 + 多账号添加/刷新 + 悬浮窗显示/隐藏 + 日志文件验证

## 已知限制

- **History token 数据空**:本地 opencode db 无 token 历史(数据源问题,非 bug)
- **account status/expire 缺**:Go 页面无,用 mock 兜底
- **悬浮窗拖动**:或ca computer synthetic drag 限制(用户手动 work)
- **多 plan 验证**:当前 workspace 是 lite,go-monthly 等其他 plan 未实测
- 仅 Windows 验证

## 技术变更(相对 v0.2)

- **多账号**:database.rs accounts 表 + CRUD + migrate_from_settings;lib.rs Tauri 命令(list/create/update/delete + refresh_one/refresh_all)
- **悬浮窗**:tauri.conf.json float 窗口 + Cargo.toml tray-icon feature + lib.rs 系统托盘 + capabilities window 权限(allow-show/hide/start-dragging 等)
- **日志**:tauri_plugin_log targets([Stdout, LogDir, Webview]) + log::info 关键路径 + 前端 logger.ts
- **parse_plan**:subscriptionPlan:"xxx" > plan:$R[N]="xxx" > useBalance:!0 -> "Lite"
- **参考**:https://github.com/Ruinique/opencode-go-dashboard

## 路线图

- **v0.4**:History token 数据采集(opencode db aggregate / /_server RPC)+ account status/expire + 开机启动(tauri-plugin-autostart)+ Windows 通知(配额阈值)
