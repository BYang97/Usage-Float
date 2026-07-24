# Usage-Float v0.3 UI 评审报告

> 评审日期: 2026-07-24
> 评审范围: `web/src/` 全部组件 + `src-tauri/src/` 涉及 UI 的 Rust 侧
> 参考文档: `docs/review/walkthrough.md`, `docs/review/review-ui.md`

---

## 总览

共发现 **20 项问题/建议**，按优先级分: P1 × 5 | P2 × 10 | P3 × 5。

|优先级|含义|数量|
|---|---|---|
|P1|Major — 直接影响数据可读性或用户感知质量|5|
|P2|Medium — 一致性/体验改善，修复成本低|10|
|P3|Low — 可推迟的技术债或微调|5|

---

## P1 — Major

### 1. Models 百分比未格式化

|字段|值|
|---|---|
|**文件**|`web/src/components/ModelUsage.tsx:13`|
|**数据源**|`src-tauri/src/collector/opencode.rs:181` — `(tokens as f64 / total_tokens as f64) * 100.0` 为完整精度 f64|
|**现状**|`{m.percentage}%` 直接渲染原始浮点，如 `66.77579356834873%`|
|**影响**|走查截图确认存在，数据不可读|
|**修复**|`ModelUsage.tsx:13`: `{m.percentage.toFixed(1)}%` → `66.8%`。亦可考虑在 Rust 侧聚合时即格式化（但会丢失精度），推荐前端做。|

### 2. Models 空名模型显示为空白

|字段|值|
|---|---|
|**文件**|`web/src/components/ModelUsage.tsx:12`|
|**数据源**|`opencode.rs:194` — `name` 来自 `session.model` 原始值，可能为 `""`|
|**现状**|`{m.name}` 当 `name == ""` 时渲染空白行|
|**影响**|走查确认存在 `""` 模型占 66.77%，视觉上像个漏洞|
|**修复**|渲染前 fallback: `m.name || "未知"`。也可在 Rust 侧 `aggregate_local` 中过滤空名或标为 "未知"。|

### 3. PlanCard 状态信息重复 + Mock 数据

|字段|值|
|---|---|
|**文件**|`web/src/components/PlanCard.tsx:13-24`|
|**现状**|三处 redundancy: (a) 固定 subtitle `"OpenCode Go - 正常"`; (b) 到期时间 `expireDate` 始终显示; (c) 末尾 `status` badge 再次显示 Active。且 `expireDate` 是 mock 硬编码 `2026-08-20`（`lib.rs:223` 即使 API 有数据也 fallback mock）。|
|**影响**|走查标为"重复 + mock"，真实 Lite 无 expire 字段|
|**修复**|(a) 移除固定 subtitle 或改为数据驱动; (b) `expireDate` 条件渲染 — `expireDate !== '—'` 或 `expireDate !== ''` 时才显示; (c) `status` badge 与 subtitle 保留其一。Rust 侧 `build_account_info` 在无 API expire 时传 `None` / `"—"` 而非 mock 值。|

### 4. Settings 多个 UI 控件无功能

|字段|值|
|---|---|
|**文件**|`web/src/pages/Settings.tsx:26-50`|
|**现状**|开机启动 toggle、刷新频率 chips、悬浮球 toggle、主题下拉框 — 全部纯渲染，无 event handler / invoke 接线|
|**影响**|用户交互无反馈，费解|
|**修复**|两种方案二选一: (a) 全部置灰 + 加 `opacity: 0.4; cursor: not-allowed` + tooltip "即将支持"; (b) 接入 Tauri command（开机启动用 `tauri-plugin-autostart`，刷新频率写入 store，主题切换动态 CSS var）。推荐 (a) 做短期标记，(b) 做后续迭代。|

### 5. 到期时间字段对 Lite/无过期套餐不适用

|字段|值|
|---|---|
|**文件**|`src-tauri/src/lib.rs:222-223` — `expire_date: mock_data.account.expire_date.clone()`|
|**现状**|即使 API 返回 `ApiAccount.expire_date = None`，Rust 侧仍 fallback 到 mock 的 `"2026-08-20"`。前端 `PlanCard` 无条件渲染到期时间行。|
|**影响**|Lite 用户看到错误的到期时间|
|**修复**|(1) `build_account_info`: 当 `api_account.expire_date` 为 `None` 时传空字符串或 `"—"`，不 fallback mock。 (2) `PlanCard` 条件渲染到期时间区域。 (3) 前端 `types/index.ts` 的 `AccountInfo.expireDate` 允许空字符串。|

