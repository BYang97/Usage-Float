# Release v0.4

OpenCode Usage Float v0.4 - usage history(/_server RPC)+ 评审团打磨(P0/P1/P2 修复)。

## 构建

```bash
cd src-tauri && cargo build --release --offline
```

产物:`src-tauri/target/release/app.exe`

## 校验

- **SHA256**:`916875ecc9e8abf1498fbdb6cfe3e57b1fa014d2aeebe4456dbbc7e4df97d7e6`(本机构建,同源同配置可复现)

## 能力(v0.3 基础 + v0.4 新增)

**v0.3 能力:** 多账号管理 + History/Models 页 + 日志系统 + 悬浮窗独立窗口 + tray + parse_plan(Lite)

**v0.4 新增:**
- **Usage History(/_server RPC)**:History 页加 usage 明细列表(model/provider/tokens/cost),POST opencode.ai/_server 解析 usg_ 记录(参考 opencode-go-dashboard fetchGoUsageHistory)
- **History 账号选择器**:下拉选账号查看不同账号 usage 历史
- **History 分页**:cursor 翻页 + 加载更多按钮
- **time_created 解析**:parse_rsc_date 解析 `new Date("...")` 日期字符串

## 评审团打磨(P0/P1/P2)

### 评审流程
1. claude 或ca computer 走查 4 页面 + 悬浮窗 -> 7 类问题
2. 4 omp 评审员并行(UI/交互/功能/Bug)-> 4 份报告(1059 行)
3. claude 汇总 -> omp 修复 -> 或ca computer e2e 验证

### P0 修复(7 项,用户可见 bug)
- Models 百分比格式化(66.77...% -> 66.8%)
- Models 空名模型 -> "未知"
- Dashboard expire 不显示 mock(真实 lite 无 expire)
- Dashboard status 不重复(删 "OpenCode Go - 正常" 副标题 + Active)
- 7d/30d token 分窗口(之前 7d==30d,现 88.1M/225.9M)
- History/Models Header 最小化不 crash(onMinimize/onClose 接线)
- Settings 未接线项置灰 + "即将支持"(开机启动/刷新频率/悬浮球/主题)

### P1 修复
- History 账号选择器 + cursor 分页
- PlanCard expire 空时完全隐藏(不显示 "到期时间 -")
- AccountTable 单账号刷新失败显示错误提示
- auto-refresh timer 去重(FloatWindow 删 startAutoRefresh,共享 App)
- History/Models retry 按钮单一(不 refreshAndNotify+loadData 双调)

### P2 美化
- cost 精度 toFixed(4)
- 重置时间格式统一(formatReset -> Xh Ym/Xm,Dashboard/AccountTable/FloatWidget)
- 加载/空态 spinner 统一(Spinner.tsx 组件)

## 测试

`cargo test --all-targets`:**20+ passed**
`tsc --noEmit`:零错误
**或ca computer e2e**:History 账号选择器/分页 + Models 格式化 + Dashboard 7d/30d + Settings 置灰 + 悬浮窗显示隐藏 + 日志文件

## 已知限制

- **account status/expire**:Go 页面不提供,用 mock 兜底(参考项目也不提供)
- **悬浮窗拖动**:或ca computer synthetic drag 限制(用户手动 work)
- **今日消耗 0**:本地 opencode db 今日无 sessions(数据源问题)
- 仅 Windows 验证

## 技术变更(相对 v0.3)

- **usage history**:collector/api.rs fetch_usage_history(POST /_server,parse_rsc_date 解析 new Date)+ lib.rs get_usage_history 命令 + History.tsx 明细列表
- **评审团**:docs/review/(walkthrough + 4 report + fix-plan + fix-p1p2)
- **P0/P1/P2**:前端(ModelUsage/PlanCard/History/Models/Settings/AccountTable/FloatWindow)+ Rust(build_account_info/aggregate_local/refresh_one)
- **参考**:https://github.com/Ruinique/opencode-go-dashboard

## 路线图

- **v0.5**:account status/expire 调研 + 开机启动(tauri-plugin-autostart)+ Windows 通知 + 多 plan 验证
