# Usage-Float v0.3 Bug 报告

> 评审日期: 2026-07-24
> 范围: web/src + src-tauri/src + walkthrough
> 方法: 代码走查 + 走查报告引用

---

## 优先级说明

| 级别 | 定义 |
|------|------|
| **P0** | 数据错误、崩溃、用户可见的无效值 |
| **P1** | 逻辑缺陷、状态不一致、功能异常 |
| **P2** | UI 未接线、没有验证、排版/风格 |

---

## P0 — 数据错误 / 无效值

### B1. Models 百分比未格式化，显示原始浮点数

**文件:** `web/src/components/ModelUsage.tsx:13`

```tsx
<span>{m.percentage}%</span>
```

**现象:** 页面直接渲染原始百分比如 `66.77579356834873%`（走查也确认此问题）。当 `LocalAggregate` 中有高精度小数时，用户看到难以阅读的长数字。

**根因:** 后端 `collector/opencode.rs:181` 计算 `(tokens as f64 / total_tokens as f64) * 100.0` 无精度控制；前端 `ModelUsage.tsx` 直接插值。两侧均无格式化。

**修复:** 前端 `ModelUsage.tsx` 加 `.toFixed(1)` 或 `Math.round(m.percentage * 10) / 10`。可选在后端 `map_local_to_usage_data` 预格式化。

---

### B2. 空模型名 `""` 显示为空白

**文件:** `src-tauri/src/collector/opencode.rs:164-175`

**现象:** 当 OpenCode 本地数据库中的 `session.model` 列为空字符串时，`aggregate_local` 以空字符串 `""` 作为 key 创建 `ModelBreakdown`。前端 `ModelUsage.tsx` 直接用 `m.name` 显示，导致看板上出现空名行，百分比归属不清。

**根因:** 模型名来自 OpenCode session 表的 `model` 列，可能为空。后端无清洗逻辑。

**修复:**
- 在 `aggregate_local` 中对 `name.is_empty()` 替换为 `"未知模型"`（或 `"unknown"`）。
- 前端 `ModelUsage.tsx` 可加兜底 `m.name || "未知"`。

---

### B3. Dashboard status 重复: "正常" + "Active"

**文件:** `web/src/components/PlanCard.tsx:15,23`

```tsx
{/* 硬编码子标签 */}
<span>OpenCode Go - 正常</span>
{/* 状态徽章 */}
<span style={{ textTransform: 'capitalize' }}>{status}</span>   // → "active"
```

**现象:** Dashboard 同一卡片同时显示 "OpenCode Go - 正常"（硬编码中文）和 "active"（英文枚举值 `PlanStatus::Active` 序列化）。走查确认此问题。

**根因:** `PlanCard` 写死了 `"OpenCode Go - 正常"` 子标签，同时 `status` 字段渲染为 `"active"`。两者含义重复。

**修复:**
- 移除硬编码的子标签行（`PlanCard.tsx:15`），由 `status` 徽章统一表达。
- 或定义状态中文映射字典：`active → "正常"`, `expired → "已过期"`, `error → "异常"`。

---

### B4. `token_7d` 与 `token_30d` 值相同

**文件:** `src-tauri/src/lib.rs:198-199`

```rust
token_7d: fmt_tokens(total),
token_30d: fmt_tokens(total),
```

**现象:** Dashboard Token 用量中 "近7天" 与 "近30天" 显示相同数值。实际应为不同时间范围的聚合。

**根因:** `map_local_to_usage_data` 中二者均使用 `local.total_tokens`（全部 session 总和），不是分别聚合 7 天和 30 天范围。

**修复:** 在 `collector/opencode.rs` 的 `aggregate_local` 中增加 `total_tokens_7d` / `total_tokens_30d` 字段，分别按时间过滤。或在 `LocalAggregate` 上新增分桶。

---

### B5. Mock 数据全量回退时显示错误套餐信息

**文件:** `src-tauri/src/lib.rs:44-48`

```rust
if has_local_data || api_quota.is_some() {
    Ok(map_local_to_usage_data(...))
} else {
    Ok(mock::mock_usage_data())  // ← 全量回退到 mock
}
```

**文件:** `src-tauri/src/mock.rs:10-11`

