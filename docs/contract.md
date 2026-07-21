# Collector 接口契约

> 本文件是测试(omp)、实现(opencode)、接线(pi)的共同依据。任何字段、签名、行为变更须先改本文件。
> Phase 2 验收以本文件 + omp 编写的测试用例为准。
> 数据格式事实依据见 [opencode-data-format.md](./opencode-data-format.md)。

## 0. 范围与双数据源

Phase 2 collector 的目标:组合两个数据源,聚合成前端 `UsageData`,经 Tauri command 返回。

| 数据源 | 取什么 | 怎么取 | 是否联网 |
|---|---|---|---|
| **本地 OpenCode SQLite** | 累计 token、cost、按 model 分组、按天历史 | rusqlite 只读 `opencode.db` | 离线 |
| **opencode.ai API** | 真实 5h/weekly/monthly 配额(已用/上限/重置)、plan/status/到期 | reqwest 带 auth cookie | 联网(仅官方域) |

**关键事实**:OpenCode 本地只有累计 token + 时间戳,**无配额周期**;5h/周/月配额在 opencode.ai 托管 plan(provider `opencode`/`opencode-go`)服务端。配额必须经 API 取。

**安全边界**(修订规划第 5 章):
- auth cookie 由**用户主动在设置面板粘贴**(collector 绝不读取浏览器)。
- cookie 仅存本地(`settings` 表,加密),**仅发往 `opencode.ai` 官方域**,**不外发任何第三方**。
- 无 cookie 或 API 不可达时,优雅降级(配额百分比缺失,token 累计仍正常显示)。
- 本地 SQLite 读取始终离线,不依赖网络。

## 1. 本地 OpenCode SQLite 数据源

- 文件:本地 SQLite,默认 `opencode.db`(渠道变体 `opencode-beta.db`/`opencode-latest.db`)。
- 目录解析优先级:
  1. 环境变量 `OPENCODE_DB`(绝对路径直接用;相对则相对 data 目录)
  2. `<XDG_DATA_HOME>/opencode/`(默认 `~/.local/share/opencode`;Windows 为 `%USERPROFILE%\.local\share\opencode`)
  3. 在上述目录按渠道找 `opencode[-channel].db`
- 打开方式:只读,WAL 兼容(rusqlite `OpenFlags::SQLITE_OPEN_READ_ONLY`)。**绝不写回 OpenCode 的 db**。
- 读 session 表预聚合列(见 [opencode-data-format.md](./opencode-data-format.md) 第 3.1 节)。

## 2. opencode.ai API 数据源(配额)

> **端点与响应格式待 [opencode.ai API 调研] 补完**(进行中)。本节先定抽象接口,端点/字段确认后回填。

- 认证:`Cookie: auth=<Fe26.2**...>`(用户从浏览器 DevTools 的 opencode.ai 请求 Cookie 头复制粘贴)。
- 客户端:reqwest,仅请求 `opencode.ai` 域,设合理超时(如 10s),失败降级不 panic。
- 取得:5h/weekly/monthly 的已用 token / 上限 / 重置时间;plan 名称 / status / 到期日;按 model 用量(若 API 提供)。
- 缓存:结果写入本地 `quota` 表,避免每次 invoke 都打 API(带 `updated_at`,超过刷新间隔才重取)。

```rust
// src-tauri/src/collector/api.rs
pub struct OpenCodeApiClient {
    cookie: String,        // 用户粘贴的 auth cookie
    // 端点常量待调研回填
}
impl OpenCodeApiClient {
    pub fn new(cookie: String) -> Self;
    /// 取真实配额(5h/周/月 + 重置)。失败返回 Err,由上层降级。
    pub async fn fetch_quota(&self) -> Result<ApiQuota, CollectorError>;
    /// 取账户信息(plan/status/到期)。失败返回 Err。
    pub async fn fetch_account(&self) -> Result<ApiAccount, CollectorError>;
}
```

```rust
// src-tauri/src/collector/model.rs (API 侧结构,字段待调研回填)
pub struct ApiQuota {
    pub five_hour:  ApiWindow,   // 已用/上限/重置(来自 API)
    pub weekly:     ApiWindow,
    pub monthly:    ApiWindow,
}
pub struct ApiWindow {
    pub used: Option<i64>,
    pub limit: Option<i64>,
    pub reset_at: Option<String>,   // 重置时间,格式待定
}
pub struct ApiAccount {
    pub plan: Option<String>,
    pub status: Option<String>,
    pub expire_date: Option<String>,
}
```

