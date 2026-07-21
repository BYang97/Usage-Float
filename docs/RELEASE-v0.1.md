# Release v0.1

OpenCode Usage Float v0.1 — 读取本地 OpenCode SQLite,桌面悬浮窗与仪表盘展示真实 token 用量与模型分布。

## 构建

```bash
cd src-tauri && cargo build --release --offline   # 产 exe(离线,依赖已缓存)
# 或完整 bundle(需联网下载 WiX 产 msi):
npx @tauri-apps/cli build
```

产物:`src-tauri/target/release/app.exe`(约 10.8 MB)

## 校验

- **SHA256**:`8AD05793464E9F2E1D03409E7059B1A49BF68F652C2A10F776DD17224BA46AAE`(本机构建,同源同配置可复现)

## 能力

- 读取本地 OpenCode SQLite(`~/.local/share/opencode/opencode.db`),只读,绝不写回
- 仪表盘:累计 token、近 7 天历史折线、按模型分组占比(真实数据)
- OpenCode Go 认证面板(auth cookie 粘贴,本地 ring AEAD 加密存储)
- 完全离线,数据不外发

## 测试

`cargo test --offline`:**8 passed;0 failed**,覆盖契约第8节本地 SQLite 采集全部场景(resolve/empty/damaged schema/multi/时间桶/model JSON/NULL model/无环境)。

## 已知限制

- **配额百分比(5h / 周 / 月)与重置倒计时**:mock 占位。OpenCode 本地无配额周期数据,需接 opencode.ai API(待 v0.2;调研受 5h 调用上限中断,端点待补)。
- **msi bundle**:tauri build 的 msi 阶段需联网下载 WiX 工具,网络受限时失败;但 exe 本身可离线产出。
- 仅 Windows 验证(macOS/Linux 路径逻辑已写,未实测)。

## 路线图

- **v0.2**:opencode.ai API 取真实配额 + 重置倒计时
- **Phase 3+**:Windows 系统托盘、开机启动、悬浮球(AlwaysOnTop)、Windows 通知
