# UI 布局排版评审报告

**评审日期**: 2026-07-25  
**评审范围**: `web/src/pages/{Dashboard,History,Models,Settings}.tsx` + `web/src/components/{PageLayout,Sidebar,Header,QuotaCard,TokenUsage,PlanCard,ModelUsage,AccountTable,Skeleton,ProgressBar}.tsx`  
**参考**: `docs/review/layout-walkthrough.md`  
**方法**: 代码静态分析 + 间距/层级/排版一致性审查

---

## 优先级定义

| 等级 | 含义 |
|------|------|
| **P0** | 破坏性 BUG：功能不正常或用户不可见,须立即修复 |
| **P1** | 明显不一致：间距/层级混乱,影响视觉质量和可维护性,应尽快修复 |
| **P2** | 次要/边缘：一致性提升,可等下次重构 |

---

## P0 — 字体大小 CSS 变量缺失（全组件层级扁平）

**发现**: `--fs-h1`, `--fs-h2`, `--fs-h3`, `--fs-body`, `--fs-secondary`, `--fs-weak`, `--fs-hero` 在 `index.css` 的 `@theme` 中**从未定义**。

**影响**: 所有组件通过 `t.fsH2` / `t.fsBody` / `t.fsSecondary` 等引用这些变量,但 CSS `var(--fs-h2)` 等解析为 `unset`,最终全部 fallback 到 `html { font-size: 13px }`。Header 标题、卡片标题、副文本、弱文本——所有字号实际相同(13px)。**Typography 层级完全扁平化**,用户无法区分标题与正文。

**位置**:
- 根源: `web/src/index.css` — `@theme` 块缺失 `--fs-*` 定义
- 所有消费点: `tokens.ts` 第 21-27 行 → 被 `Header.tsx:24`, `QuotaCard.tsx:37/42`, `TokenUsage.tsx:9/18`, `Sidebar.tsx:51/76`, `PlanCard.tsx:14/19`, `ModelUsage.tsx:8/12/13`, `Settings.tsx:19/54/58/71/89/98`, `Dashboard.tsx:97`, `History.tsx:110/122/134/139/161/164/167/177/198/203`, `AccountTable.tsx:132/157/160/174/195/201` 等引用

**建议**:
```
@theme {
  + --fs-hero:     24px;
  + --fs-h1:       20px;
  + --fs-h2:       17px;
  + --fs-h3:       15px;
  + --fs-body:     13px;
  + --fs-secondary: 12px;
  + --fs-weak:     11px;
}
```
同时在 `index.css` 中将 `html { font-size: 13px }` 改为引用 `--fs-body` 以保持单一来源。

---

## P1 — 间距/层级明显问题

### 1.1 PageLayout 内容区间距硬编码

**发现**: `PageLayout.tsx:30-33` 内容区 `padding: 16, gap: 12` 直接内联写死,没有使用 CSS 变量。

**评价**: 注释自称 `p-4 gap-3`,但 Tailwind 的 `p-4` = 16px(正确),而 `gap-3` 在 Tailwind v4 为 12px(正确)。问题是这些值散布在多个组件中,无法通过修改一个变量统一调整。

**涉及**:
- `PageLayout.tsx:32` — `padding: 16, gap: 12`
- `Dashboard.tsx:129` — grid `gap: 12`
- `Dashboard.tsx:139` — flex `gap: 12`

**建议**: 抽取为 `--spacing-content-padding: 16px` 和 `--spacing-content-gap: 12px`,由 `tokens.ts` 引用。

### 1.2 Dashboard Bento Grid 配额排列与权重倒置

**发现**: `Dashboard.tsx:129-136` 用 `gridTemplateColumns: '2fr 1fr'` 把"本月额度"放大为两列宽卡片,将"5小时额度"和"本周额度"压缩到右侧小卡。

**问题**:
- 5小时(有 resetTime,用户需关注剩余)和本周(有 resetTime)才是高频查看配额
- 本月额度没有 resetTime 也缺少 urgency signal,不应占据 2fr 视觉权重
- 布局将 "本月额度" 放在视觉主轴首位,被打为最重要的配额,信息架构颠倒

**建议**: 按时间紧迫度排列: `5小时 → 本周 → 本月`,可改为 3 列等宽卡,或 `1fr 1fr 1fr`。

### 1.3 Settings 卡片内重复 Header（标题 + ✕）

**发现**: `Settings.tsx:14-21` — 卡片内又渲染了一个"设置"标题和 ✕ 关闭按钮,完全重复了 PageLayout Header 的 `title="设置"` 和 `Header.tsx` 提供的关闭按钮。

**冗余元素**:
- Header: `title="设置"` + ⚙/─/✕ 按钮组
- 卡片内: `<span>设置</span>` + ✕ → navigate to dashboard

**建议**: 移除卡片内的标题行和 ✕(line 18-21 的整个 `div`),或将其改为纯关闭引导。如果卡片需要独立的 dismiss 路径,用文字"返回首页"而非重复 ✕。

### 1.4 Header 标题语义不一致

**发现**:
| 页面 | Header title | 问题 |
|------|-------------|------|
| Dashboard | "仪表盘" | 应为账号名或"使用概览" |
| History | "使用记录" | 正确,但 walkthrough 说"缺标题"(实际已传 title) |
| Settings | "设置" | 与卡片内重复 |

Dashboard 页通常显示用户/账号名称,硬编码"仪表盘"在全局导航语境中无区分度。

### 1.5 QuotaCard / TokenUsage / ModelUsage padding 来源分散

