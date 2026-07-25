# Layout Hierarchy 评审报告

> 评审日期: 2026-07-25 | 基于代码: web/src/pages/{Dashboard,History,Models,Settings}.tsx + components/{PageLayout,Sidebar,Header,QuotaCard,TokenUsage,ModelUsage,PlanCard,AccountTable}
> 优先级: P0(断裂) / P1(重要) / P2(改进)

---

## P0 — 关键缺陷

### 1. 字体大小变量未定义 [P0]

**文件**: `web/src/index.css` + `web/src/tokens.ts`

`tokens.ts` 中所有 `fs-*` 引用 (`fsH1`…`fsHero`) 均映射为 `var(--fs-h1)` 等 CSS 变量，但 `index.css` 的 `@theme` 块中**从未定义这些变量**。

| 引用 | 定义状态 | 实际值 |
|------|----------|--------|
| `--fs-h1` / `--fs-h2` / `--fs-h3` | 未定义 | 退回继承值(13px) |
| `--fs-body` | 未定义 | 退回 13px |
| `--fs-secondary` | 未定义 | 退回 13px |
| `--fs-weak` | 未定义 | 退回 13px |
| `--fs-hero` | 未定义 | 退回 13px |

**影响**: 整个视觉层级系统的字体比例完全断裂。Header 标题 `t.fsH2`、卡片标题 `t.fsH3`、小字 `t.fsWeak` 全部渲染为相同的 13px，所有组件之间无大小层级区别。

**修复**: 在 `@theme {}` 中补充:

```css
--fs-hero: 28px;
--fs-h1: 22px;
--fs-h2: 17px;
--fs-h3: 15px;
--fs-body: 13px;
--fs-secondary: 12px;
--fs-weak: 11px;
```

---

## P1 — 重要层级问题

### 2. Settings 页面: 双重标题 + 双重关闭 [P1]

**文件**: `web/src/pages/Settings.tsx`

`PageLayout > Header` 渲染了标题"设置" + 按钮 `⚙ ─ ✕`，同时 Settings 内容区卡片内又独立渲染了一个"设置"标题 + `✕`：

```
Header:         [设置]                                [⚙] [─] [✕]
Content card:   [设置]                                                    [✕]
                ──────────────────
                [通用设置]
                [外观设置]
                [隐私设置]
                [账号管理]
```

问题:
- 两个"设置"标题 —— Header 一个、内容区一个，角色重复
- 两个 `✕` 关闭按钮 —— Header 的关闭整个应用，内容区的只导航回首页，语义混乱
- Header 中的 `⚙` 设置齿轮在当前页面(设置)是自我引用，无操作意义

**建议**: 去掉内容区卡片内的"设置"标题和关闭按钮，信息完全由 Header 承载。如有必要在内容区保留额外导航，改为 Back 箭头。

### 3. Dashboard 配额卡片排序违背信息优先级 [P1]

**文件**: `web/src/pages/Dashboard.tsx:129-136`

当前顺序(从左到右、从上到下):

```
┌──────────────────┬──────────────┐
│                  │  5小时额度    │
│  本月额度 (2fr)   │  xx%         │
│                  ├──────────────┤
│                  │  本周额度     │
│                  │  xx%         │
└──────────────────┴──────────────┘
```

问题:
- 本月额度(时间跨度最长，重置最晚)占据最大的视觉面积(2fr)，但用户最需要关注的是**即将重置的配额**(5小时滚动额度 → 本周额度 → 本月额度)
- 5小时额度是最紧迫的(几小时内重置)，却被压缩在右列的顶部小卡片中
- 视觉权重与信息紧急度倒挂

**建议**: 按重置紧迫度排列: 5小时(左，大) + 本周 + 本月(右列，小); 或保持 2fr 但给 5小时额度。

### 4. 页面间 Header 标题不一致 [P1]

**文件**: `web/src/components/Header.tsx` + 各 page

| 页面 | 实际标题 | 备注 |
|------|----------|------|
| Dashboard | "仪表盘" | 但该页面展示的是用户 plan/配额/Token，标题应为账号名或更具体的信息 |
| History | "使用记录" | 正确 ✅ |
| Models | "模型统计" | 正确 ✅ |
| Settings | "设置" | Header 标题"设置"但内容区又重复一个"设置" |

Dashboard 标题"仪表盘"是一个通用占位词，在额度消耗场景下缺乏信息量。建议改为 `{plan}` 或用户可识别的标识(如账号名)。

---

## P2 — 改进建议

### 5. Dashboard Bento Grid 层级扁平 [P2]

**文件**: `web/src/pages/Dashboard.tsx:123-147`

内容区三大块(PlanCard / 配额网格 / Token + Model)使用 `gap: 12`，与 PageLayout 的 `gap: 12` 相同。页面上所有元素之间的间距都是一致的，无法体现"区块 vs 区块内"的层级关系。

建议:
- 区块间距 `gap: 16~20`，区块内间距 `gap: 12`，形成嵌套层级
- 为 Token 区域和 Model 区域增加分组视觉容器(浅灰背景或顶部横线)，与配额卡片区分层级

### 6. TokenUsage 图表高度固定 140px [P2]

**文件**: `web/src/components/TokenUsage.tsx:22`

折线图区域 `height: 140px` 固定，在总空间充足时也不会扩展，导致 Token 消耗卡片的剩余空间浪费，图表信息密度低。

建议: 设为 `min-height: 140px; flex: 1` 以自适应容器高度。

