# P1/P2 修复计划

## 任务 A: History 分页(fix-p2-pagination)
web/src/pages/History.tsx 加 cursor 分页:
1. state 加 cursor + hasMore
2. loadHistory(accountId?, append?) append 时 cursor 递增 + items append
3. 加载更多按钮(onClick loadMore: cursor+50, loadHistory(selectedAccountId, true))
4. 切账号时重置 cursor + items
参考 @docs/review/report-functionality.md P1-1 分页。

## 任务 B: P2 美化(fix-p2-polish)
1. web/src/pages/History.tsx cost 精度 toFixed(4)(或动态:>=1 2位,<1 4位)
2. 重置时间格式统一(reset_in_sec -> "Xh Ym" / "Xm",Dashboard/AccountTable/FloatWidget 一致)
3. 加载/空态 spinner 统一(Dashboard/History/Models/AccountTable 用同款 spinner)
参考 @docs/review/report-ui.md P2。
