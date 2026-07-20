# Collector 接口契约

> 本文件是测试(omp)与实现(opencode)、接线(pi)的共同依据。任何字段、签名、行为变更须先改本文件。
> Phase 2 验收以本文件 + omp 编写的测试用例为准。
> 数据格式事实依据见 [opencode-data-format.md](./opencode-data-format.md)。

## 0. 范围

Phase 2 collector 的目标:从本地 OpenCode SQLite 数据库读取用量,聚合成前端 `UsageData` 所需结构,经 Tauri command 返回。

**重要事实(来自调研)**:OpenCode 本地只有累计 `cost` + 5 类 token + `time_created` 时间戳,**没有 5h/weekly/monthly 配额周期概念**。这些周期是 provider 侧规则(Anthropic 计划 / OpenCode 托管 plan)。见第 7 节的已知缺口与处理策略。

## 1. OpenCode 数据源

- 文件:本地 SQLite,默认 `opencode.db`(渠道变体 `opencode-beta.db` / `opencode-latest.db`)。
- 目录解析优先级:
  1. 环境变量 `OPENCODE_DB`(绝对路径直接用;相对则相对 data 目录)
  2. `<XDG_DATA_HOME>/opencode/`(或默认 `~/.local/share/opencode`;Windows 为 `%USERPROFILE%\.local\share\opencode`)
  3. 在上述目录按渠道找 `opencode[-channel].db`
- 打开方式:只读,WAL 兼容(rusqlite `OpenFlags::SQLITE_OPEN_READ_ONLY`)。**绝不写回 OpenCode 的 db**。

## 2. Collector 函数签名(Rust)

```rust
// src-tauri/src/collector/opencode.rs

use std::path::PathBuf;
use crate::collector::error::CollectorError;
use crate::collector::model::{RawSessionUsage, UsageAggregate, ModelBreakdown};

/// 解析 OpenCode 数据目录候选路径(按第 1 节优先级)。
/// 返回找到的第一个存在的 db 文件路径;都没有则 Err(NotFound)。
pub fn resolve_opencode_db() -> Result<PathBuf, CollectorError>;

/// 读取所有 session 的预聚合用量(直接读 session 表,不遍历 message)。
pub fn read_all_sessions(db_path: &PathBuf) -> Result<Vec<RawSessionUsage>, CollectorError>;

/// 将原始 session 用量聚合成前端所需结构(按时间窗口分桶 + 按 model 分组)。
pub fn aggregate(raw: &[RawSessionUsage], now_ms: i64) -> UsageAggregate;
```

`resolve_opencode_db` + `read_all_sessions` 失败时**返回 Result,不 panic**(规划第 4 章代码规范)。

## 3. 数据结构

```rust
// src-tauri/src/collector/model.rs

/// 单个 session 的原始用量(对应 session 表一行)。
pub struct RawSessionUsage {
    pub session_id: String,
    pub model_id: String,         // session.model 的 id(JSON 解出)
    pub provider_id: String,      // session.model 的 providerID
    pub cost: f64,
    pub tokens_input: i64,        // 非缓存输入
    pub tokens_output: i64,
    pub tokens_reasoning: i64,
    pub tokens_cache_read: i64,
    pub tokens_cache_write: i64,
    pub time_created: i64,        // epoch ms
}

/// 聚合结果 —— 由 collector 产出,再由 Tauri command 映射成前端 UsageData。
pub struct UsageAggregate {
    pub total_cost: f64,
    pub total_tokens: i64,        // input + output + reasoning + cache.read + cache.write
    pub window_5h: WindowStat,    // 最近 5 小时
    pub window_weekly: WindowStat,// 最近 7 天
    pub window_monthly: WindowStat,// 最近 30 天(本月)
    pub daily_history: Vec<DayBucket>, // 近 7 天每日 token,供折线图
    pub models: Vec<ModelBreakdown>,
}

pub struct WindowStat {
    pub tokens: i64,
    pub session_count: usize,
}

pub struct DayBucket {
    pub date: String,   // "周一" 等,或 ISO 日期,前端再格式化
    pub tokens: f64,    // 折线图用 M 单位
}

pub struct ModelBreakdown {
    pub name: String,
    pub percentage: f64,
    pub color: String,
}
```

> 注:规划文档原始 `UsageRecord { model, input, output, total }` 字段更少,这里扩展为 5 类 token + cost + 时间戳,以匹配真实 OpenCode schema。前端 `UsageData`(见 `web/src/types/index.ts`)结构不变。

## 4. SQLite Schema(本项目自用,非 OpenCode 的)

本项目缓存库 `usage-float.db`(见 `src-tauri/src/database.rs`),Phase 3 建表。三表:

