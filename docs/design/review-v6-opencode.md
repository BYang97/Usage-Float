# v6 设计稿评审报告（OpenCode）

## 总体评价
v6 设计稿在功能完整性和视觉一致性上表现优秀，采用浅色+玻璃拟态+渐变的现代设计语言，5 个页面覆盖核心监控场景。但存在关键信息层级不够突出、缺失边界状态设计、对比度未达标等问题。**评分：7.8/10**

## 优点
- **视觉系统严谨**：9 色配色、5 级字体、统一圆角（12/9/6/4）、柔和多层阴影，跨 5 页面无游离值（dashboard.html:13-38, history.html:13-33, models.html:13-28, settings.html:13-44, float.html:9-23）
- **组件化良好**：card/badge/table/toggle 可复用，settings.html 账号表格（106-135）与 history 用量表格（57-122）使用相同 table 样式
- **信息架构完整**：dashboard 三圆环（5h/周/月）+ token 汇总 + 模型分布（dashboard.html:48-159），覆盖重度用户核心监控需求
- **玻璃拟态细腻**：float 悬浮窗 rgba(255,255,255,0.8) + blur(24px)（float.html:9），shadow-float 双层阴影营造悬浮感
- **渐变 accent 优雅**：badge.lite/go（settings.html:36-37）、btn-accent（settings.html:38）统一使用 135deg 渐变，避免生硬纯色

## 问题与建议

| # | 维度 | 问题 | 严重度 | 建议 |
|---|---|---|---|---|
| 1 | 功能完整性 | 缺失空状态：无账号、无历史记录、模型数据为空时无 empty state 设计 | P1 | 增加 empty 插图 + "添加首个账号" CTA；history/models 空状态显示占位符+引导文案 |
| 2 | 功能完整性 | 缺失 loading 状态：dashboard 3 账号卡片、history 表格加载时无骨架屏 | P1 | 增加 skeleton loading 设计（圆环+数值骨架），避免白屏闪烁 |
| 3 | 功能完整性 | 缺失错误状态：API 失败、账号失效、超额警告时无视觉反馈 | P1 | 超额（>90%）圆环增加脉动动画；失效账号卡片增加红色边框+重新授权入口 |
| 4 | 信息架构 | 配额预警不够突出：dashboard 本月 88%/91% 用红色圆环，但卡片无整体警示（dashboard.html:68-69/102-103） | P0 | 超 85% 卡片增加黄色左边框；超 95% 增加红色背景渐变 + 顶部 banner"即将用尽" |
| 5 | 视觉一致性 | 弱文本 #9ca3af 对比度 2.6:1，低于 WCAG AA 4.5:1（settings.html:18, float.html:19） | P1 | 关键信息（reset time、账号 workspace）改用 #6b7280；纯装饰性文本保留 #9ca3af |
| 6 | 交互体验 | float 悬浮窗与主窗口衔接断裂：点击"打开仪表盘"后无上下文延续（float.html:50） | P2 | 打开 dashboard 时高亮对应账号卡片 + 滚动到该卡片；或悬浮窗增加账号切换下拉 |
| 7 | 交互体验 | settings 表格编辑/删除按钮无确认流程，误操作风险高（settings.html:120-132） | P2 | 删除改为 popover 二次确认；编辑打开 modal 而非 inline 编辑 |
| 8 | 可访问性 | 按钮触控目标过小：float 关闭/最小化 24×24（float.html:11-14），settings 文本按钮 padding 2×6（settings.html:40-42） | P2 | float 按钮改为 32×32；settings 编辑/删除按钮改为 icon button 28×28，增加 hover/active 状态 |
| 9 | 设计风格 | 数值字号跨度过大：dashboard 圆环内 13px（dashboard.html:53）vs history token 汇总 20px（history.html:42-44） | P2 | 统一中型数值为 16-18px；hero 数值（dashboard 账号 token）用 24px 而非内联 13px |
| 10 | 视觉一致性 | models 页模型卡片未复用 .card 类，自定义 box-shadow（models.html:23-24） | P3 | 改用 .card 类 + .model-card 扩展类，避免阴影参数漂移 |

## 竞品对比

### vs Linear（项目管理）
- **优势**：v6 渐变 accent 与 Linear 紫色主题一致，badge/button 渐变更现代；玻璃拟态悬浮窗比 Linear 传统卡片更有科技感
- **差距**：Linear 配额警示用整页 banner + 倒计时，v6 仅靠圆环红色不够醒目；Linear 空状态有插图+文案，v6 未设计

### vs Raycast（启动器）
- **优势**：v6 圆角 12/20 与 Raycast 大圆角风格接近；悬浮窗 blur(24px) 达到 Raycast 毛玻璃质感
- **差距**：Raycast 主界面优先级清晰（搜索框+快捷操作巨大），v6 dashboard 三账号卡片等权重，未突出最紧急账号；Raycast 暗色主题占主流，v6 仅浅色

### vs Vercel Dashboard（部署面板）
- **优势**：v6 token 汇总三栏网格（history.html:41-45）与 Vercel 指标卡片布局相似，简洁高效
- **差距**：Vercel 用大面积留白 + 单一 accent 色引导视线，v6 渐变/红/黄/绿多色分散注意力；Vercel loading skeleton 完善，v6 缺失

### 结论差距
1. **信息优先级不够狠**：竞品用 banner/巨大 CTA 突出关键动作，v6 依赖用户扫描三圆环找红色
2. **边界状态缺失**：竞品 empty/loading/error 设计完整，v6 仅有 happy path
3. **颜色克制不足**：竞品单一 accent 色 + 大量留白，v6 渐变/多色虽美观但略显拥挤

## 结论
**需小迭代** → 补齐 P0/P1 问题（配额预警强化、empty/loading/error 状态、对比度修正）后可投入开发。建议优先实现 dashboard + float，用真实数据验证信息层级后再开发其余页面。下一步：用 omp 按设计稿重构 dashboard.html 时同步增加警示 banner + skeleton loading。
