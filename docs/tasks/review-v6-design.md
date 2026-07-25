# v6 设计稿评审（双 agent 独立评审）

路由: omp + opencode（两个不同 CLI 各自独立评审）| 模式: B 平行 handoff

## 目标
评审 Usage Float v6 设计稿，输出结果与建议。**不写代码，不改任何文件**，只输出评审报告。

## 评审对象（只读）
- `docs/design/monitor-v6-all.html`（汇总画布，含 5 页面 iframe）
- `docs/design/monitor-v6-dashboard.html`（首页：3 账号 + 圆环 + token + 模型）
- `docs/design/monitor-v6-history.html`（使用记录：表格 + 选择器 + 分页）
- `docs/design/monitor-v6-models.html`（模型统计：进度条 + 卡片）
- `docs/design/monitor-v6-settings.html`（设置：通用/外观/隐私/账号）
- `docs/design/monitor-v6-float.html`（悬浮窗：320×180 玻璃白）
- `docs/design/system-v2-light.md`（设计系统：配色/字体/圆角/阴影）
- `docs/design/review-v6.md`（已有的自审报告，参考但独立判断）

## 产品背景
Usage Float 是 Tauri 桌面应用，监控多账号的 AI API 配额（5小时/周/月）+ token 消耗 + 模型使用，含桌面悬浮窗常驻显示。目标用户是重度 AI API 使用者。

## 评审维度
1. **功能完整性**：各页面功能是否覆盖产品需求；有无缺失场景（loading/error/empty/空账号/超额警告）
2. **视觉一致性**：配色/字体/圆角/间距/组件跨 5 页面是否统一；有无游离值
3. **信息架构**：布局/层级/优先级是否合理；用户第一眼能否看到最关键信息（配额预警）
4. **交互体验**：操作流是否顺畅；反馈是否完善；悬浮窗与主窗口衔接
5. **可访问性**：对比度/字号/触控目标；是否符合 WCAG AA
6. **设计风格**：浅色+玻璃拟态+渐变是否协调；是否像成品而非 demo；与 Linear/Vercel/Stripe/Raycast 等竞品对比的优劣

## 输出要求
把报告写到指定文件（见下方"输出路径"）。格式：

```markdown
# v6 设计稿评审报告（<agent 名>）

## 总体评价
（2-3 句，含评分如 7.5/10）

## 优点
- （具体，引用页面/元素）

## 问题与建议
| # | 维度 | 问题 | 严重度 | 建议 |
|---|---|---|---|---|
| 1 | 视觉一致性 | ... | P0/P1/P2 | ... |

## 竞品对比
（与至少 2 个竞品对比，指出差距与优势）

## 结论
Dev Ready / 需小迭代 / 需大改
（一句话下一步）
```

## 输出路径
- omp agent 写到: `C:\Users\admin\AppData\Local\Temp\review-omp.md`
- opencode agent 写到: `C:\Users\admin\AppData\Local\Temp\review-opencode.md`

## 完成标准
- 报告文件已写入指定路径
- 覆盖全部 6 个维度
- 至少 8 条具体问题/建议（带严重度）
- 含竞品对比
- 含明确结论