## 3. Collector 函数签名(Rust)

```rust
// src-tauri/src/collector/opencode.rs

use std::path::PathBuf;
use crate::collector::error::CollectorError;
use crate::collector::model::*;

/// 解析 OpenCode 本地 db 路径(按第 1 节优先级)。都没有则 Err(NotFound)。
pub fn resolve_opencode_db() -> Result<PathBuf, CollectorError>;

/// 读取所有 session 预聚合用量(直接读 session 表)。失败返回 Result,不 panic。
pub fn read_all_sessions(db_path: &PathBuf) -> Result<Vec<RawSessionUsage>, CollectorError>;

/// 本地用量聚合:按时间窗口分桶(5h/7d/30d)+ 按 model 分组 + 逐日历史。
pub fn aggregate_local(raw: &[RawSessionUsage], now_ms: i64) -> LocalAggregate;
```

命令层组合两个数据源:
```rust
// src-tauri/src/lib.rs (pi 实现)
#[tauri::command]
async fn get_usage_data(state: State<AppState>) -> Result<UsageData, String> {
    // 1. 本地 SQLite 采集(始终尝试,失败回落零值)
    // 2. 若 settings 有 cookie:调 opencode.ai API 取配额 + 账户(失败降级:仅展示 token)
    // 3. 合并成本地 token + API 配额 → UsageData
    // 4. 任何失败都回落到 mock,前端永远收到有效 UsageData
}
```

## 4. 数据结构

```rust
// src-tauri/src/collector/model.rs

/// 单个 session 原始用量(session 表一行)。
pub struct RawSessionUsage {
    pub session_id: String,
    pub model_id: String,        // session.model JSON 的 id
    pub provider_id: String,
    pub cost: f64,
    pub tokens_input: i64,       // 非缓存输入
    pub tokens_output: i64,
    pub tokens_reasoning: i64,
    pub tokens_cache_read: i64,
    pub tokens_cache_write: i64,
    pub time_created: i64,       // epoch ms
}

/// 本地聚合结果(token 来自 SQLite)。
pub struct LocalAggregate {
    pub total_tokens: i64,        // input+output+reasoning+cache.read+cache.write
    pub total_cost: f64,
    pub daily_history: Vec<DayBucket>,   // 近 7 天,供折线图
    pub models: Vec<ModelBreakdown>,
}
pub struct DayBucket { pub date: String, pub tokens: f64 }   // 折线图 M 单位
pub struct ModelBreakdown { pub name: String, pub percentage: f64, pub color: String }
```

> 前端 `UsageData`(见 `web/src/types/index.ts`)结构不变。Rust 命令层负责把 `LocalAggregate` + `ApiQuota` + `ApiAccount` 合并映射成 `UsageData`。

## 5. SQLite Schema(本项目自用 `usage-float.db`)

Phase 3 建表。Phase 2 需 `settings` 存 cookie:

```sql
-- settings: KV 设置,存 auth cookie(加密)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value BLOB,            -- cookie 等敏感值加密存储
    updated_at INTEGER
);

-- account: 订阅信息(来自 opencode.ai API,带缓存)
CREATE TABLE IF NOT EXISTS account (
    id INTEGER PRIMARY KEY,
    plan TEXT, status TEXT, expire_date TEXT, updated_at INTEGER
);

-- quota: 时间窗口配额缓存(来自 API,按刷新间隔重取)
CREATE TABLE IF NOT EXISTS quota (
    window TEXT PRIMARY KEY,   -- '5h'|'weekly'|'monthly'
    used INTEGER, limit INTEGER, percent REAL, reset_at TEXT, updated_at INTEGER
);

-- usage: 按 day/model 用量明细(来自本地 SQLite,供历史折线图)
CREATE TABLE IF NOT EXISTS usage (
    day TEXT, model TEXT, tokens INTEGER, cost REAL,
    PRIMARY KEY (day, model)
);
```

cookie 加密:Windows 用 DPAPI(`crypt32`)或项目级密钥;Phase 2 初版可用 `ring` + 机器绑定密钥,具体由 opencode 选型。

## 6. 错误类型

