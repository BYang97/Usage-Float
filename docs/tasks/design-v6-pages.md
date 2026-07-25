# 设计稿 v6 - 其他页面（History/Models/Settings/Float）

路由: omp (qoder 无额度) | 模式: B 单任务 handoff

## 目标
为 4 个页面设计 HTML 静态设计稿（不写 React，只 HTML+CSS），风格与首页 `monitor-v6-dashboard.html` 统一。

## 非目标
- 不改 React/TS 源码
- 不改 Rust 源码
- 不改首页 dashboard 设计稿

## 依赖
- `docs/design/system-v2-light.md`（设计系统：配色/字体/圆角/阴影/组件）
- `docs/design/monitor-v6-dashboard.html`（首页风格基准，照抄其 CSS 变量与卡片样式）

## 输出文件（新建 4 个）
1. `docs/design/monitor-v6-history.html`
2. `docs/design/monitor-v6-models.html`
3. `docs/design/monitor-v6-settings.html`
4. `docs/design/monitor-v6-float.html`

## 共享风格约束（所有页面必须遵守）
- 背景 `#f8f9fa`，卡片白 `#fff` + 圆角 12 + shadow `0 1px 3px rgba(0,0,0,0.04),0 1px 2px rgba(0,0,0,0.02)` + hover `0 4px 12px rgba(0,0,0,0.06)` + translateY(-1px)
- 字体 Inter；标题 H1 20px/700 `#1a1d21`，section-title 13px/600 `#6b7280`，body 13px，secondary 11px `#9ca3af`，weak 10px
- accent 渐变 `linear-gradient(135deg,#6366f1,#8b5cf6)`；status 绿 `#10b981`/黄 `#f59e0b`/红 `#ef4444`
- badge: 10px/600 + 圆角 9 + padding 2x8；lite 渐变紫，go 渐变绿青 `linear-gradient(135deg,#10b981,#06b6d4)`
- 面板宽 600px 居中（与首页一致），padding 24
- header: `<h1>` + `<span class="sub">` 副标题
- 圆环用 SVG（viewBox 0 0 56 56，r=24，stroke-width 4，dasharray=150.8，按百分比算 dashoffset，颜色按值：<50% 绿/50-80% 黄/>80% 红）

## 各页面设计要求

### 1. History（使用记录）`monitor-v6-history.html`
- header: "使用记录" + sub "3 个账号 · 用量明细"
- **Token 消耗汇总**（section-title + card）：3 列网格（今日/近7天/近30天），数值 20px/700，近7天用 accent 紫色高亮，label 10px `#9ca3af`
- **用量历史**（section-title + card）：
  - 卡片头部：左 "用量历史" 13px/600，右 账号选择器（select 样式：白底 + border `#e9ecef` + 圆角 4 + padding 4x8，3 个选项：默认/测试账号/团队号）
  - 表格：列 时间/模型/Provider/Input/Output/Reasoning/Cache/费用；表头 11px `#9ca3af` + 底边框；行 13px `#1a1d21` + 底边框 `#f1f3f5` + hover `#f8f9fa`；数字列右对齐 + tabular-nums；时间列 `#9ca3af` + nowrap
  - 造 5 行示例数据（模型: deepseek-v4-flash/gpt-5.5/未知；费用 4 位小数）
  - 底部居中 "加载更多" 按钮（白底 + border + 圆角 6 + padding 6x20）

### 2. Models（模型统计）`monitor-v6-models.html`
- header: "模型统计" + sub "近30天 · 模型分布"
- **模型使用**（section-title + card）：
  - 每行：模型名（140px 宽，11px）+ 进度条（flex:1，高 6px，bg `#e9ecef`，fill 圆角 3）+ 百分比（40px 右对齐，11px/600 `#6b7280`）
  - bar 颜色按使用量：第1名 accent 紫 `#6366f1`，第2名 黄 `#f59e0b`，其余 绿 `#10b981`
  - 3 行示例：deepseek-v4-flash 32.8%、未知 66.7%、gpt-5.5 0.4%
- **模型卡片网格**（section-title + 3 列 grid）：每张卡显示 模型名(14px/600) + 调用次数(20px/700) + token 数(11px `#9ca3af`) + 费用(11px accent)；3 张示例卡

### 3. Settings（设置）`monitor-v6-settings.html`
- header: "设置" + sub "通用 · 外观 · 隐私 · 账号"
- 单卡片宽 480 居中，内含分区（Section: 标题 13px/600 `#6b7280` + 分区间分隔线 1px `#e9ecef`）
- **通用设置**：
  - 开机自动启动：label + toggle（36x20 圆角 10 灰 `#e9ecef` opacity 0.4）+ "即将支持" chip（10px `#9ca3af` bg `#f1f3f5` 圆角 3 padding 2x6）
  - 刷新频率：label + 3 个 chip（5分钟/30分钟/60分钟，24 高 圆角 4 bg `#f1f3f5` opacity 0.4）+ "即将支持"
- **外观设置**：
  - 悬浮球：toggle 置灰 + "即将支持"
  - 主题：选择器（28 高 圆角 4 bg `#f1f3f5` "深色模式" + ">"）置灰 + "即将支持"
- **隐私设置**：图标(14x16 `#9ca3af`) + "数据仅保存在本机 · 不上传用户数据" 13px `#6b7280`
- **账号管理**（section-title + AccountTable）：
  - 表格：列 账号名/Workspace/Plan/操作；3 行（默认/Lite、测试账号/Lite、团队号/Go，badge 同首页）
  - 操作列：编辑/删除文字按钮（11px accent / 红）
  - 底部 "添加账号" 按钮（渐变 accent + 白字 + 圆角 8 + padding 8x16）

### 4. FloatWidget（悬浮窗）`monitor-v6-float.html`
- body 居中显示一个悬浮窗（不套面板，直接居中）
- 悬浮窗：320x180 + 圆角 20 + 玻璃白 `rgba(255,255,255,0.8)` + backdrop-filter blur(24px) + shadow `0 12px 40px rgba(0,0,0,0.12),0 4px 12px rgba(0,0,0,0.06)` + overflow hidden + relative + flex column center
- 左上 ProviderBadge（小圆角 badge "Claude" 渐变紫）
- 右上 关闭 ✕（24x24 透明 `#9ca3af` 12px）+ 右上偏左 最小化 ─（24x24）
- 中间 QuotaRing（圆环 80x80，viewBox 0 0 80 80，r=34，stroke-width 6，百分比 12% 绿色，中间文字 "12%" 16px/700 绿色）
- 圆环下：5小时额度 13px/500 `#6b7280` + 分隔点 + 重置 13px `#9ca3af` "4h 3m"
- 底部 "打开仪表盘 >" 按钮（12px/500 accent 透明背景）

## 验证
- 每个文件用浏览器打开无报错
- 4 个页面风格与 dashboard 一致（配色/字体/圆角/卡片）
- 数据示例合理（与 dashboard 账号/模型对应）

## 完成标准
- 4 个 HTML 文件已创建
- 每个文件可独立打开预览
- 风格统一，功能完整
