# UI 布局排版评审报告：信息架构（IA）视角

**评审日期**: 2026-07-25  
**范围**: `web/src/pages/Dashboard, History, Models, Settings` + `components/PageLayout, Sidebar, Header, QuotaCard, TokenUsage` + `docs/review/layout-walkthrough.md`  
**方法**: 代码审查 + 走查记录交叉验证

---

## 全局组件层级

```
PageLayout
├── Sidebar (240px / 64px collapsed)
│   ├── Logo/Brand
│   ├── Nav: 首页|使用记录|模型统计|设置
│   └── ◀/▶ toggle
└── Main (flex:1)
    ├── Header (56px)
    │   ├── title
    │   └── [⚙] [─] [✕]
    └── Content (flex:1, overflow-y:auto, p-16, gap-12)
        └── page-specific children (flex column, gap 12)
```

所有页面共享同一骨架，差异仅在 children 区域。

---

## 问题列表

### P0 (Critical — 直接导致用户困惑或操作失误)

#### 1. Settings 页面内容区重复 Header（双重标题 + 双重关闭）

**文件**: `Settings.tsx:13-20`  
**描述**: `PageLayout` 的 `Header` 已显示标题 "设置" + ⚙/─/✕ 按钮。Settings 内容区的 `<div class="card">` 内又渲染了 "设置" 标题 + ✕ 按钮，二者完全独立。  
- Header 的 ✕ → close window  
- 内容区 ✕ → `onNavigate('dashboard')`  
- 用户无法区分两个 ✕ 的行为差异。

**代码证据**:
```tsx
// Header (PageLayout -> Header.tsx:24-31)
<span>设置</span>
<button>&#9881;</button> <button>&#9472;</button> <button>&#10005;</button>

// 内容区 (Settings.tsx:18-21)
<span>设置</span>
<button onClick={() => onNavigate('dashboard')}>&#10005;</button>
```

**建议**: 移除内容区的 title bar 整段 (`Settings.tsx:17-21`)。关闭行为在 Header 中已提供。

---

#### 2. Settings 中的 AccountTable 内嵌 QuotaCard，与 Dashboard 配额争抢语义归属

**文件**: `AccountTable.tsx:182-193`  
**描述**: AccountTable 在 Settings 页面的"账号管理"分区下对每个账号独立渲染 `QuotaCard(滚动配额/周配额/月配额)`。Dashboard 页面的 Bento Grid 也展示 `QuotaCard(本月额度/5小时额度/本周额度)`。两组数据语义重叠但展示位置不同：  
- Dashboard: 多账号聚合（只有一个 AccountInfo，来自 `getUsageData().quota`）  
- AccountTable: 按账号拆分（每个 account 独立 `fiveHourPercent/weeklyPercent/monthlyPercent`）  
- 用户不清楚 Dashboard 的配额是哪个账号的数据，也不理解 Settings 里为什么又有同样的卡片。

**建议**: 
- Dashboard 配额应标明来源账号（当前无任何账号标识）
- AccountTable 中的 QuotaCard 是"账号级配额详情"，应明确与 Dashboard "汇总概览"区分
- 短期可移除 AccountTable 的 QuotaCard，改为纯文字数字显示；长期考虑统一配额数据模型

---

### P1 (Important — 影响导航效率和信息查找)

#### 3. Header 标题与 Sidebar 导航项命名不一致

| Sidebar | Header | 差异 |
|---------|--------|------|
| 首页 | 仪表盘 | Sidebar 用"首页"（Home），Header 用"仪表盘"（Dashboard） |
| 使用记录 | 使用记录 | 一致 |
| 模型统计 | 模型统计 | 一致 |
| 设置 | 设置 | 一致 |

"首页" vs "仪表盘" 指向同一页面，用户侧两个词都有可能出现，产生认知混淆。

**建议**: 统一命名。推荐使用"仪表盘"（与功能定位一致）或"首页"二者择一。

---

#### 4. Dashboard 配额卡片顺序不符合用户优先级

