# 设计稿 v8 迭代（限定窗口尺寸 + 规格说明）

路由: omp | 模式: B 单任务 handoff

## 目标
基于 v7 + `docs/design/spec-v8.md` 迭代 v8。**核心改动：所有主窗口页面设计稿按实际窗口 1000×700 完整设计（含 Sidebar + Header + 内容区 760×644），不再用 600px 独立 panel**。保留 v7 的功能修复（预警/三态/排序等）。不写 React/Rust 代码。

## 依赖
- `docs/design/spec-v8.md`（设计规格：窗口/布局/栅格/间距/组件/页面 -- 必须遵循）
- `docs/design/monitor-v7-*.html`（v7 5 页面 + states，功能基础）
- `docs/design/system-v2-light.md`（设计系统）

## 输出文件（新建 v8）
1. `docs/design/monitor-v8-dashboard.html`
2. `docs/design/monitor-v8-history.html`
3. `docs/design/monitor-v8-models.html`
4. `docs/design/monitor-v8-settings.html`
5. `docs/design/monitor-v8-float.html`
6. `docs/design/monitor-v8-states.html`

## 核心改动：完整窗口布局

### 主窗口页面（dashboard/history/models/settings）
每个设计稿画板 **1000×700**，必须包含三部分：

```
<div class="window"> <!-- 1000×700, 圆角 16, overflow hidden, shadow -->
  <div class="sidebar"> <!-- 240×700, 玻璃白 -->
    <div class="logo"> <!-- 56 高, logo + "OpenCode Usage Float" -->
    <div class="nav"> <!-- 4 项: 首页/使用记录/模型统计/设置, 选中渐变 accent -->
    <div class="toggle"> <!-- 折叠按钮 ◀ -->
  </div>
  <div class="main"> <!-- 760×700 -->
    <div class="header"> <!-- 760×56, 标题 + 右侧 ⚙─✕ -->
    <div class="content"> <!-- 760×644, padding 16, gap 12, overflow-y auto -->
      <!-- 页面内容 -->
    </div>
  </div>
</div>
```

**Sidebar**（照抄 web/src/components/Sidebar.tsx 风格）：
- 240 宽，玻璃白 `rgba(255,255,255,0.6) + blur(20px)`
- logo 区 56 高：20×20 圆角 5 紫色方块 + "OpenCode Usage Float" 14px/600
- nav 4 项：每项 padding 8×12, 圆角 8, 选中渐变 accent + 白字, 未选透明 + #6b7280
- 底部折叠按钮 ◀

**Header**（照抄 web/src/components/Header.tsx 风格）：
- 56 高，左标题 H1 20px/700, 右侧 ⚙(设置) ─(最小化) ✕(关闭) 三个按钮
- 底部 1px 边框 #e9ecef

**内容区**：
- 760×644（实际 760 宽，高度 700-56=644），padding 16, gap 12
- 背景可加微妙渐变或纯 #f8f9fa

### 各页面内容区布局

#### Dashboard（760×644 内容区）
- 账号卡片纵向排列（满宽 728），保留 v7 预警（黄/红边框 + banner + 脉动）+ 失效账号
- token 汇总（3 列网格）+ 模型 Top2 + 查看全部
- 内容可超出 644 滚动（设计稿展示完整内容，画板高度可 > 644 但标注"内容区 760×644，超出滚动"）

#### History（760×644）
- token 汇总（3 列）+ 筛选栏（日期 chips + 账号选择器）+ 表格（满宽 8 列）+ 分页
- 保留 v7 排序 ↑↓ + 空状态 + 骨架

#### Models（760×644）
- 模型使用进度条（满宽）+ 模型卡片网格（3 列）
- 保留 v7 "未识别"虚线条纹

#### Settings（760×644）
- 单卡片 480 居中，分区（通用/外观/隐私/账号管理）
- 保留 v7 可交互 mockup + 删除 popover + 编辑 modal

### Float（320×180）
- 保持 v7 设计（拖拽手柄 + 32×32 按钮 + 账号切换）
- 画板 320×180，独立窗口（不加 Sidebar/Header）

### States（4 状态画板）
- 每个状态画板用 760×644 内容区尺寸（或 1000×700 完整窗口，含 Sidebar/Header）
- 4 状态：空账号 / 加载骨架 / 错误 / 超额警告

## 规格标注（必须）
每个设计稿画板**右下角**加规格标注卡片：
```
画板: 1000×700
Sidebar: 240 | Header: 56
内容区: 760×644 (padding 16, gap 12)
```
（float 标注 "画板: 320×180"）

## 验证
- 每个文件浏览器打开无报错
- 主窗口页面画板严格 1000×700（含 Sidebar + Header + 内容区）
- 遵循 spec-v8.md 间距/字号/组件规格
- 保留 v7 功能修复
- 有规格标注

## 完成标准
- 6 个 v8 HTML 文件
- 主窗口页面统一 1000×700 完整窗口布局
- 规格标注清晰
- 无暗色模式
