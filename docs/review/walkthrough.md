# 或ca computer 端到端走查报告(2026-07-24)

## Dashboard
- 账户区:Lite / "OpenCode Go - 正常" / 到期时间 2026-08-20 / Active
  - 问题:到期时间+Active 是 mock(真实 lite 无 expire),status 重复("正常"+"Active")
- 配额三窗口:5小时 0% / 本周 3% / 本月 89% ✅
- Token 消耗:今日 0 / 近7天 522.1M / 近30天 522.1M ✅

## History
- Token 消耗:今日 0 / 近7天 522.1M / 近30天 522.1M ✅
- 用量历史:deepseek-v4-flash 50 条明细 ✅(model/tokens/cost)

## Models
- 模型使用情况
- 问题:百分比显示原始数字 66.77579356834873%(应格式化 66.8%)
- 问题:空名模型 "" 66.77%(应"未知"或过滤)

## Settings
- 通用设置:开机自动启动 / 刷新频率(5/30/60分钟) -- UI 无功能(没接线)
- 外观设置:悬浮球 / 主题(深色模式) -- UI 无功能
- 隐私设置:数据仅本地 ✅
- 账号管理:2 账号(默认+测试,Lite,配额) ✅

## 悬浮窗
- 显示/隐藏 ✅,拖动或ca computer synthetic 限制(用户手动 work)

## 问题汇总
1. Models 百分比没格式化(66.77...% -> 66.8%)
2. Models 空名模型(""/应"未知")
3. Dashboard mock 数据(到期时间/Active/"正常",真实 lite 无 expire)
4. Dashboard status 重复("正常"+"Active")
5. Settings 通用设置没接线(开机启动/刷新频率)
6. Settings 外观设置没接线(悬浮球/主题)
7. History 今日消耗 0(数据问题,可能今日无 sessions)
