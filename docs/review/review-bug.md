# 评审任务: bug 视角

## 背景
评审 Usage-Float v0.3(多账号+History/Models+日志+悬浮窗)。走查报告见 @docs/review/walkthrough.md。

## 评审范围
Bug/边界:看代码 + 走查。提 Bug:
- 空名模型("" 66.77%)
- Dashboard status 重复("正常"+"Active")
- 空数据/错误/并发(无账号/cookie 过期/网络失败)
- 缓存/刷新边界
产出 docs/review/report-bug.md。