### 7. ModelUsage 固定宽度导致溢出风险 [P2]

**文件**: `web/src/pages/Dashboard.tsx:143`

`<div style={{ width: 296, flexShrink: 0 }}>` —— ModelUsage 固定 296px。当窗口缩小时，flex 容器中 TokenUsage(`flex: 1`) 被压缩，极端宽度下内容溢出。

建议: 使用 `min-width` + `flex-basis` 替代固定宽度，或添加响应式断点。

### 8. 配额本体号: 仅百分比，缺少绝对数值 [P2]

**文件**: `web/src/components/QuotaCard.tsx`

`QuotaCard` 只显示百分比 `${displayValue}%`，不显示实际用量/限额(如 "4.5h / 5h")。百分比视觉强烈但信息不完整。

建议: 在百分比下方或 ProgressBar 旁边补充 `已用/总量` 显示。

### 9. AccountTable 中 QuotaCard 三列拥挤 [P2]

**文件**: `web/src/components/AccountTable.tsx:183-193`

Settings 内 AccountTable 的每个账号卡片中，三个 QuotaCard 以 `flex: 1` 排列。在 480px 宽的 Settings 卡片内三列配额卡片极度拥挤，各卡片标题截断。

建议: 改为 2+1 或垂直堆叠布局，或为 AccountTable 内的 QuotaCard 设置最小宽度。

### 10. PlanCard vs QuotaCard 视觉权重相同 [P2]

**文件**: `web/src/components/PlanCard.tsx` vs `QuotaCard.tsx`

订阅计划卡(PlanCard)应比配额卡(QuotaCard)具有更高的视觉权重，但两者均使用 `padding: 16` + `.card` class，视觉层级完全相同。订阅计划是其他所有数据的上下文基础。

建议: PlanCard 增加内边距(20px)或增加左侧蓝色强调边框/accent 色点缀提升权重。

### 11. Sidebar 折叠动画问题 [P2]

**文件**: `web/src/components/Sidebar.tsx`

- `transition: 'width 0.2s ease'` 只平滑宽度变化，但内部元素(Logo 文字/导航文字/折叠按钮箭头)无 opacity transition，从 240px→64px 时文字因 `overflow: hidden` 突然截断
- 折叠按钮文本 `◀` / `▶` 应配合宽度动画做 fade 过渡，避免突兀跳跃

建议: 给内部文字容器加 `opacity` 过渡; 折叠态隐藏全程而非仅在 overflow 裁切后。

### 12. PageLayout 内容区 gap 无层次 [P2]

**文件**: `web/src/components/PageLayout.tsx:31`

`padding: 16, gap: 12` 作为内容区的通用间距。但"页面内容"和"内容内部的区块"共享同一 gap 值，视觉效果扁平。

建议: PageLayout 用 `gap: 16` 作为一级间距，区块内部各自控制二级间距。

### 13. History 用量表格缺少 stripes / zebra [P2]

**文件**: `web/src/pages/History.tsx:161-188`

Table 行间无交替背景色(斑马线)，每行仅靠底部分割线区分。密集数据时横向扫读容易跳行。

建议: 添加 tr:nth-child(even) 淡背景，或增加 hover 高亮。

---

## 重构建议摘要

| 区域 | 建议 | 优先级 |
|------|------|--------|
| `index.css @theme` | 补充 `--fs-*` 字体大小变量定义 | P0 |
| `Settings.tsx` | 删除内容区重复的标题 + close | P1 |
| `Dashboard.tsx` | 将急迫配额(5h/本周)放在本月之前 | P1 |
| `Header.tsx` / pages | 统一标题语义 | P1 |
| `PageLayout.tsx` | 增大一级间距(gap: 16)区分层级 | P2 |
| `TokenUsage.tsx` | 图表高度自适应(flex) | P2 |
| `Dashboard.tsx` | ModelUsage 固定宽度改为响应式 | P2 |
| `QuotaCard.tsx` | 补充绝对数值(已用/总量) | P2 |
| `AccountTable.tsx` | 账号内配额卡片改用多行布局 | P2 |
| `PlanCard.tsx` | 提升视觉权重比 QuotaCard 高一级 | P2 |
| `Sidebar.tsx` | 折叠时内部元素 opacity 动画 | P2 |
| `History.tsx` | 表格加斑马线/hover 高亮 | P2 |

---

## 视觉层级现状总结

当前层级结构(预期 vs 实际):

```
预期:                       实际(因 --fs-* 未定义):
  Header   fsH2 17px   →    Header   13px (body)
  PlanCard fsH2 17px   →    PlanCard 13px
  Quota %  fsHero? 不适用 →  inline font-size 36px (硬编码)
  卡片标题  fsH3 15px   →    卡片标题  13px
  正文字体  fsBody 13px →    正文字体  13px
  次要文字  fsSecondary 12px → 次要文字 13px
```

整个字体层级链断裂，所有文字实际渲染为同一大小(13px)。当前唯一有效的视觉层级来自于:
- **36px 硬编码**配额百分比(QuotaCard) —— 但只有这一个数字突出
- **卡片阴影** (`.card:hover` hover 效果) —— 区分卡片与背景
- **颜色** (蓝 `accentBlue`、绿 `statusOk`、红 `statusDanger`)

**最大问题(单一修复收益最高)**: 修复 `--fs-*` 变量后，标题/正文/辅助文字的大小层级自然恢复，当前的 60% 层级问题自动解决。