```rust
plan: "Go 月度版".to_string(),
status: PlanStatus::Active,
expire_date: "2026-08-20".to_string(),
```

**现象:** 当本地 SQLite 无数据且 API 配额获取失败时（如首次使用、cookie 过期、网络不通），Dashboard 显示：
- 套餐名 "Go 月度版"（用户实际可能是 Lite）
- 过期时间 "2026-08-20"（硬编码 mock）
- 状态 "Active"（无真实状态）

用户被错误信息误导。

**修复:**
- mock 回退时 `plan` 显示 "未获取到套餐信息"。
- `expireDate` 显示 "—"。
- `status` 设为 `error` 而非 `Active`。
- Dashboard 在 mock 全量回退时加提示条："部分数据为占位值，请配置账号获取真实数据"。

---

## P1 — 逻辑缺陷

### B6. History 页面时间格式类型不匹配

**文件:** `web/src/pages/History.tsx:12-17,141`

```tsx
function formatTime(ts: number): string {          // 声明: number
    if (!ts) return '—';
    const d = new Date(ts * 1000);                 // 期望 Unix timestamp(秒)
    ...
}
// 调用:
formatTime(item.time_created)                      // item.time_created: string
```

**类型:** `UsageHistoryItem.time_created` 在 TS 类型中为 `string`（来自 Rust 序列化的 ISO 时间字符串），而 `formatTime` 以 `number` 处理。运行时 `string * 1000` → `NaN` → `new Date(NaN)` → "Invalid Date" → 日期列全显示乱码。

**根因:** Rust 侧 `UsageHistoryItem` 的 `time_created` 字段在 `collector/model.rs` 定义为 `String`，TS 侧未做转换地按 `number` 处理。

**修复:**
- 统一类型: 将 `UsageHistoryItem.time_created` TS 类型改为 `number`（Unix 秒时间戳），或
- 将 `formatTime` 改为接受 ISO 字符串: `new Date(ts_string).getTime()` 或直接用 `new Date(ts_string).toLocaleString()`。

---

### B7. `refresh_one` 不写入缓存

**文件:** `src-tauri/src/lib.rs:392-415`

**现象:** `refresh_one` 成功获取 API 配额后 **不写入** `quota` 和 `account` 缓存表。之后 `get_usage_data` 再次调用时，`fetch_api_quota` 检查缓存 5 分钟 TTL，由于缓存未写入，每次都会重新调 API。

**影响:** 
- 每次 Dashboard 加载或自动刷新都触发完整的 API 请求（尽管 `refresh_one` 刚刚刷过）。
- `refresh_all` 同样只返回数据不写缓存。
- 活跃用户每小时触发 12 次 API 调用（频率 5min），浪费配额。

**修复:** 在 `refresh_one` 成功返回前调用 `database::set_quota_cache` + `database::set_account_cache`，与 `fetch_api_quota` 的缓存写入逻辑保持一致。

---

### B8. FloatWindow 卸载时清除全局自动刷新定时器

**文件:** `web/src/components/FloatWindow.tsx:50,55`

```tsx
startAutoRefresh(5 * 60 * 1000);  // 启动
// cleanup:
stopAutoRefresh();                 // 清除全局定时器
```

**现象:** 当悬浮窗被关闭/隐藏（FloatWindow 组件卸载）时，`stopAutoRefresh()` 清除了 `usage-service` 中的唯一定时器。Dashboard 和 History 页面失去自动刷新能力，数据停留到最后一次刷新时刻。

**根因:** `startAutoRefresh` / `stopAutoRefresh` 操作的是 `usage-service.ts` 中的单一 `refreshTimer`，非引用计数。任何调用者清除都影响全局。

**修复:**
- FloatWindow 不应管理定时器生命周期。移走 `startAutoRefresh` 和 `stopAutoRefresh`，由 `App.tsx` 独占控制。
- 或改为引用计数的定时器管理。

---

### B9. 多账号场景下 `account` 缓存表硬编码 `id = 1`

**文件:** `src-tauri/src/database.rs:320`

```sql
SELECT plan, updated_at FROM account WHERE id = 1
```

**现象:** 当有多个账号时，`fetch_api_quota` 中的缓存路径（`get_account_cache` / `set_account_cache`）始终读写 `id = 1`。若账号 B 刷新后写缓存，会覆盖账号 A 的套餐信息。最终 Dashboard 显示的套餐名取决于最后一次刷新的账号。