**文件**: `Dashboard.tsx:128-136`  
**布局**: 2fr + 1fr grid。
```
┌─────────────┬──────────┐
│             │ 5小时额度 │
│  本月额度    ├──────────┤
│  (大卡)     │ 本周额度  │
│             │ (小卡)    │
└─────────────┴──────────┘
```

- 本月（最宽松、最长周期，占比 91%）放在左侧占用 2fr + 跨行，视觉权重最大
- 5小时（最紧迫、最短周期，占比 0%）反而在右侧小卡，视觉权重最小

**建议**: 按约束紧迫度排列：5小时（最紧）→ 本周 → 本月。建议 grid 改为 `1fr 1fr 1fr` 三列等宽，或仍是 2fr+1fr 但把 5小时额度作为大卡。

---

#### 5. TokenUsage 组件在 Dashboard 和 History 之间语义重叠

- `Dashboard.tsx:141`: `<TokenUsage .../>` — 作为当前 Token 消耗状态展示
- `History.tsx:129`: `<TokenUsage .../>` — 完全复用同一组件

两个页面使用完全相同的组件、接收相同的 `tokens` 数据（`tokenToday/token7d/token30d/tokenHistory`）。History 页面应有更丰富的历史对比能力（多时间段叠加、日期范围选择），而不仅是重复 Dashboard 的"今日/7天/30天"摘要。

**建议**:
- Dashboard 的 TokenUsage 保留作为快速摘要
- History 的 TokenUsage 应扩展为趋势组件：支持日期范围选择、多账号对比、缩放等 Dashboard 不具备的能力
- 或将 TokenUsage 拆分为 `TokenSummaryCard`（摘要版）和 `TokenTrendChart`（历史版）

---

#### 6. Models 页面与 Dashboard 功能重复，独立价值不清晰

**文件**: `Models.tsx:70-71` → 只渲染 `<ModelUsage models={models} />`  
Dashboard 底部 `Dashboard.tsx:143-145` 同样渲染 `<ModelUsage models={models} />`，数据源完全一致。Models 页面没有额外筛选、时间维度、排序能力。

**建议**:
- 方案 A: 删除 Models 页面，将 ModelUsage 作为 Dashboard 的固定区块（已是）；调整 Sidebar 到 3 项
- 方案 B: 保留 Models 页面但赋予差异化能力（按日期范围过滤、按模型名称搜索、趋势对比图），否则作为单独页面无意义

---

### P2 (Polish — 视觉/对齐/微结构问题)

#### 7. Dashboard Bento Grid 视觉不平衡

**文件**: `Dashboard.tsx:129-136`  
`gridTemplateColumns: '2fr 1fr'` 左侧月度额度卡（跨两行）vs 右侧两个小卡的高度差。右侧总高度 = 5h卡 + gap + 周卡，左侧一张大卡高度由内部内容撑起。两侧高度不严格对齐（QuotaCard 内部有 resetTime 时才会显示"重置：xx"，否则更短）。

**建议**: 
- 使用 `grid-template-rows: auto auto` 确保左右对齐
- 或等卡片都加载完后再渲染（当前已有 Skeleton，正常态不会有闪烁）

#### 8. Settings 双重滚动容器

- `PageLayout.tsx:32`: 内容区 `overflowY: 'auto'`
- `Settings.tsx:15`: card 内 `overflowY: 'auto'`

如果 Settings 内容超过卡片高度，外层 scroll 和内层 scroll 同时出现，操作层级混乱。

**建议**: 移除 inner card 的 `overflowY`，让内容撑高 card，外层负责滚动；或增大 card `maxHeight`。

#### 9. Sidebar 折叠态导航项无视觉区分

**文件**: `Sidebar.tsx:86-93`  
collapsed 时每个 nav item 只显示一个 16×16 彩色方块（`background`），不同页面之间无图标差异，无法区分当前位置是哪个页面。

**建议**: collapsed 时，为每个 nav item 赋予不同的 icon/符号，而非仅一个统一方块。

#### 10. Settings 页面大部分控件为"即将支持"占位符，内容密度低

