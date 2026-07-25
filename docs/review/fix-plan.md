# P0 修复计划(评审汇总)

## 前端修复(review-frontend)
1. Models 百分比格式化(ModelUsage.tsx toFixed(1))+ 空名 fallback "未知"
2. Dashboard 条件渲染 expire(PlanCard: expireDate !== '-' 才显示)+ 删 status 重复(硬编码"正常"副标题)
3. History/Models Header onMinimize/onClose 接线(同 Dashboard,不传 undefined)
4. History/Models error 加重试按钮
5. Settings 未接线项置灰 + "即将支持"标注(开机启动/刷新频率/悬浮球/主题)

## Rust 修复(review-rust)
1. build_account_info: api_account.expire_date None 时传 "-" 不 fallback mock
2. aggregate_local: 分 tokens_7d/tokens_30d 窗口(当前 7d==30d)
3. aggregate_local: 空名模型标 "未知"
4. refresh_one 写缓存(set_quota_cache + set_account_cache,同 fetch_api_quota)
