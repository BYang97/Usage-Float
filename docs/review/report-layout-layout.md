# UI 布局排版评审报告

> 评审日期: 2026-07-25
> 依据: `docs/review/layout-walkthrough.md` + 源码走读 (`web/src/pages/*.tsx` + `web/src/components/*.tsx`)
> 变更代码: 不涉及,仅报告

---

## 问题列表

### P0 — 必须修复

| # | 页面 | 问题 | 说明 |
|---|------|------|------|
| 1 | Settings | **内容区重复 Header** | `PageLayout` 已渲染全局 Header(标题"设置" + ⚙/─/✕)。内容区的 `<div className="card">` 内部又渲染了一个"设置"标题 + ✕ 关闭按钮(`onNavigate('dashboard')`)。结果: 两个"设置"标题、两个 ✕,功能重叠,视觉冗余。 |

### P1 — 高优先级

| # | 页面 | 问题 | 说明 |
|---|------|------|------|
| 2 | Dashboard | **配额卡片顺序不符合认知模型** | Bento grid 左栏为"本月额度"(大卡),右栏上下为"5小时额度"、"本周额度"。用户通常按时间范围递进查看(5小时 → 本周 → 本月),或按重要性排列。当前大卡压前,5小时/本周作为配角。 |
| 3 | Dashboard | **Header 标题"仪表盘"为静态占位符** | Header 固定显示"仪表盘",但 Sidebar 导航项已标明"首页"。更合理的做法是显示当前账号名称(`account.plan`)或账号标识,让用户确认自己正在查看哪个账号的 Dashboard。 |
| 4 | History | **内容区域层级扁平,缺乏分组标识** | `TokenUsage` 卡片(含今日/7d/30d 统计 + 折线图)与下方的"用量历史"表格为同级兄弟节点,中间无 section heading 或视觉分隔。用户无法快速定位这两部分的关系——前者是概览、后者是明细。 |
| 5 | History | **Header 标题** | 代码中已传递 `title="使用记录"`,理论上 Header 可正常显示。若运行时确实缺标题,需排查传递链是否中断;若已有,此条可降级。 |

### P2 — 低优先级/优化

| # | 页面/组件 | 问题 | 说明 |
|---|-----------|------|------|
| 6 | Sidebar | **导航图标无区分度** | 4 个导航项使用完全相同的 16×16 灰色方块,仅 active 时变为白色。用户无法通过图标快速区分页面,视觉扫描效率低。 |
| 7 | Dashboard | **Bento grid 缺少响应式后备** | `gridTemplateColumns: '2fr 1fr'` 硬编码比例。若窗口宽度不足,小卡被压缩过窄,无 `auto-fill`/`minmax` 等兜底。当前 Tauri 窗口可拖拽缩放,窄屏时布局会崩。 |
| 8 | 全局 | **内容区 padding/gap 不可按页定制** | `PageLayout` 在内容区硬编码 `padding: 16, gap: 12`。所有页面共享一样的内边距和间距。部分页面(如 Settings 居中卡片)需要更灵活的间距控制,只能靠子元素覆盖(如 `alignSelf: 'center'`)。 |
| 9 | Settings | **卡片内部滚动嵌套全局滚动** | 内容区已有 `overflowY: 'auto'`(PageLayout),Settings 卡片另设 `maxHeight: '90vh', overflowY: 'auto'`。两层滚动条,且卡片高度基于 viewport,在大屏上可能与全局滚动冲突。 |
| 10 | Sidebar | **折叠态导航不可点击图标** | 折叠后 nav items 仅显示 16px 方块,`padding: 8px`,点击热区较小。且无 tooltip 提示当前项,用户需 hover(但无 title)或展开才能确认。 |
| 11 | 全局 | **Header window 控制按钮布局** | Header 的 ⚙/─/✕ 三个按钮使用 inline `gap: 4`,无分组间距。⚙(设置)与窗口控制按钮(最小化/关闭)混排,功能类别不同。建议将 ⚙ 独立在左侧,─/✕ 居右。 |

---

## 重构建议

### 1. Settings 内容区去重 (P0)

**方案**: 移除内容区 `<div className="card">` 内部的"设置"标题行和 ✕ 按钮。

```
DEL Settings.tsx:17-21 (Title + ✕, 保留分隔线)
```

原本 `onNavigate('dashboard')` 的关闭行为可通过全局 Header 的 ✕ 或 Sidebar 导航完成。若仍需快捷返回,在 Footer 区域加一个"返回"文本链接,而非与 Header 冲突。