**根因:** v0.2 单账号时代的缓存设计未适配 v0.3 的多账号架构。

**修复:**
- `account` 表改为按 `workspace_id` 或 `account_id` 分条缓存。
- 或移除 `account` 缓存表，改为每次从 API 获取（因 5 分钟 TTL 对少量账号的体验无损）。

---

### B10. History 始终只取第一个账号的用量历史

**文件:** `web/src/pages/History.tsx:44-53`

```tsx
const accounts = await invoke<Account[]>('list_accounts');
if (accounts.length === 0) { ... return; }
const items = await invoke<UsageHistoryItem[]>('get_usage_history', {
    accountId: accounts[0].id,  // ← 仅取第一个
    cursor: 0,
});
```

**现象:** History 页面永远只显示第一个账号的用量历史记录。多账号用户无法查看其他账号的明细。

**修复:** History 页面增加账号选择器，或请求所有账号的历史并合并显示。同时 `get_usage_history` 后端可支持批量查询。

---

### B11. `get_usage_history` 无分页支持

**文件:** `src-tauri/src/lib.rs:453`

```rust
async fn get_usage_history(app_handle: tauri::AppHandle, account_id: String, cursor: i64)
```

**现象:** `cursor` 参数已存在，但前端 `History.tsx:51` 始终传 `cursor: 0`，且没有"加载更多"按钮或滚动加载。API 按 50 条分页，但用户只能看到第一批。超过 50 条的记录不可见。

**修复:** History 页面增加分页/滚动加载逻辑，保存 `cursor` 并在翻页时递增。

---

### B12. Dashboard 重试按钮触发双重请求

**文件:** `web/src/pages/Dashboard.tsx:108`

```tsx
onClick={() => { refreshAndNotify().catch(() => loadData()); }}
```

**现象:** 重试按钮同时调用 `refreshAndNotify()`（清缓存+重新获取+通知）和 `loadData()`（重新状态+获取）。若 `refreshAndNotify` 成功，`loadData` 因 `cached` 已存在只返回缓存值，不造成实质问题，但多了无效请求。若失败，`.catch(() => loadData())` 用 `getUsageData()` 再取——但 `cached` 变 `null` 后又变回旧值。

**修复:** 改为单一调用: `onClick={loadData}` 或 `onClick={() => refreshAndNotify()}`。两者选一即可。

---

## P2 — UI 未接线 / 缺少验证

### B13. Settings 页面四个控件未接线

**文件:** `web/src/pages/Settings.tsx:25-50`

| 控件 | 位置 | 状态 |
|------|------|------|
| 开机自动启动 toggle | L27 | 静态 UI，无 event handler，无 Tauri API 调用 |
| 刷新频率 5/30/60min | L31-33 | 默认 "5分钟" 高亮，点击无切换逻辑 |
| 悬浮球 toggle | L43 | 静态 UI，无 event handler |
| 主题选择 | L46-50 | "深色模式" 点击无响应 |

**影响:** 用户可看到设置 UI 但操作无效，造成困惑。

**修复:** 至少实现以下之一:
- 移除未接线控件（clean 但 UX 下降）
- 添加 `onClick` 占位功能: 刷新频率切换调用 `startAutoRefresh`，悬浮球 toggle 控制 `FloatWindow.show/hide`
- 或添加 TODO 注释并 disabled

---

### B14. AccountDialog 接受空 auth_cookie 和空 workspace_id

**文件:** `web/src/components/AccountDialog.tsx:38-43`

```tsx
const form: AccountForm = {
    name,
    workspace_id: workspaceId,    // 无格式校验
    auth_cookie: cookieRef.current?.value ?? '',  // 允许空字符串
    notes,
};
```

**现象:** 用户可以创建不包含 auth_cookie 或 workspace_id 的账号。这些账号在 `refresh_one` 时会返回 `NoCookie` 或 `NotFound` 错误，影响用户体验。

**修复:** 在 `handleSave` 中增加验证:
- `workspaceId.trim() === ''` → 提示 "工作区 ID 不能为空"
- `cookieRef.current?.value?.trim() === ''` → 提示 "Auth Cookie 不能为空"
- `name.trim() === ''` → 提示 "名称不能为空"