```rust
// src-tauri/src/collector/error.rs
#[derive(Debug)]
pub enum CollectorError {
    NotFound,                // 找不到 OpenCode 数据目录 / db
    OpenFailed(String),      // 打开 db 失败(权限/损坏)
    QueryFailed(String),     // SQL 查询失败(schema 不符)
    ParseFailed(String),     // model JSON 解析失败
    NoCookie,                // 用户未设置 auth cookie(API 功能不可用,非致命)
    ApiError(String),        // opencode.ai API 请求失败(网络/5xx)
    Unauthorized,            // cookie 失效/过期(API 返回 401)
}
```

命令层捕获后**降级而非抛错**:本地失败→mock;API 失败→仅展示 token(配额缺失);任何情况前端收到有效 `UsageData`。

## 7. 配额周期处理(已定方向)

- 5h/周/月**真实配额**(已用/上限/重置)来自 opencode.ai API(第 2 节),**不是**本地分桶。
- 本地 SQLite 提供:累计 token、逐日历史、按 model 分组。
- 无 cookie / API 不可达:配额百分比与重置倒计时缺失(前端显示"未连接"/"—"),token 数据仍真实。

## 8. 测试场景清单(规划第 6 章 + API 场景)

omp 按此清单编写测试,opencode/pi 的实现须全部通过:

**本地 SQLite 采集:**
- [ ] 无 OpenCode 环境:`resolve_opencode_db()` 返回 `NotFound`,命令回落 mock。
- [ ] 数据为空:db 存在但 session 表无行,`aggregate_local` 返回全零结构(非 panic)。
- [ ] 数据损坏:db 非 SQLite / schema 缺列,`OpenFailed`/`QueryFailed`,命令回落 mock。
- [ ] 多 session:`aggregate_local` 正确累加,按 model 正确分组。
- [ ] 时间窗口分桶:给定固定 `now_ms` 和若干 `time_created`,验证 7d/30d 桶归属正确。
- [ ] Windows 权限:目录存在但无读权限,`OpenFailed`,命令降级不崩溃。
- [ ] model JSON 解析:合法/缺 variant 的 JSON,`ParseFailed` 容错。

**opencode.ai API(用 mock HTTP server 测试):**
- [ ] 有 cookie 且 API 返回配额:`fetch_quota` 正确解析成 `ApiQuota`,合并进 `UsageData`。
- [ ] 无 cookie:`NoCookie`,配额缺失,token 正常,前端拿到 token-only `UsageData`。
- [ ] cookie 失效(401):`Unauthorized`,配额缺失,token 正常。
- [ ] API 超时/5xx:`ApiError`,降级,token 正常。
- [ ] 配额缓存:重复 invoke 在刷新间隔内不重复打 API(读 `quota` 表)。

## 9. 文件归属

| 文件 | 负责 agent | 职责 |
|---|---|---|
| `src-tauri/src/collector/opencode.rs` | opencode | 本地:`resolve_opencode_db`/`read_all_sessions`/`aggregate_local` |
| `src-tauri/src/collector/api.rs` | opencode | opencode.ai API 客户端(端点待调研回填) |
| `src-tauri/src/collector/model.rs` | opencode | 第 4 节 + 第 2 节 API 侧数据结构 |
| `src-tauri/src/collector/error.rs` | opencode | 第 6 节错误类型 |
| `src-tauri/src/database.rs` | opencode | 第 5 节 schema + cookie 加密存储 |
| `src-tauri/src/lib.rs` | pi | `get_usage_data` 组合双数据源 + fallback;settings 读写命令 |
| `web/src/providers/tauri-provider.ts` | pi | 不变(invoke 接口一致) |
| `web/src/pages/Settings.tsx` | pi | 新增 auth cookie 粘贴框 |
| 测试 | omp | 第 8 节全部场景(本地用真实临时 db,API 用 mock HTTP server) |

## 10. 分派批次

为最大化并行且不被 API 端点调研阻塞:

**批次 1(立即可并行,不依赖 API 端点):**
- omp:写本地 SQLite 采集的测试场景(第 8 节前 7 项)+ API 测试骨架(用 mock server,端点占位)。
- opencode:实现本地 SQLite 采集(`opencode.rs`/`model.rs`/`error.rs`)+ `database.rs` schema。
- pi:前端 Settings 加 cookie 粘贴框 + Rust 侧 settings 命令 + `get_usage_data` 接本地采集(配额暂占位)。

**批次 2(待 opencode.ai API 调研返回):**
- opencode 续:实现 `api.rs` 客户端(按调研回填的端点/字段)。
- pi 续:`get_usage_data` 接入 API 配额,替换占位。
- omp 续:补完 API 测试的端点断言。

合并验收(任务 #7)在批次 2 完成后进行。
