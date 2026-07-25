# Dashboard 设计稿(OpenPencil)

## 配色(深色主题)
| token | hex | 用途 |
|---|---|---|
| surface | #1a1c1e | 主背景 |
| surfaceAlt | #2e3033 | Sidebar/card 背景 |
| card | #282b2d | 卡片 |
| border | #3f4247 | 边框 |
| textPrimary | #f2f4f9 | 主文本 |
| textSecondary | #9ea3ad | 次文本 |
| textTertiary | #72757f | 弱文本 |
| accentBlue | #599eff | 强调(选中/链接/7天 token) |
| success | #4cd18c | 正常(低额度) |
| warning | #f2c14f | 警告(中额度) |
| danger | #ea6060 | 危险(高额度) |

## 布局
- 主窗口 1000x700,圆角 8
- Sidebar 240 宽,surfaceAlt 背景,nav 项 208x36,选中 accent 文字 + surface 圆角 8
- Header 760x56,surface 背景,"OpenCode Go" 16px Semi Bold + "Lite · 正常" 11px textTertiary
- QuotaCard 156x120,card 背景,圆角 12:标题 11px textTertiary + 百分比 32px Semi Bold(按额度配色 success/warning/danger)+ 重置 10px textTertiary
- TokenCard 488x100,card 背景,圆角 12:标题 13px Semi Bold textSecondary + 三列(今日/7天/30天)标签 10px textTertiary + 值 20px Semi Bold(7天用 accent)

## 间距/圆角
- 卡片圆角 12,主 frame 圆角 8,nav 项圆角 8
- 卡片 padding 16,间距 12-16

## 实现
- index.css @theme 已改配色(见上)
- 组件:Dashboard/QuotaCard/TokenCard/Sidebar/Header 按 design 调样式(圆角 12 + 配色 + 间距)