---

## P2 — Medium

### 6. QuotaCard 百分比值未格式化

|字段|值|
|---|---|
|**文件**|`web/src/components/QuotaCard.tsx:10`, `ProgressBar.tsx:6-9`|
|**现状**|`{percentage}%` 直接渲染，API 可能返回高精度 f64。`ProgressBar` 用 `width: ${percentage}%` 做 CSS 宽度，CSS 会截断小数位但展示值不受影响。|
|**修复**|`QuotaCard.tsx:10` 加 `Math.round(percentage)` 或 `percentage.toFixed(0)`。`QuotaRing.tsx:20` 同理。|

### 7. QuotaRing 百分比值未格式化

|字段|值|
|---|---|
|**文件**|`web/src/components/QuotaRing.tsx:20`|
|**现状**|`{percentage}%` — 同上，悬浮球环内显示高精度百分比|
|**修复**|`{Math.round(percentage)}%`|

### 8. Dashboard 空数据状态缺少重试操作

|字段|值|
|---|---|
|**文件**|`web/src/pages/Dashboard.tsx:125-140`|
|**现状**|当 `!account || !quota || !tokens || !models` 时只显示 "暂无使用数据"，无操作按钮。而 Error 状态（上方）有重试按钮。|
|**影响**|全字段空时用户无法触发重新加载，只能等自动刷新|
|**修复**|追加重试按钮: `onClick={() => refreshAndNotify()}`，复用 error 态的按钮样式。|

### 9. History 表加载/空状态缺少骨架屏

|字段|值|
|---|---|
|**文件**|`web/src/pages/History.tsx:113-121`|
|**现状**|用量历史明细区域的 loading 和 empty 仅纯文本 "加载中…" / "暂无用量历史"，而 token 消耗图表区通过 `loadState` 统一展示 spinner。两个区 loading 样式不统一。|
|**修复**|加载态用 spinner 或 skeleton（复用 Dashboard/Models 同款 spinner）；空态用居中带 icon 的提示。|

### 10. Cost 显示精度过高

|字段|值|
|---|---|
|**文件**|`web/src/pages/History.tsx:160`|
|**现状**|`{item.cost.toFixed(6)}` — 6 位小数（如 `0.000123`）|
|**修复**|改为 4 位小数 `toFixed(4)`，或根据值动态精度（`>=1 → 2 位; <1 → 4 位`）。|

### 11. 重置时间格式不一致

|字段|值|
|---|---|
|**文件**|`src-tauri/src/lib.rs:254-267` / `mock.rs:15-16`|
|**现状**|API 路径 (`format_reset`) → `"1h 30m"` / `"45m"` / `"-"`; Mock → `"01:42:30"` / `"周五 09:00"` 两种格式混用|
|**影响**|用户看到格式不统一|
|**修复**|统一为 API 格式 `"Xh Ym"` / `"Xm"`。Mock 数据对齐。QuotaCard resetTime 的 label 文案确认是否匹配。|

### 12. AccountTable 空状态对话窗条件渲染脆弱

|字段|值|
|---|---|
|**文件**|`web/src/components/AccountTable.tsx:114` 和 `:225`|
|**现状**|`<AccountDialog>` 在空状态分支 (line 114) 和列表尾 (line 225) 各渲染一次。两个条件互斥但视觉上是一个 dialog，靠 boolean 开关激活。|
|**风险**|容易在后续重构中 break（如移除空状态分支后 dialog 消失）|
|**修复**|移除空状态内的 dialog，统一放在组件最底部（列表渲染外部），只保留一个 `<AccountDialog>`。|

### 13. Settings 遮罩与 AccountDialog 遮罩透明度不一致

|字段|值|
|---|---|
|**文件**|`Settings.tsx:15` vs `AccountDialog.tsx:78`|
|**现状**|Settings: `rgba(0,0,0,0.4)`; AccountDialog: `rgba(0,0,0,0.6)`|
|**修复**|统一为同一变量 `--color-overlay` 或统一值。推荐 0.6。|

### 14. `@layer components` CSS 工具类未被组件使用