后端 `create_account` 也应做对应校验。

---

### B15. AccountTable 启动时并发 N 个 API 请求

**文件:** `web/src/components/AccountTable.tsx:24-27`

```tsx
for (const a of list) {
    refreshOneUsage(a.id);   // 无间隔，并行发起
}
```

**现象:** 用户有 N 个账号时，页面加载瞬间并发 N 个 `refresh_one` 请求到 `opencode.ai`。可能触发 API 限流或 429。

**修复:** 串行刷新或控制并发数（如 `p-limit`）。单次最多 2-3 个并发即可。

---

### B16. ErrorBoundary 重置后不重新加载数据

**文件:** `web/src/components/ErrorBoundary.tsx:35-37`

```tsx
handleRetry = () => {
    this.setState({ hasError: false, error: null });
};
```

**现象:** 当 ErrorBoundary 捕获到渲染异常后，用户点击 "重试" 按钮只是清空错误状态，不触发父组件重新加载。若数据未自然变化，组件将再次抛错，陷入循环。

**修复:** `handleRetry` 或父组件在错误重置后应重新调用数据加载（如调用 `refreshAndNotify` 或 `loadData`）。

---

### B17. 降级数据中 `tokenToday` 可能为 0

**文件:** `src-tauri/src/lib.rs:175-177`

```rust
let token_today = history_records.last()
    .map(|r| fmt_tokens(r.tokens as i64))
    .unwrap_or_else(|| fmt_tokens(total));
```

**现象:** 今天的日期桶值为 0 时（今日无 session），`tokenToday` 显示 `"0.0"`。走查提到"今日 0" 疑似数据问题。从代码看这是正常逻辑——最近一天无数据即 0。不是 bug，但 UX 上可优化为 `"—"` 或 `"0"`。

**建议:** 当 `r.tokens == 0.0` 时显示 `"0"` 而非 `"0.0"`；或文案提示 "今日暂无用量"。

---

## 汇总

| 编号 | 问题 | 文件 | 优先级 |
|------|------|------|--------|
| B1 | Models 百分比未格式化 | `ModelUsage.tsx:13` | P0 |
| B2 | 空模型名显示为 `""` | `opencode.rs:174` | P0 |
| B3 | Dashboard 状态重复 ("正常"+"Active") | `PlanCard.tsx:15,23` | P0 |
| B4 | `token_7d` == `token_30d` | `lib.rs:198-199` | P0 |
| B5 | Mock 全量回退显示错误套餐 | `lib.rs:46`, `mock.rs:10` | P0 |
| B6 | History 时间格式类型不匹配 | `History.tsx:12-17,141` | P1 |
| B7 | `refresh_one` 不写缓存 | `lib.rs:392-415` | P1 |
| B8 | FloatWindow 卸载清除全局定时器 | `FloatWindow.tsx:55` | P1 |
| B9 | `account` 缓存表硬编码 `id=1` | `database.rs:320` | P1 |
| B10 | History 只取第一个账号 | `History.tsx:44-53` | P1 |
| B11 | `get_usage_history` 无前端分页 | `History.tsx:51` | P1 |
| B12 | Dashboard 重试双重请求 | `Dashboard.tsx:108` | P1 |
| B13 | Settings 四个控件未接线 | `Settings.tsx:25-50` | P2 |
| B14 | AccountDialog 无字段验证 | `AccountDialog.tsx:38-43` | P2 |
| B15 | AccountTable 并发 N 个 API 请求 | `AccountTable.tsx:24-27` | P2 |
| B16 | ErrorBoundary 重置后不重新加载 | `ErrorBoundary.tsx:35-37` | P2 |
| B17 | 今日 token 为 0 时显示 `0.0` | `lib.rs:175-177` | P2 |

### 按严重程度分布

- P0: 5
- P1: 7
- P2: 5
- 总计: 17

### 修复顺序建议

1. **P0 全部** — 数据正确性/无效值，用户可见且影响信任度
2. **P1 B7 + B8 + B9** — 缓存和全局状态一致性缺陷
3. **P1 B6 + B10 + B11** — History 页面功能缺陷
4. **P1 B12** — 轻微冗余请求
5. **P2** — 体验和验证改进
