# OpenCode Usage Float

监控本地 [OpenCode](https://opencode.ai) 用量的桌面悬浮球应用。读取本地 OpenCode SQLite 数据,在桌面悬浮窗与仪表盘展示 token 用量、按天历史、模型分布。

> 完全离线:只读本地 `opencode.db`,数据不外发。auth cookie(用于配额 API,待 v0.2)仅本地加密存储。

## 当前能力(v0.1)

- ✅ 读取本地 OpenCode SQLite(`opencode.db`),聚合 session 用量
- ✅ 仪表盘显示:累计 token、近 7 天历史折线、按模型分组占比
- ✅ OpenCode Go 认证设置面板(auth cookie 粘贴,本地加密存储)
- ⏳ 配额百分比(5h / 周 / 月)与重置倒计时:暂为占位,待 v0.2 接 opencode.ai API

## 技术栈

- Desktop:Tauri 2
- 后端:Rust(rusqlite 只读 OpenCode db + ring AEAD 加密 cookie)
- 前端:React 19 + TypeScript + Tailwind CSS v4 + Recharts

## 开发

```bash
# 前端(在 web/ 目录)
bun install
bun run dev

# Tauri 桌面应用(在项目根)
npx @tauri-apps/cli dev

# 离线编译(cargo 用本地缓存,因 crates.io 网络限制)
cd src-tauri && cargo check --offline
```

## 数据源

读取 OpenCode 本地 SQLite(`~/.local/share/opencode/opencode.db`,Windows 为 `%USERPROFILE%\.local\share\opencode\opencode.db`)。只读打开(`SQLITE_OPEN_READ_ONLY`),绝不写回 OpenCode 的数据库。

session 表预聚合列:`cost` / `tokens_input` / `tokens_output` / `tokens_reasoning` / `tokens_cache_read` / `tokens_cache_write` / `time_created` / `model`(JSON)。

## 路线图

- **v0.1**(当前):本地 SQLite 采集 + 仪表盘 + 认证面板
- **v0.2**:接 opencode.ai API 取真实配额(5h / 周 / 月 + 重置倒计时)
- **Phase 3+**:Windows 系统托盘、开机启动、悬浮球(AlwaysOnTop)、Windows 通知

## 多 agent 协作

本项目用 Orca 多 agent 协作开发:
- **opencode**:Rust collector 实现(读 OpenCode SQLite + 聚合)
- **pi**:Tauri 接线 + 前端 cookie 面板 + settings 命令
- **claude**(本 agent):契约设计 + 合并验收 + bug 定位

接口契约见 [`docs/contract.md`](./docs/contract.md),OpenCode 数据格式调研见 [`docs/opencode-data-format.md`](./docs/opencode-data-format.md)。
