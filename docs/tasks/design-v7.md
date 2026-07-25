# 设计稿 v7 迭代（修复双 agent 评审问题）

路由: omp | 模式: B 单任务 handoff

## 目标
基于 v6 设计稿迭代 v7，修复 omp + opencode 双 agent 评审的问题。**不写 React/Rust 代码，只改 HTML 设计稿**。暗色模式不做。

## 依赖
- `docs/design/monitor-v6-*.html`（v6 5 页面，作为基础，照抄其 CSS/结构再改）
- `docs/design/system-v2-light.md`（设计系统）
- `docs/design/review-v6-omp.md` + `review-v6-opencode.md`（评审报告，对照修复）

## 输出文件（新建 v7，保留 v6）
1. `docs/design/monitor-v7-dashboard.html`
2. `docs/design/monitor-v7-history.html`
3. `docs/design/monitor-v7-models.html`
4. `docs/design/monitor-v7-settings.html`
5. `docs/design/monitor-v7-float.html`
6. `docs/design/monitor-v7-states.html`（新增：empty/loading/error/超额 状态稿）

## 全局改动（所有页面）
1. **card 圆角 12px -> 16px**（按设计系统 radius-lg）
2. **弱文本 #9ca3af -> #6b7280**（label/time/sub/table header 等关键信息）；纯装饰极弱文本保留 #9ca3af
3. **按钮圆角 8px -> 12px**
4. `<title>` 修正为 "Usage Float v7 · <页面名>"
5. models 页 `.mc` 复用 `.card` 类

## 各页面改动

### 1. Dashboard
- **配额预警强化**（opencode P0）：
  - 月度 >85% 卡片加黄色左边框 `border-left:3px solid #f59e0b`
  - 月度 >95% 卡片加红色背景渐变 `background:linear-gradient(to right,rgba(239,68,68,0.06),#fff)` + 卡片顶部 banner "⚠ 本月额度即将用尽"
  - 圆环 >90% 加脉动动画 `@keyframes pulse{0%,100%{opacity:1}50%{opacity:.55}} animation:pulse 1.8s ease-in-out infinite`
  - 保留 3 账号示例，其中"测试账号"本月 91% 触发红色预警（边框+banner+脉动）
- **模型区去重**（omp P2）：底部模型使用改为"最近使用 Top 2"精简（只前 2 个模型 + 进度条）+ "查看全部 >" 链接（accent 色跳 Models）
- **失效账号**：可选加 1 个"已失效"账号示例（红色边框 + "重新授权"按钮）—— 若排版紧张可省

### 2. History
- **表头排序**（omp P2）：时间/费用列表头加 ↑↓ 箭头（默认 `color:#d1d5db`，hover `#6b7280`）
- **日期范围筛选**：表格上方加快捷 chips（今日/7天/30天，选中态 accent 渐变背景 + 白字）
- **空状态稿**：暂无记录时显示插图占位（灰色圆 + 文档图标 SVG）+ "暂无用量历史" + 引导文案
- **loading 骨架屏**：3 行骨架（灰色条 `background:#f1f3f5` + shimmer `@keyframes shimmer{0%{background-position:-200px 0}100%{background-position:calc(200px + 100%) 0}}`）

### 3. Models
- **"未知"改"未识别"**（omp P2）+ 进度条虚线条纹填充 `background:repeating-linear-gradient(45deg,#f59e0b,#f59e0b 4px,#fbbf24 4px,#fbbf24 8px)`
- **空状态稿**：暂无模型数据时占位 + 引导

### 4. Settings
- **可交互 mockup**（omp P1）：刷新频率 chips 改可点击（选中态 accent 渐变背景 + 白字，如 "30分钟" 选中）；主题选择器（浅色/跟随系统，浅色选中）
- **删除二次确认**（opencode P2）：删除按钮改为 popover（点击后浮层 "确认删除？" + 确认/取消按钮）
- **编辑 modal**：编辑按钮（静态展示一个 modal 样式：账号名输入 + workspace 显示 + 保存/取消）

### 5. Float
- **拖拽手柄**（omp P2）：顶部加 4px 拖拽线 `width:40px;height:4px;border-radius:2px;background:rgba(0,0,0,0.12)` 居中，cursor:grab
- **按钮放大**（opencode P2）：关闭/最小化 24x24 -> 32x32，hover 加 `background:rgba(0,0,0,0.06)` 圆角
- **账号切换**（opencode P2）：底部"打开仪表盘"上方加账号切换下拉（当前账号 + ▼ 箭头）

### 6. States（新增 monitor-v7-states.html）
4 个状态画板（垂直排列，每个带标题标注）：
1. **空账号**：居中插图（灰色账号图标 SVG）+ "还没有账号" + "添加你的第一个 API 账号" 按钮（渐变 accent）
2. **加载中**：Dashboard 骨架屏（3 个卡片骨架，圆环占位圆 + 数值灰色条 shimmer）
3. **错误**：⚠ 图标（红色圆 + 感叹号）+ "数据加载失败" + 错误信息 + "重试"按钮（accent）
4. **超额警告**：顶部红色 banner "⚠ 本月额度即将用尽（91%）" + 账号卡片红色边框 + 脉动圆环

## 验证
- 每个文件浏览器打开无报错
- 风格与 v6 一致（浅色 + 玻璃 + 渐变）
- 对照评审报告，P0/P1 问题已修复

## 完成标准
- 6 个 v7 HTML 文件已创建
- 全局改动 + 各页面改动 + 状态稿完成
- 无暗色模式