|字段|值|
|---|---|
|**文件**|`web/src/index.css:88-124`|
|**现状**|定义了 `.card` / `.glass` / `.tag` / `.tag-*` 四类，但所有组件使用 inline `style={}` + `t.*` token 引用。CSS 类处于"已定义未引用"状态。|
|**修复**|两种路径: (a) 逐步迁移组件使用 CSS 类替换 inline style（降低 JS bundle + 利于 Tailwind 优化）; (b) 删除未使用的 CSS 类。推荐渐进走 (a)，先从 `PlanCard` / `QuotaCard` 的重复卡片样式开始。|

### 15. 空模型名在数据层未防御

|字段|值|
|---|---|
|**文件**|`src-tauri/src/collector/opencode.rs:194`|
|**现状**|`ModelBreakdown { name, ... }` 直接使用 `session.model` 原始值，SQLite 数据可能存在空字符串 model。|
|**修复**|`aggregate_local` 中对空名: (a) 过滤 (`if name.is_empty() { continue; }`) 或 (b) 替换为 `"未知"`。配合前端 #2 处理。|

---

## P3 — Low

### 16. Spinner `@keyframes spin` 重复定义

|字段|值|
|---|---|
|**文件**|`Dashboard.tsx:79`, `Models.tsx:50`, `History.tsx:84`|
|**现状**|三处独立 `<style>` 注入同名 `@keyframes spin`，无冲突但冗余|
|**修复**|提升到 `index.css` 全局 `@keyframes spin` 一次，移除各组件内联 `<style>`。同时考虑提取 `<Spinner>` 共享组件。|

### 17. 无共享 Spinner 组件

|字段|值|
|---|---|
|**文件**|`Dashboard.tsx:74-78`, `Models.tsx:48-51`, `History.tsx:82-85`|
|**现状**|三份相同 spinner JSX（24px 旋转 border + 文本 "加载中…"）|
|**修复**|提取 `components/Spinner.tsx`，接受可选 `label` prop。|

### 18. PlanCard 状态标签英文 capitalize

|字段|值|
|---|---|
|**文件**|`PlanCard.tsx:23`|
|**现状**|`textTransform: 'capitalize'` + 纯英文 `status` → "Active" / "Expired" / "Error"，与全中文 UI 不一致|
|**修复**|状态用中文: `{ status === 'active' ? '正常' : status === 'expired' ? '已过期' : '异常' }`。移除 `textTransform: capitalize`。|

### 19. 页面切换无过渡动画

|字段|值|
|---|---|
|**文件**|`App.tsx:41-50`|
|**现状**|`{page === 'dashboard' && <Dashboard/>}` 直接条件渲染切换，内容瞬间替换|
|**修复**|加 CSS transition / fade 效果，如 `opacity` 过渡 150ms，或使用简单的 `CSSTransition` 包装。|

### 20. FloatWidget 拖拽区域与按钮区可能冲突

|字段|值|
|---|---|
|**文件**|`FloatWindow.tsx:94`|
|**现状**|拖拽区域 `width: calc(100% - 48px)` 留 48px 给右上角按钮。`FloatWidget.tsx` 中按钮占约 24px+间隙，余量较小。|
|**修复**|加大留白至 60px，或改用 `pointer-events: none` 方式在按钮区域排除拖拽。|

---

## 评审总结

### 最需优先处理 (P1)
1. **百分比格式化 (#1, #6, #7)** — 三处数字原始展示，视觉效果差，修复量极小
2. **空名模型 (#2, #15)** — 前后端均需防御
3. **Mock 数据泄漏 (#3, #5)** — 到期时间和状态对真实用户产生误导
4. **Settings 空控件 (#4)** — 减少用户困惑，短期加 disabled 态

### 次优先 (P2)
- 空状态/Restry/格式一致性改善，无数据丢失风险但影响日常使用体验

### 低优先 (P3)
- 技术债：重复代码提取、CSS 类使用、微动画

### 设计系统健康度

|维度|评价|详情|
|---|---|---|
|Token 系统|✅|`tokens.ts` ↔ `index.css @theme` 绑定良好，单源真值|
|CSS 类使用|⚠️|`@layer components` 定义的工具类未使用，组件全走 inline style|
|组件内联样式|⚠️|全应用 inline `style={}` — 无 CSS-in-JS 或 utility class 一致性保障|
|i18n|⚠️|绝大部分 UI 为中文，但 PlanCard status / 部分 fallback 信息为英文|
|响应式|⚠️|侧栏 240px + 内容区固定网格 `repeat(3, 1fr)`，未测试小窗口|