### 2. Dashboard 配额卡片顺序调整 (P1)

**方案 A** — 时间从小到大(推荐):

```
┌─────────────┬──────────┐
│  5小时额度   │ 本月额度  │
│  (小卡)      │  (大卡)   │
├─────────────┤          │
│  本周额度    │          │
│  (小卡)      │          │
└─────────────┴──────────┘
```

布局: 右侧大卡占 2 行,左侧 2 个小卡上下排列。`grid-template-columns: 1fr 2fr;` 5小时/本周 在左,本月在右。

**方案 B** — 按重要性: 本月(大)靠左,5h + 本周 靠右(当前布局,仅保持现状)。

**方案 C** — 纵向 flow:

```
┌──────────────────┐
│  5小时额度 (小)   │
├──────────────────┤
│  本周额度 (小)    │
├──────────────────┤
│  本月额度 (大)    │
└──────────────────┘
```

### 3. Dashboard Header 动态标题 (P1)

- 将 `PageLayout` 的 `title` 改为从 `account.plan` 或 `account` 派生,如 `account.name || account.plan`。
- 或保留"仪表盘"作为 fallback,但优先显示账号标识。

### 4. History 添加区域分组标题 (P1)

在 `TokenUsage` 卡片上方插入 `<h2>` 区段标题如"用量概览",在"用量历史"表格区上方添加"使用明细"。使用与 Sidebar 一致的导航层级语义,让用户明确两个区块的关系。

```
┌── 页面垂直流 ──┐
│ Token 概览      │  ← 新增 section title
│ [TokenUsage 卡] │
│                 │
│ 使用明细        │  ← 新增 section title
│ [用量历史 表]   │
└─────────────────┘
```

### 5. PageLayout 内容区样式可定制 (P2)

将 `padding` / `gap` 从 `PageLayout` 硬编码改为可选的 props:

```tsx
interface PageLayoutProps {
  active: string;
  title?: string;
  onNavigate: (page: string) => void;
  onMinimize?: () => void;
  onClose?: () => void;
  children: ReactNode;
  contentPadding?: number | string;  // default 16
  contentGap?: number;              // default 12
}
```

向后兼容,各页面按需覆盖。

### 6. Sidebar 导航图标差异化 (P2)

为每个导航项分配不同图标:

| 页面 | 图标建议 |
|------|----------|
| 首页 | 方块/房子 |
| 使用记录 | 折线/时钟 |
| 模型统计 | 立方体/网格 |
| 设置 | 齿轮 |

可引入简单 SVG sprite 或 unicode 字符,避免增加依赖。

### 7. Bento grid 响应式 (P2)

将 `gridTemplateColumns: '2fr 1fr'` 改为:

```css
grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
```

并配合 `grid-column: span 2` 在大卡上实现"大"的效果(仅在大屏)。

或者用 flexbox wrap:

```tsx
<div style={{ display: 'flex', flexWrap: 'wrap', gap: 12 }}>
  <QuotaCard ... style={{ flex: '2 1 300px' }} />  {/* 本月 */}
  <div style={{ flex: '1 1 150px', display: 'flex', flexDirection: 'column', gap: 12 }}>
    <QuotaCard ... />  {/* 5小时 */}
    <QuotaCard ... />  {/* 本周 */}
  </div>
</div>
```

### 8. Header 控制按钮分组 (P2)

将 ⚙ 移到标题左侧或另立分组,与窗口控制按钮(─/✕)分开:

```
[ 标题 ]  [⚙]        [─] [✕]
```

或

```
[⚙] [ 标题 ]          [─] [✕]
```

### 9. Sidebar 折叠态 hover tooltip (P2)

为折叠后的 nav button 添加 `title` 属性(已存在 collaspe 按钮的 `title`,但 nav 按钮没有):

```tsx
<button ... title={collapsed ? item.label : undefined}>
```

---

## 优先级汇总

| 层级 | 数量 | 核心问题 |
|------|------|----------|
| P0   | 1    | Settings 内容区重复 Header |
| P1   | 4    | 配额顺序、Header 动态化、History 层级、Header 标题确认 |
| P2   | 6    | Sidebar 图标/折叠态、grid 响应式、样式定制、滚动嵌套、按钮分组 |

---

## 未在本次范围

- 功能逻辑、数据加载、类型定义
- 颜色/字体/主题(token)设计
- 动画/过渡效果(除 Sidebar collapse 外)
- 响应式断点的完整方案(仅识别问题)
