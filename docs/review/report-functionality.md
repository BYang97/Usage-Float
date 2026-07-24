# Usage-Float v0.3 功能评审报告

> 评审日期: 2026-07-24 | 范围: functionality 视角
> 基于 `web/src` + `src-tauri/src` 代码走查 + `docs/review/walkthrough.md`

---

## 优先级定义

| 级别 | 含义 |
|------|------|
| **P0** | 用户可见数据错误 / 核心功能缺失 / 可能导致误解或困惑 |
| **P1** | 功能不完整 / 数据链断裂 / 多账号场景异常 / 性能问题 |
| **P2** | 样式格式 / 降级处理 / 冗余代码 / 可维护性 |

---

## 问题列表

### P0 — 必须修复

#### P0-1. Models 百分比未格式化
- **位置**: `web/src/components/ModelUsage.tsx:13`
- **表现**: `{m.percentage}%` 直接渲染 raw f64，如 `66.77579356834873%`
- **原因**: `src-tauri/src/collector/opencode.rs:180` 计算 `(tokens as f64 / total_tokens as f64) * 100.0`，未做精度截断；前端也未格式化
- **修复**: 前端 `(Math.round(m.percentage * 10) / 10).toFixed(1) + '%'`，或 Rust 侧 `(pct * 10.0).round() / 10.0`

#### P0-2. Models 空名模型显示为空白
- **位置**: `web/src/components/ModelUsage.tsx:10-19`
- **表现**: model_id 为空字符串时渲染空行，React key (`m.name`) 为 `""` 导致渲染异常
- **原因**: `collector/opencode.rs:174` 直接从 `s.model_id` 作为 name，数据库可能有空值
- **修复**: Rust 侧 `if name.is_empty() { "未知".to_string() }`；前端侧 `m.name || '未知'`

#### P0-3. token_7d 与 token_30d 值相同
- **位置**: `src-tauri/src/lib.rs:197-199`
- **代码**:
  ```rust
  token_7d: fmt_tokens(total),
  token_30d: fmt_tokens(total),
  ```
- **表现**: Dashboard/History 页"近7天"和"近30天"显示相同数值
- **原因**: `aggregate_local` 仅返回 `total_tokens`（全量总和），未分别计算 7d 和 30d 聚合；`map_local_to_usage_data` 把 `total` 重复赋值给两个字段
- **修复**: `LocalAggregate` 增加 `tokens_7d`/`tokens_30d` 字段，`aggregate_local` 按时间窗口分别求和

#### P0-4. Dashboard 到期时间/Active/"正常" 为 mock 数据
- **位置**: `src-tauri/src/mock.rs:10-12` + `src-tauri/src/lib.rs:216-224`
- **表现**: Lite 套餐无到期时间，但统一显示 `expire_date: "2026-08-20"`、`PlanStatus::Active`、PlanCard 硬编码 "正常"
- **原因**: Go 页面不提供 status/expire，`build_account_info` 用 mock 兜底
- **修复建议**:
  - Lite plan: `expire_date` 显示 "—" 或 "不适用"，status 根据 API 是否可达决定
  - Go-monthly plan: 尝试从 Go 页面解析 expire 信息（如有）
  - Mock 降级时标注 "数据未同步"

#### P0-5. Dashboard status 重复: "正常" + "Active"
- **位置**: `web/src/components/PlanCard.tsx:15` + `:21-24`
- **表现**: PlanCard 同时显示硬编码文字 `"OpenCode Go - 正常"` + 状态徽章 `{status}`
- **原因**: 两个信息源表达同一状态
- **修复**: 删除硬编码副标题或改为显示 plan 名称，保留 status 徽章

#### P0-6. Settings 通用设置未接线
- **位置**: `web/src/pages/Settings.tsx:25-35`
- **表现**: "开机自动启动" toggle + "刷新频率" chips 仅 UI 展示，无实际行为
- **缺少**:
  - 自动启动: 未接入 `tauri-plugin-autostart` 或 `auto-launch`
  - 刷新频率: `startAutoRefresh` 硬编码 5 分钟，未读取用户选择，无保存逻辑
- **修复**: 接入插件 + 保存频率到 settings 表 + 读取设置启动 timer

#### P0-7. Settings 外观设置未接线
- **位置**: `web/src/pages/Settings.tsx:41-50`
- **表现**: "悬浮球" toggle + "主题" selector 仅 UI
- **缺少**:
  - 悬浮球 toggle: 应控制 FloatWindow 的 show/hide
  - 主题切换: 应切换 CSS 变量（深色/浅色），但现有 CSS 仅有深色一套
