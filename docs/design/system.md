# Usage Float 设计系统(OpenPencil)

## 设计原则
深色主题 + 简洁卡片 + 清晰层级 + 统一圆角/间距。所有页面 Sidebar(240) + Header(56) + 内容区(padding 16, gap 12)布局。

## 配色(已实现 index.css @theme)
| token | hex | 用途 |
|---|---|---|
| surface | #1a1c1e | 主背景 |
| surfaceAlt | #2e3033 | Sidebar/Card 背景 |
| card | #282b2d | 卡片 |
| surfaceHover | #34363a | hover |
| border | #3f4247 | 边框 |
| textPrimary | #f2f4f9 | 主文本 |
| textSecondary | #9ea3ad | 次文本 |
| textTertiary | #72757f | 弱文本 |
| accentBlue | #599eff | 强调(选中/链接/7天) |
| statusOk | #4cd18c | 正常(低额度<50%) |
| statusWarning | #f2c14f | 警告(中额度50-80%) |
| statusDanger | #ea6060 | 危险(高额度>80%) |

## Typography(Inter)
| 级别 | size | weight | 颜色 | 用途 |
|---|---|---|---|---|
| H1 | 24 | Semi Bold | textPrimary | 页面标题 |
| H2 | 16 | Semi Bold | textPrimary | 区块标题(Header) |
| H3 | 14 | Semi Bold | textSecondary | 卡片标题 |
| 正文 | 13 | Regular | textPrimary | 主要文本 |
| 次要 | 12 | Regular | textSecondary | 标签/说明 |
| 弱 | 11 | Regular | textTertiary | 辅助/重置时间 |
| 数值大 | 28-32 | Semi Bold | 按额度配色 | 配额百分比 |
| 数值中 | 20 | Semi Bold | textPrimary/accent | token 值 |

## Spacing / Radius
| token | 值 | 用途 |
|---|---|---|
| radius-sm | 4 | 小元素(tag) |
| radius-md | 6 | Input/Button |
| radius-lg | 8 | nav 项/主 frame |
| radius-xl | 12 | Card |
| gap-sm | 8 | 卡片内元素 |
| gap-md | 12 | 卡片间/网格 |
| gap-lg | 16 | 区块间 |
| padding-card | 16 | 卡片内边距 |
| padding-page | 16 | 内容区内边距 |

## 布局规范(所有页面)
- 主窗口 1000x700,圆角 8
- Sidebar 240 宽,surfaceAlt 背景,nav 项 208x36 圆角 8,选中 accent 文字 + surface 背景
- Header 760x56,surface 背景,H2 标题 + 弱副标题
- 内容区 padding 16,gap 12,卡片网格 gap 12

## 组件规范
- **Card**: card 背景,圆角 12,padding 16,gap 12,可选 border 1px
- **QuotaCard**: Card + 标题(H3) + 百分比(数值大,按额度配色) + ProgressBar + 重置时间(弱)
- **TokenCard**: Card + H3 标题 + 三列(标签弱 + 值数值中,7天 accent)
- **Button**: accent 背景,圆角 8,padding 8x16,白字 Semi Bold
- **Input**: surface 背景,圆角 6,border 1px,padding 8x12
- **AccountCard**: Card + 账号名(H3) + workspaceId(弱等宽) + plan badge + 配额三窗口 + 操作按钮

## 悬浮窗
- 320x180,圆角 12,玻璃背景(glass rgba(26,27,30,0.72) + blur 20)
- 顶部拖拽区 36px
- QuotaRing(配额环) + 百分比 + 重置时间 + 打开仪表盘链接

## 实现任务(omp)
重构所有页面按设计系统:
1. Dashboard: Sidebar + Header + 3 QuotaCard(网格) + TokenCard + 图表
2. History: Sidebar + Header + TokenUsage + usage 明细表格(Card)
3. Models: Sidebar + Header + 模型占比列表(Card + ProgressBar)
4. Settings: Sidebar + Header + 设置项(Card) + AccountTable
5. FloatWindow: 玻璃背景 + QuotaRing + 拖拽区
统一:圆角 12(Card)/8(nav/Button)/6(Input),间距 gap 12/padding 16,配色按额度,Typography 分级。
