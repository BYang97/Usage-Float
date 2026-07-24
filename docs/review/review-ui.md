# 评审任务: ui 视角

## 背景
评审 Usage-Float v0.3(多账号+History/Models+日志+悬浮窗)。走查报告见 @docs/review/walkthrough.md。

## 评审范围
UI 美化:看 web/src/tokens.ts + 组件样式(Dashboard/History/Models/Settings/FloatWidget/AccountDialog/AccountTable)。提美化建议:
- 百分比/数字格式化(Models 66.77...% -> 66.8%,Dashboard 配额)
- 间距/对齐/配色/字体一致性
- 空状态/加载/错误视觉
- mock 数据显示(到期时间/Active 是否该隐藏或标注)
产出 docs/review/report-ui.md(问题列表 + 修复建议 + 优先级)。