**文件**: `Settings.tsx:26-73`  
通用、外观、隐私三组设置中所有可交互控件处于 disabled 状态并带有 "即将支持" 标签。实际可操作的仅有 AccountTable。页面内容与页面标题（全局设置）不符。

**建议**:
- 将"即将支持"项折叠或灰显，减少视觉噪声
- 或将 AccountTable 提升为设置页面的主要/唯一内容，待其他设置项实现后再展开

---

## 重构建议汇总

### 架构级重构（建议排期）

| # | 建议 | 涉及文件 | 优先级 | 工作量 |
|---|------|---------|--------|--------|
| R1 | 统一 Header/Sidebar 命名 | `Sidebar.tsx` items → `Header.tsx` title props | P1 | 小 |
| R2 | 移除 Settings 内容区 title bar | `Settings.tsx:17-21` | P0 | 小 |
| R3 | AccountTable 去 QuotaCard / 明确数据归属 | `AccountTable.tsx:182-193`, `Dashboard.tsx` | P1 | 中 |
| R4 | Dashboard 配额卡片重排（5h → 周 → 月） | `Dashboard.tsx:128-136` | P1 | 小 |
| R5 | TokenUsage 拆分或差异化 History 版本 | `TokenUsage.tsx`, `History.tsx:129` | P1 | 中 |
| R6 | Models 页面重新定位或移除 | `Models.tsx`, `Sidebar.tsx` items | P1 | 中 |
| R7 | Sidebar collapsed 图标差异化 | `Sidebar.tsx:86-93` | P2 | 小 |
| R8 | Settings 双重 scroll 修复 | `Settings.tsx:15` | P2 | 小 |
| R9 | Dashboard 网格对齐修复 | `Dashboard.tsx:129` | P2 | 小 |

### IA 重组方案（建议的页面职责划分）

```
┌──────────┬──────────────────────────────────────┬─────────────────────────┐
│ 页面     │ 职责                                 │ 包含组件                │
├──────────┼──────────────────────────────────────┼─────────────────────────┤
│ 仪表盘   │ 当前状态总览：订阅 + 配额 + Token    │ PlanCard, QuotaCard x3, │
│          │ + 模型分布，一屏扫完                   │ TokenSummaryCard,       │
│          │                                      │ ModelUsage              │
├──────────┼──────────────────────────────────────┼─────────────────────────┤
│ 使用记录 │ 历史趋势 + 明细查询                   │ TokenTrendChart,        │
│          │                                      │ UsageTable,             │
│          │                                      │ 日期筛选器, 账号筛选器    │
├──────────┼──────────────────────────────────────┼─────────────────────────┤
│ 设置     │ 应用配置（通用/外观/隐私）+ 账号管理  │ Section, Row,           │
│          │ (不展示配额卡片)                       │ AccountTable(去Quota)   │
└──────────┴──────────────────────────────────────┴─────────────────────────┘
```

### 关键 Data Flow 问题

```
Dashboard ← getUsageData() → 聚合配额(单账号)
AccountTable ← invoke('list_accounts') → 逐个配额
```

Dashboard 的 `getUsageData()` 返回的是哪个账号的数据？代码中无账号选择器。对比 AccountTable 支持账号切换 + 各自的 QuotaCard。这意味着 Dashboard 可能只显示"第一个账号"或"默认账号"的配额，但没有任何标签说明这一点。

**建议**: Dashboard 也应允许选择账号，或明确标示当前展示的账号 ID。

---

## 优先级总结

| 优先级 | 计数 | 关键问题 |
|--------|------|---------|
| **P0** | 2 | Settings 双重 title, AccountTable 内嵌 QuotaCard 语义冲突 |
| **P1** | 4 | Header/Sidebar 命名不一致、配额顺序、TokenUsage 重复、Models 无独立价值 |
| **P2** | 4 | grid 视觉平衡、双重 scroll、Sidebar 图标、Settings placeholder 噪声 |

---

*本报告仅做 IA 评审，不涉及代码实现修改。*