| 组件 | padding | 来源 |
|------|---------|------|
| `QuotaCard.tsx:34` | `padding: 16` | 硬编码 |
| `TokenUsage.tsx:8` | `padding: 16` | 硬编码 |
| `ModelUsage.tsx:7` | `padding: 16` | 硬编码 |
| `PlanCard.tsx:9` | `padding: 16` | 硬编码 |

全部 card 都用 `className="card"` + 31-36px font × gap × padding,但缺少统一变量。改用 `--spacing-card-padding: 16px` 统一。

### 1.6 History 页面 TokenUsage + 表格之间缺少分割

**发现**: `History.tsx:129-207` — TokenUsage 卡片和"用量历史"卡片在 PageLayout 内容区 `gap: 12` 下依次排列。但两张.card 之间无额外分割,且没有 section 标题来提示"Token 消耗"和"用量历史"是不同的内容域。

**建议**: 在每个 section 前增加区域标题,或在两张 card 之间使用 `gap: 16` 而非 `gap: 12` 以增强视觉分离。

### 1.7 PlanCard gap:40 与全局间距不协调

**发现**: `PlanCard.tsx:9` — flex `gap: 40` 是全局缝隙值最大的一处。组件其余部分用 `gap: 4`/`gap: 6`,40px 的 icon→plan name 间距让 icon 视觉孤立。

**建议**: 改为 `gap: 16` 或 `gap: 20`,与其他 card 的水平间距一致。

### 1.8 AccountTable 内外间距不一致

**发现**:
- 外层容器 `gap: 12` (line 129)
- 内部 card `padding: 16` (line 148)
- 空数据提示 `padding: '8px 0'` (line 195)
- 错误提示 `padding: '4px 0'` (line 201)

缺乏统一的 `--spacing-card-inner-gap` 等变量,且 padding 值手动散布。

---

## P2 — 次要一致性问题

### 2.1 Sidebar 分割线 margin 与 nav padding 不匹配

**发现**: `Sidebar.tsx:58` — 分割线 `marginLeft: 16, marginRight: 16`,但 nav 区 `padding: 10` (line 61)。分割线宽度 = sidebar - 32px,而 nav items 从 padding 10 开始。

**结果**: 分割线比 nav items 左右各宽 6px,视觉对齐失效。

**建议**: 将分隔线 margin 改为 `marginLeft: 10, marginRight: 10` 匹配 nav padding,或统一二者使用同一变量。

### 2.2 History 翻页按钮 padding 单独

**发现**: `History.tsx:192` — `padding: '12px 0'`,但其他类似按钮/loading 使用 `padding: '8px 0'`(AccountTable)或 `padding: '4px 0'`(error)。

**建议**: 翻页按钮用统一的 `--spacing-btn-padding-y`。

### 2.3 Settings card 宽度固定 480px

**发现**: `Settings.tsx:15` — `width: 480, alignSelf: 'center'`。Tauri 窗口最小宽度约 720px 时 card 居中且两侧空白大,扩大窗口时浪费大量空间;缩小窗口时 card 可能截断。

**建议**: 改为 `max-width: 540px, min-width: 320px` 配合 `width: 100%`,让小窗口也能正常使用。

### 2.4 QuotaCard ProgressBar 的 height 硬编码

**发现**: `QuotaCard.tsx:48-51` — ProgressBar 使用默认 `height = 8`,而 `TokenUsage.tsx` 的 `ResponsiveContainer` 高度为 `140px`(line 37), `ModelUsage.tsx` 的自定义 bar `height: 8`。进度条高度没有纳入间距规范。

### 2.5 PlanCard expireDate 条件渲染导致 flex 抖动

**发现**: `PlanCard.tsx:16-21` — `expireDate` 为空时不渲染中间的日期区块,影响 flex 对齐,日期不存在时 icon+plan+status 三元素拉宽。

**建议**: 用固定宽度容器包裹日期区块,或统一使用 `display: grid` 固定列宽。

### 2.6 AccountTable 按钮间隙 gap 不统一

**发现**: `AccountTable.tsx:208` — action 按钮 `gap: 8`,但 header 的 `添加账号` 按钮和账号 name 之间使用 `gap: 12`。无明确理由不同的 gap。

### 2.7 History 表格 cell padding 偏小

**发现**: 表格 `th/td` 使用 `padding: '6px 8px'` (line 164-172)。对比 Settings Row `height: 32`(含内容和 padding),表格 6+6=12px 垂直 padding 在 13px 字体下 touch target 不足。

**建议**: 改为 `padding: '8px 10px'` 增加可点击/可读区域。

---

## 总结

| 优先级 | 数量 | 关键问题 |
|--------|------|---------|
| **P0** | 1 | `--fs-*` 变量全未定义,全应用 13px |
| **P1** | 8 | 间距硬编码、Dashboard 配额顺序、Settings 重复标题、Header 语义、card padding 分散、History 缺乏分区、PlanCard gap 过大、AccountTable 间距不统一 |
| **P2** | 7 | Sidebar 分割线对齐、翻页按钮 padding、Settings 宽度固定、ProgressBar 规范、PlanCard 条件渲染抖动、按钮 gap 不统一、表格 cell 偏小 |

**推荐规范变量(新加)**:
```
--spacing-content-padding: 16px
--spacing-content-gap:     12px
--spacing-card-padding:    16px
--spacing-card-gap:        12px
--spacing-section-gap:     8px
--spacing-row-gap:         4px
```
配合 `--fs-*` 补齐,实现单一事实来源。
