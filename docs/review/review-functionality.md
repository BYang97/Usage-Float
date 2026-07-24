# 评审任务: functionality 视角

## 背景
评审 Usage-Float v0.3(多账号+History/Models+日志+悬浮窗)。走查报告见 @docs/review/walkthrough.md。

## 评审范围
功能完整性:看多账号(database/api/lib)+ 配额(fetch_quota/缓存)+ History(usage_history)+ 悬浮窗(tray)+ 走查。提功能建议:
- mock 数据(到期时间/Active/plan -- 真实 lite 无 expire,该显示什么)
- 未接线功能该实现 or 移除
- 配额/History 数据链完整性
- 多账号 CRUD + 刷新链路
产出 docs/review/report-functionality.md。