- **修复**: 实现 toggle 逻辑 + 主题变量切换

---

### P1 — 高优先级

#### P1-1. History 仅查询第一个账号
- **位置**: `web/src/pages/History.tsx:44-53`
- **代码**:
  ```tsx
  const accounts = await invoke<Account[]>('list_accounts');
  if (accounts.length === 0) { ... return; }
  const items = await invoke<UsageHistoryItem[]>('get_usage_history', {
    accountId: accounts[0].id, cursor: 0,
  });
  ```
- **影响**: 多账号用户只能看到第一个账号的历史记录
- **修复**: 增加账号选择器（select/dropdown），切换时重新拉取

#### P1-2. FloatWindow 和 App 双重启动 autoRefresh
- **位置**: `web/src/App.tsx:35` + `web/src/components/FloatWindow.tsx:50`
- **表现**: 两个 `startAutoRefresh(5*60*1000)` 创建两个独立定时器，每次 double fetch
- **原因**: 双窗口各启动一次自动刷新
- **修复**: 统一管理 autoRefresh（全局仅一处启动），或 FloatWindow 只 subscribe 不 start

#### P1-3. refresh_one / refresh_all 不写 quota/account 缓存
- **位置**: `src-tauri/src/lib.rs:393-415` (refresh_one), `:418-445` (refresh_all)
- **表现**: 手动刷新配额后，`get_usage_data` 仍走旧缓存或兜底 mock 数据
- **原因**: `refresh_one` 调用 API 拿到结果后不写 `database::set_quota_cache` / `set_account_cache`
- **修复**: `refresh_one` 成功后写入缓存表

#### P1-4. History formatTime 参数类型不匹配
- **位置**: `web/src/pages/History.tsx:12-16` + `:141`
- **表现**: `formatTime(item.time_created)` 参数 `ts` 声明为 `number`，但 `UsageHistoryItem.time_created` 类型为 `string`（ISO 日期字符串）
- **原因**: `formatTime(ts * 1000)` 在 ts 为字符串时产生 `NaN`，`new Date(NaN)` → Invalid Date
- **修复**: `formatTime` 兼容 ISO 字符串: `const d = new Date(ts)`，移除 `* 1000`

#### P1-5. 无内置自动启动实现
- **位置**: `web/src/pages/Settings.tsx:26-28` UI 存在但未接入
- **影响**: 用户期望 toggle 生效但无反应
- **修复**: 接入 `tauri-plugin-autostart` 或 `auto-launch` npm 包

#### P1-6. 无主题切换实现
- **位置**: `web/src/pages/Settings.tsx:45-50` UI 存在但未接入
- **影响**: "深色模式"选择器不可操作
- **修复**: 定义浅色 CSS 变量集 + 切换逻辑（或仅移除未实现的 UI）

---

### P2 — 中等优先级

#### P2-1. TauriProvider 模型降级数据硬编码
- **位置**: `web/src/providers/tauri-provider.ts:57-62`
- **代码**:
  ```ts
  private getModelUsage(): ModelUsageData[] {
    return [
      { name: 'GPT 系列', percentage: 60, color: '#4a9eff' },
      { name: 'Claude 系列', percentage: 40, color: '#d97706' },
    ];
  }
  ```
- **问题**: Rust 端 `get_usage_data` 本身有 mock 兜底，TS 侧降级是冗余且不一致的（数据与 Rust mock 不同）
- **建议**: 移除 TS 侧模型降级，TauriProvider invoke 失败时统一用 Rust 的 mock 数据

#### P2-2. AccountTable 并发刷新无限制
- **位置**: `web/src/components/AccountTable.tsx:25-27`
- **代码**:
  ```ts
  for (const a of list) {
    refreshOneUsage(a.id);
  }
  ```
- **问题**: N 个账号同时发起 invoke 请求，无并发控制
- **建议**: 使用 `Promise.allSettled` + 分批（如 3 个一批）或串行

#### P2-3. FloatWindow 独立启动 autoRefresh 的竞态
- **位置**: `web/src/components/FloatWindow.tsx:25-57`
- **问题**: FloatWindow 挂载时 `startAutoRefresh` + `subscribe`，若在 App 之后挂载，会重置 timer 引用
- **影响**: `stopAutoRefresh` 在 FloatWindow 卸载时关闭 timer，导致主窗口停止刷新
- **建议**: AutoRefresh 归 App 管理，FloatWindow 只 subscribe

