# UI 布局排版走查(2026-07-25)

## Dashboard
- Sidebar(logo + nav + ◀ 折叠) + Header(仪表盘 + ⚙/─/✕) + 账户区(Lite/正常)
- 本月额度 91%(大卡) + 5小时 0% / 本周 8%(小卡) + Token 消耗
- 问题:本月额度在前(应 5小时/本周/本月 顺序 或 按重要性);账户区(Lite/正常)位置;Header 标题"仪表盘"(该账号名?)

## History
- Header(缺标题,只有 ─/✕) + Token 消耗 + 折线图 + usage 明细表格
- 问题:Header 缺"使用记录"标题;Token 消耗 + 折线 + 表格层级混乱

## Settings
- Header(设置 + ⚙/─/✕) + 内容(设置 + ✕ + 通用/外观/隐私/账号管理)
- 问题:两个"设置"标题(Header + 内容) + 两个 ✕(Header + 内容)重复;内容区"设置"标题 + ✕ 多余

## 布局问题汇总
1. Header 标题不一致(Dashboard"仪表盘"/History 缺/Settings 重复)
2. Settings 内容区重复 Header(设置标题 + ✕)
3. Dashboard 配额顺序(本月在前,应按时间或重要性)
4. 间距/层级/视觉平衡(卡片大小/对齐/留白)
5. 信息架构(账户区/配额/Token/图表 分组)
6. 响应式(窗口缩放布局)
