# Usage Float 设计规格 v8

> 桌面应用设计规格。所有设计稿必须遵循本规格，确保设计稿与实际开发窗口一致。

## 1. 应用窗口（来源：src-tauri/tauri.conf.json）

| 窗口 | 尺寸 W×H | 最小尺寸 | 特性 |
|---|---|---|---|
| 主窗口 main | **1000×700** | 800×600 | resizable, center, 圆角 16 |
| 悬浮窗 float | **320×180** | - | 固定, decorations=false, transparent, alwaysOnTop, skipTaskbar |

## 2. 主窗口布局

```
┌───────────┬─────────────────────────────────┐
│           │ Header  56px                    │
│ Sidebar   ├─────────────────────────────────┤
│ 240px     │ 内容区  760×644                 │
│ (折叠64)  │ padding:16  gap:12             │
│           │ overflow-y:auto                │
└───────────┴─────────────────────────────────┘
   240           760
                 1000
```

- **Sidebar** 240px（折叠 64px）：玻璃白，logo 区(56) + nav(4 项) + 折叠按钮
- **Header** 56px：左标题(H1 20px) + 右设置⚙/最小化─/关闭✕
- **内容区** 760×644：padding 16, gap 12, 垂直滚动

## 3. 栅格系统

内容区可用宽 = 760 - 32(padding) = **728px**
12 列网格，列间距 16px：
- 列宽 = (728 - 11×16) / 12 ≈ 45px
- 常用组合：12(满宽) / 6+6 / 4+4+4 / 8+4 / 3+3+3+3

## 4. 间距 token

| token | 值 | 用途 |
|---|---|---|
| page-padding | 16 | 内容区 padding |
| page-gap | 12 | 内容区子项 gap |
| card-padding | 16 | 卡片 padding |
| card-gap | 12 | 卡片内 gap |
| section-gap | 16 | 分区间距 |
| control-gap | 8 | 控件间 |

## 5. 字号 token

| 级别 | size | weight | 用途 |
|---|---|---|---|
| H1 | 20 | 700 | 页面标题(Header) |
| H2 | 18 | 600 | 区块标题 |
| H3 | 15 | 600 | 卡片标题 |
| Body | 14 | 400 | 正文 |
| Secondary | 13 | 400 | 次文本 |
| Weak | 12 | 400 | 弱文本 |
| Micro | 10-11 | 500-600 | label/badge/time |

## 6. 组件规格

### Card
- 圆角 **16**, padding 16, shadow-card `0 1px 3px rgba(0,0,0,0.04),0 1px 2px rgba(0,0,0,0.02)`, hover shadow-hover + translateY(-1px)
- 预警变体：card-warning-yellow(左边框 3px #f59e0b) / card-warning-red(左边框 3px #ef4444 + 背景渐变 rgba(239,68,68,0.06)->#fff)

### Button
- Primary: 渐变 accent, 圆角 12, padding 8×16, 高 **36**, 白字
- Secondary: 白底 + border #e9ecef, 圆角 12
- Text: 无背景, accent/红

### Badge
- 圆角 9, padding 2×8, 字 10px/600
- lite: 渐变紫 / go: 渐变绿青 / error: #ef4444

### 圆环 Ring
- Dashboard: 56×56, stroke 4, viewBox 0 0 52 52, r=22
- Float: 80×80, stroke 6, viewBox 0 0 80 80, r=34
- 颜色：<50% #10b981 / 50-80% #f59e0b / >80% #ef4444
- >90% 脉动动画 `pulse 1.8s`

### 表格 Table
- 表头 11px #6b7280, padding 8×6, 底边框 #e9ecef
- 行 13px #1a1d21, padding 8×6, 底边框 #f1f3f5, hover #f8f9fa
- 数字列右对齐 tabular-nums

## 7. 页面规格

**每个主窗口页面设计稿画板尺寸 = 1000×700（完整窗口）**，必须包含 Sidebar(240) + Header(56) + 内容区(760×644)。不再用独立 600px panel。

| 页面 | 内容区(760×644)布局 |
|---|---|
| Dashboard | 账号卡片纵向(满宽 728) + token 汇总(3列) + 模型 Top2 |
| History | token 汇总(3列) + 筛选栏 + 表格(满宽) + 分页 |
| Models | 模型使用(满宽) + 模型卡片网格(3列) |
| Settings | 单卡片(480 居中) 分区 |
| Float | 320×180 独立窗口 |
| States | 4 状态画板（每个 760×644 内容区）|

## 8. 配色（9 色 token）

| token | hex | 用途 |
|---|---|---|
| surface | #f8f9fa | 主背景 |
| surfaceAlt/card | #ffffff | 卡片背景 |
| surfaceHover | #f1f3f5 | hover |
| surfaceBorder | #e9ecef | 边框 |
| textPrimary | #1a1d21 | 主文本 |
| textSecondary | #6b7280 | 次文本(原 #9ca3af 提对比度) |
| accentBlue | #6366f1 | 强调(渐变起点) |
| accentGradient | linear-gradient(135deg,#6366f1,#8b5cf6) | 渐变 accent |
| statusOk/Warning/Danger | #10b981/#f59e0b/#ef4444 | 状态色 |

## 9. 设计稿规格标注

每个设计稿画板上方必须标注：
- 画板尺寸（1000×700 主窗口 / 320×180 悬浮窗）
- 页面名 + 序号
- 关键区域尺寸（Sidebar 240 / Header 56 / 内容区 760×644）
