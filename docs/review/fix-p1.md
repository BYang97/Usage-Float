# P1 修复计划

## 任务 A: History 账号选择器 + 分页(fix-p1-history)
1. web/src/pages/History.tsx 加账号选择器(下拉选账号,默认第一个),调 invoke get_usage_history 用选定 accountId
2. History 分页:cursor 翻页(加载更多按钮,递增 cursor)
3. 错误协调:token 数据 + history 明细统一加载态(任一失败显示错误)

## 任务 B: 交互修复(fix-p1-interaction)
1. web/src/components/PlanCard.tsx: expire 为空/"-"时不显示到期时间行(完全隐藏,不是显示"-")
2. web/src/components/AccountTable.tsx: 单账号刷新失败显示错误提示(不静默)
3. auto-refresh timer race: App.tsx + FloatWindow.tsx 都调 startAutoRefresh(重复刷新)。修复:只在 App.tsx 启动,FloatWindow 不启动(共享 usage-service cached)
4. web/src/pages/History.tsx + Models.tsx: error 态重试按钮 onClick 单一(不 refreshAndNotify+loadData 双调)

参考 @docs/review/report-interaction.md + report-ui.md + report-functionality.md。