#### P2-4. Sidebar 导航 items 硬编码
- **位置**: `web/src/components/Sidebar.tsx:5-10`
- **问题**: 四个导航项目的定义在组件内，无法动态扩展
- **建议**: 无实际影响，可保持现状

#### P2-5. Dashboard 数据兜底路径混乱
- **位置**: `src-tauri/src/lib.rs:43-48`
- **逻辑**: 有 local 或 API 数据 → map_local_to_usage_data；纯 mock 走整份 mock
- **问题**: 混合路径可能导致 API quota 搭配 mock token 数据的组合，不够透明
- **建议**: 明确标注数据来源（如 token 显示"本地数据"、quota 显示"在线"）

#### P2-6. balance/lite 场景处理存疑
- **位置**: `src-tauri/src/collector/api.rs:340-341`
- **逻辑**: 当 `useBalance:!0` 时返回 `"Lite"`
- **问题**: Lite 套餐无 quota 窗口（5h/weekly/monthly 均为 0%），Dashbaord 显示三个 0% 卡片意义存疑
- **建议**: Lite 用户隐藏配额卡片或显示"按量计费，无配额限制"

#### P2-7. 空账号列表时 History 无引导
- **位置**: `web/src/pages/History.tsx:44-48`
- **表现**: 无账号时显示空列表，但无提示引导用户添加
- **建议**: 增加 "请先在设置中添加账号" 引导文案

---

## 数据链完整性评估

### Token 消耗数据链

```
OpenCode SQLite (session表)
  → collector/opencode.rs:read_all_sessions()
  → aggregate_local() → LocalAggregate { total_tokens, daily_history, models }
  → lib.rs:map_local_to_usage_data()
  → UsageData.tokens
```

- **问题**: `total_tokens` 无时间窗口过滤，7d/30d 相同（P0-3）
- **daily_history**: 仅计算近 7 天按 weekday 聚合
- **建议**: 增加 30d 窗口计算，或 frontend 做聚合

### 配额数据链

```
opencode.ai Go 页面 HTML
  → collector/api.rs:fetch_quota()
  → ApiQuota { five_hour, weekly, monthly }
  → database::set_quota_cache() (仅 get_usage_data 路径)
  → QuotaInfo → UsageData.quota
```

- **缓存写**: `get_usage_data` 路径写缓存；`refresh_one`/`refresh_all` **不写**（P1-3）
- **缓存读**: 5 分钟 TTL，`get_quota_cache` 窗口级别独立过期
- **建议**: 统一缓存写入路径

### 用量历史数据链

```
opencode.ai /_server RPC
  → collector/api.rs:fetch_usage_history()
  → Vec<UsageHistoryItem>
  → lib.rs:get_usage_history command
  → History.tsx:invoke('get_usage_history')
```

- **仅查询第一个账号**（P1-1），多账号场景断裂
- 无本地缓存，每次切换账号都是 API 调用

### 多账号 CRUD 链

```
AccountTable (TSX)
  → invoke('list_accounts'/'create_account'/'update_account'/'delete_account')
  → database.rs CRUD (auth_cookie AES-256-GCM 加密)
  → SQLite accounts 表

AccountTable 加载链:
  list_accounts → UI 显示(usage=null) → refreshOne (逐个刷新)
```

- **CRUD 完整**: create/read/update/delete/refresh 均已实现
- **问题**:
  - `refresh_one` 不写缓存（P1-3）
  - 并发无限制（P2-2）
  - 首次加载显示 `null usage` 状态不够优雅

---

## 悬浮窗(tray)功能评估

### 现有功能
- Tray 菜单: 显示主窗口 / 显示悬浮窗 / 隐藏悬浮窗 / 退出
- Tray 左键点击: toggle 悬浮窗 show/hide
- 悬浮窗: 显示 5h 配额环 + reset 时间 + 打开仪表盘
- 悬浮窗隐藏不退出应用

### 缺少
- Tray 右键菜单悬浮窗状态不同步（隐藏时菜单仍显示"隐藏悬浮窗"）
- 悬浮窗关闭按钮触发 hide，但 tray 菜单无刷新
- 悬浮球显示/隐藏 toggle 未接线（Settings UI）

---

## 汇总

| 领域 | P0 | P1 | P2 | 总计 |
|------|----|----|----|------|
| Dashboard | 5 | 0 | 1 | 6 |
| History | 1 | 2 | 1 | 4 |
| Models | 2 | 0 | 0 | 2 |
| Settings | 2 | 3 | 0 | 5 |
| Multi-account | 0 | 1 | 1 | 2 |
| 数据链 | 0 | 1 | 1 | 2 |
| **总计** | **7** | **6** | **4** | **17** |
