# v6 设计稿审查（openpencil skill + lint/analyze 原则）

审查范围：5 个 HTML 设计稿（docs/design/monitor-v6-*.html）
审查依据：openpencil skill review-template（功能覆盖）+ lint（consistency/structure/accessibility）+ analyze（colors/typography/spacing/clusters）

## 1. 功能覆盖审查（skill review-template）

| 页面 | 必需功能 | Found | 备注 |
|---|---|---|---|
| Dashboard | 3 账号 + 圆环(5h/周/月) + 重置时间 + token 今日/30天 + 汇总 + 模型使用 | Yes | 完整 |
| History | token 汇总(今日/7天/30天) + 用量历史表(8列) + 账号选择器 + 加载更多 | Yes | 完整 |
| Models | 模型使用进度条 + 模型卡片(调用/token/费用) | Yes | 完整 |
| Settings | 通用/外观/隐私/账号管理 + toggle/chip/badge/添加账号 | Yes | 完整 |
| Float | 圆环 + ProviderBadge + 关闭/最小化 + 5h 额度 + 重置 + 打开仪表盘 | Yes | 完整 |

功能覆盖率：5/5 = 100%

## 2. 视觉一致性（lint consistency）

| 维度 | 状态 | 详情 |
|---|---|---|
| 配色 | ✅ | 9 色统一：#f8f9fa/#fff/#1a1d21/#6b7280/#9ca3af/#6366f1/#10b981/#f59e0b/#ef4444 |
| 字体 | ✅ | Inter + 分级 20/14/13/11/10 |
| 圆角 | ✅ | 12 卡片 / 9 badge / 6 按钮 / 4 select |
| 间距 | ✅ | 16 padding / 12 gap / 8 cell |
| 卡片 | ✅ | 白 + shadow-card + hover shadow-hover + translateY(-1px) |
| badge | ✅ | 渐变 lite(紫) / go(绿青) |
| accent | ✅ | linear-gradient(135deg,#6366f1,#8b5cf6) |

无游离色/无字号越界/无圆角不一致。

## 3. 结构（lint structure）

- panel 600px 居中 + header(H1 20px + sub 11px) + section-title(13px) + card ✅
- 层级清晰：H1 > section-title > card 内容 ✅
- 组件化：card/badge/button/table 跨页复用 ✅
- float 例外：320x180 玻璃白（悬浮窗独立形态，合理）

## 4. 可访问性（lint accessibility）

| 问题 | 严重度 | 建议 |
|---|---|---|
| 弱文本 #9ca3af 对比度 ~2.6:1（低于 WCAG AA 4.5） | P2 | 仅用于次要信息（label/time/sub），关键信息已用 #6b7280 |
| 按钮 32px 高（padding 6x20） | P2 | 桌面端可接受，触控目标接近 44px |
| float 关闭/最小化 24x24 | P2 | 悬浮窗紧凑可接受 |
| 最小字号 10px | ✅ | v6 已从 9px 修正 |

无 P0/P1 可访问性问题。

## 5. 设计 tokens（analyze 维度）

| 维度 | 结果 |
|---|---|
| colors | 9 色规范，无游离色 ✅ |
| typography | 5 级清晰（20/14/13/11/10）✅ |
| spacing | 16/12/8/6 规范 ✅ |
| clusters | card×5 / badge×3 / table×2 组件化好 ✅ |

## 6. 跨页面一致性矩阵

| 检查项 | dashboard | history | models | settings | float |
|---|---|---|---|---|---|
| panel 600px | ✅ | ✅ | ✅ | ✅ | N/A(320) |
| header H1+sub | ✅ | ✅ | ✅ | ✅ | N/A |
| card 白+shadow | ✅ | ✅ | ✅ | ✅ | 玻璃白 |
| badge 渐变 | ✅ | - | - | ✅ | ✅ |
| 配色 9 色 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 字体分级 | ✅ | ✅ | ✅ | ✅ | ✅ |

## 7. 发现的问题

| # | 问题 | 严重度 | 位置 | 修复建议 |
|---|---|---|---|---|
| 1 | models 模型卡片重复定义 shadow（.mc 自带，未用 .card 类） | P3 | models.html:23 | 可改用 .card 类，但不影响视觉 |
| 2 | history 费用列有 $ 符号，dashboard 无费用数据 | P3 | history.html:79 | 数据格式差异，合理 |
| 3 | float 圆环 80px vs dashboard 圆环 56px | - | - | 合理（悬浮窗单圆环更大）|
| 4 | settings card-inner max-width 480 居中，其他页 card 满宽 | - | settings.html:14 | 合理（设置页窄卡片）|

无 P0/P1/P2 结构问题。

## Dev Readiness: Prototype Ready

- 功能覆盖 100%（5/5 Yes）
- 视觉一致性高（配色/字体/圆角/间距/组件统一）
- 可访问性少数 P2（弱文本对比度/按钮高度），桌面端可接受
- 可用 omp 按设计稿重构前端

## 建议下一步
1. 用 omp 按 v6 设计稿重构 Dashboard/History/Models/Settings/FloatWidget 前端
2. 重构时关键文本用 #6b7280（不用 #9ca3af）提对比度
3. 按钮高度提至 36px