```sql
-- account: 订阅/计划信息(Phase 2 暂用占位,真实数据需 provider 侧)
CREATE TABLE IF NOT EXISTS account (
    id INTEGER PRIMARY KEY,
    plan TEXT,
    status TEXT,           -- active | expired | error
    expire_date TEXT,
    updated_at INTEGER
);

-- quota: 时间窗口用量缓存(collector 按周期刷新)
CREATE TABLE IF NOT EXISTS quota (
    window TEXT PRIMARY KEY,  -- '5h' | 'weekly' | 'monthly'
    tokens INTEGER,
    percent REAL,
    reset_at TEXT,
    updated_at INTEGER
);

-- usage: 按 session / 按 day 的用量明细(供历史折线图)
CREATE TABLE IF NOT EXISTS usage (
    day TEXT,              -- ISO date
    model TEXT,
    tokens INTEGER,
    cost REAL,
    PRIMARY KEY (day, model)
);
```

## 5. Tauri Command 接口

Phase 1 已有 `get_usage_data`(返回 mock)。Phase 2 改为:

```rust
#[tauri::command]
fn get_usage_data() -> Result<UsageData, String> {
    // 1. resolve_opencode_db() —— 失败则回落 mock(无 OpenCode 环境)
    // 2. read_all_sessions()
    // 3. aggregate()
    // 4. 映射成 UsageData(含 daily_history → tokenHistory, models → models)
    // 5. 失败优雅回落 mock,不向前端抛错
}
```

前端 invoke 不变:`invoke<UsageData>('get_usage_data')`。
pi 负责:把 `get_usage_data` 从 mock 切换到调用 collector,并把 mock 作为 fallback。

## 6. 错误类型

```rust
// src-tauri/src/collector/error.rs
#[derive(Debug)]
pub enum CollectorError {
    NotFound,              // 找不到 OpenCode 数据目录 / db 文件
    OpenFailed(String),   // 打开 db 失败(权限/损坏)
    QueryFailed(String),  // SQL 查询失败(schema 不符/损坏)
    ParseFailed(String),  // model JSON 解析失败
}
```

命令层捕获 `CollectorError` 后**回落 mock**,前端永远收到有效 `UsageData`(规划第 6 章测试场景"无 OpenCode 环境/数据为空")。

## 7. 已知缺口:配额周期(5h / weekly / monthly)

**问题**:UI 设计规范要求显示"5小时额度 82% / 本周 63% / 本月 45%"百分比 + 重置倒计时。但 OpenCode 本地无配额周期数据,只有累计 token + 时间戳。

**初版策略(Phase 2 采用)**:
- 窗口 token:collector 用 `time_created` 分桶统计最近 5h / 7天 / 30天 的 token 用量(真实,来自本地)。
- 百分比:Phase 2 **无法从本地得到真实配额上限**。初版用可配置的"假定上限"(如月度假定上限来自设置项),或暂沿用 mock 百分比并标注"估算"。
- 重置倒计时:5h 窗口可由"最早一条仍在 5h 窗内的 session 时间戳"推算近似重置;weekly/monthly 用自然周/自然月边界。初版为占位字符串。

**后续选项(留待决策,不阻塞 Phase 2)**:
- (a) 接入 provider quota API(需用户授权 provider 凭证,偏离"完全离线"原则)。
- (b) 用户在设置里自填配额上限,collector 用真实用量 / 用户上限算百分比。
- (c) 仅展示"累计用量"而不展示百分比,UI 调整。

## 8. 测试场景清单(规划第 6 章)

omp 按此清单编写测试,opencode 的实现须全部通过:

- [ ] 无 OpenCode 环境:`resolve_opencode_db()` 返回 `NotFound`,命令回落 mock,前端拿到 mock `UsageData`。
- [ ] 数据为空:db 存在但 session 表无行,`aggregate` 返回全零结构(非 panic、非空指针)。
- [ ] 数据损坏:db 文件非 SQLite / schema 缺列,`OpenFailed` 或 `QueryFailed`,命令回落 mock。
- [ ] 多 session:`aggregate` 正确累加多个 session 的 token,按 model 正确分组。
- [ ] 时间窗口分桶:给定固定 `now_ms` 和若干 `time_created`,验证 5h/7d/30d 桶归属正确。
- [ ] Windows 权限:目录存在但无读权限,`OpenFailed`,命令回落 mock(不崩溃)。
- [ ] model JSON 解析:session.model 为合法/缺失 variant 的 JSON,`ParseFailed` 容错。

## 9. 文件归属

| 文件 | 负责 agent | 职责 |
|---|---|---|
| `src-tauri/src/collector/opencode.rs` | opencode | `resolve_opencode_db` / `read_all_sessions` / `aggregate` 实现 |
| `src-tauri/src/collector/model.rs` | opencode | 第 3 节数据结构 |
| `src-tauri/src/collector/error.rs` | opencode | 第 6 节错误类型 |
| `src-tauri/src/lib.rs` | pi | `get_usage_data` 从 mock 切换到 collector + fallback |
| `src-tauri/src/database.rs` | opencode | 第 4 节 schema 建表(Phase 3 接续) |
| `web/src/providers/tauri-provider.ts` | pi | 不变(invoke 接口一致) |
| `src-tauri/tests/` 或 `src-tauri/src/collector/` 内 `#[cfg(test)]` | omp | 第 8 节测试用例 |
