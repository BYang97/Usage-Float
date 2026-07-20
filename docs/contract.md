# Collector 接口契约

> 本文件是测试(omp)与实现(opencode)的共同依据。任何字段、签名、行为变更须先改本文件并通知双方。
> Phase 2 验收以本文件 + 测试用例为准。

## 0. 状态

- [ ] OpenCode 数据格式调研结果(填充第 1 节)
- [ ] Rust collector 函数签名(填充第 2 节)
- [ ] UsageRecord / 聚合结构(填充第 3 节)
- [ ] SQLite schema(填充第 4 节)
- [ ] Tauri command 接口(填充第 5 节)
- [ ] 错误类型(填充第 6 节)
- [ ] 测试场景清单(填充第 7 节)

## 1. OpenCode 数据格式(待调研填充)

来源:OpenCode 开源项目调研。

### 1.1 数据目录
- Windows:
- macOS:
- Linux:
- 环境变量覆盖:

### 1.2 文件格式与命名

### 1.3 token usage 字段

### 1.4 配额周期(5h / weekly / monthly 概念来源)

### 1.5 session / message 层级

## 2. Collector 函数签名(Rust)

```rust
// src-tauri/src/collector/opencode.rs
// 占位,待调研确认后填充
```

## 3. 数据结构

```rust
// 规划文档定义的统一输出结构
struct UsageRecord {
    model: String,
    input: i64,
    output: i64,
    total: i64,
}
```

## 4. SQLite Schema

规划文档要求三表:account / quota / usage。待设计。

## 5. Tauri Command 接口

```ts
// 前端 invoke 调用
invoke<UsageData>('get_usage_data')
```
Phase 1 已存在,返回 mock。Phase 2 改为读 collector + database。

## 6. 错误类型

规划要求:Rust 错误用 Result,不 panic。待定义 CollectorError。

## 7. 测试场景清单(规划文档第 6 章)

- 无 OpenCode 环境
- 数据为空
- 数据损坏
- 多 session
- Windows 权限
