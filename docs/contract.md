# Collector 接口契约

> 本文件是测试(omp)、实现(opencode)、接线(pi)的共同依据。任何字段、签名、行为变更须先改本文件。
> **Phase 2 状态**:批次 1(本地 SQLite) ✅ 完成;批次 2(API 集成) ✅ 完成(v0.2)。
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

> **端点已确认**(2026-07-22 重新调研,参考 [opencode-go-dashboard](https://github.com/Ruinique/opencode-go-dashboard) + 真实 cookie 验证)。
> **旧方案(`console.opencode.ai/api/*` JSON API)已废弃** -- 真实请求全 401(端点不存在 / cookie 域不匹配)。配额数据在 opencode.ai 工作区页面的 React Server Component flight 数据里,不是独立 JSON API。

- **端点**: `GET https://opencode.ai/workspace/{workspaceId}/go`(HTML 页面)
- **认证**: `Cookie: auth=<Fe26.2**...>`(用户从浏览器 DevTools Application 面板复制)
- **必需参数**: `workspaceId`(`wrk_xxx`,从 opencode.ai 工作区 URL 获取,用户在设置面板粘贴)
- **客户端**: reqwest(rustls-tls,避开 Windows schannel 代理 MITM 握手失败),15s 超时,User-Agent 伪装浏览器
- **响应**: HTML,含 React Server Component flight 序列化数据:
  ```
  rollingUsage:$R[N]={status:"ok",usagePercent:1,resetInSec:7828}
  weeklyUsage:$R[N]={status:"ok",usagePercent:0,resetInSec:375487}
  monthlyUsage:$R[N]={status:"ok",usagePercent:87,resetInSec:481129}
  plan:$R[N]="go-monthly"
  ```

### 解析方式

正则提取(非 JSON 反序列化,因为 flight 数据的 key 无引号,不是合法 JSON):
- `{key}:$R[\d+]=({[^}]+})` -> 提取 `{...}` 对象,再正则取 `usagePercent` / `resetInSec`
- `rollingUsage` / `weeklyUsage` / `monthlyUsage` 各自独立提取
- `plan:$R[\d+]="([^"]+)"` -> plan 名称

### 三窗口(已分开,非共享)

- `rollingUsage` -> 5h 滚动窗口(`ApiQuota.five_hour`)
- `weeklyUsage` -> 周窗口(`ApiQuota.weekly`)
- `monthlyUsage` -> 月窗口(`ApiQuota.monthly`)
- 每窗口:`{ usagePercent: f64, resetInSec: i64 }`(已用百分比 + 重置秒数)

### 完整调用栈

```
fetch_api_quota(app_handle)
  ├─ database::get_quota_cache(三窗口) + get_account_cache(plan) -> 缓存命中(5min) -> 直接返回
  ├─ 否则:
  │   读 cookie + workspace_id(settings 表,加密)
  │   OpenCodeApiClient::new(cookie, workspace_id)
  │   └─ fetch_quota() -> GET /workspace/{ws}/go -> HTML
  │       ├─ 检查 /sign-in(cookie 过期)-> Unauthorized
  │       ├─ 正则提取 rolling/weekly/monthly(usagePercent + resetInSec)
  │       └─ 正则提取 plan
  │       └─ -> ApiQuota { five_hour, weekly, monthly, plan }
  └─ set_quota_cache(三窗口) + set_account_cache(plan)
```

```rust
// src-tauri/src/collector/api.rs — 已实现
pub struct OpenCodeApiClient {
    cookie: String,
    client: reqwest::Client,
}
impl OpenCodeApiClient {
    pub fn new(cookie: String) -> Self;
    /// 取真实配额(调用 GET /api/budgets/org)。
    pub async fn fetch_quota(&self) -> Result<ApiQuota, CollectorError>;
    /// 取账户信息(调用 GET /api/billing/status)。
    pub async fn fetch_account(&self) -> Result<ApiAccount, CollectorError>;
}
```

```rust
// src-tauri/src/collector/model.rs — API 侧结构
pub struct ApiQuota {
    pub five_hour:  ApiWindow,   // 已用/上限/重置(来自 API,micro-cents 转 token 近似)
    pub weekly:     ApiWindow,
    pub monthly:    ApiWindow,
}
pub struct ApiWindow {
    pub usage_percent: f64,    // 已用百分比(0-100,从 Go 页面 usagePercent)
    pub reset_in_sec: i64,     // 重置倒计时(秒,从 Go 页面 resetInSec)
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

命令层组合两个数据源(已实现):
```rust
// src-tauri/src/lib.rs
#[tauri::command]
async fn get_usage_data(app_handle: tauri::AppHandle) -> Result<UsageData, String> {
    // 1. 本地 SQLite 采集 → resolve_local_data() (失败→空 aggregate)
    // 2. API 配额 → fetch_api_quota() (失败→ None)
    //    2a. 先读 quota + account 缓存(5min TTL)
    //    2b. 未命中→ OpenCodeApiClient → fetch + 写缓存
    // 3. map_local_to_usage_data(local, api_quota, api_account)
    //    → build_quota_info() API 优先, mock 兜底
    //    → build_account_info() API 优先, mock 兜底
    // 4. 全失败 → mock::mock_usage_data()
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

四张表均已实现,由 `database::init_schema()` 创建。

```sql
-- settings: KV 设置,存 auth cookie(加密)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value BLOB,            -- cookie 等敏感值加密存储
    updated_at INTEGER
);

-- account: 订阅缓存(来自 API,5min TTL)
CREATE TABLE IF NOT EXISTS account (
    id INTEGER PRIMARY KEY,
    plan TEXT, status TEXT, expire_date TEXT, updated_at INTEGER
);

-- quota: 配额缓存(来自 API,5min TTL)
CREATE TABLE IF NOT EXISTS quota (
    window TEXT PRIMARY KEY,   -- 'five_hour'|'weekly'|'monthly'
    used INTEGER, limit INTEGER, percent REAL, reset_at TEXT, updated_at INTEGER
);

-- usage: 按 day/model 用量明细(来自本地 SQLite,供历史折线图)
CREATE TABLE IF NOT EXISTS usage (
    day TEXT, model TEXT, tokens INTEGER, cost REAL,
    PRIMARY KEY (day, model)
);
```

### 缓存读写函数(`database.rs`)

| 函数 | 作用 |
|---|---|
| `get_quota_cache(conn, "window")` | 读 quota,检查 `updated_at` 5min TTL,过期→None |
| `set_quota_cache(conn, "window", used, limit, reset_at)` | 写入 quota |
| `get_account_cache(conn)` | 读 account id=1,检查 TTL |
| `set_account_cache(conn, plan, status, expire_date)` | 写入 account |

### Cookie 加密

选型: `ring` AEAD AES-256-GCM + 机器绑定密钥(COMPUTERNAME/HOSTNAME → SHA256)。
- `encrypt(plaintext)` → nonce(12B) + ciphertext + AEAD tag
- `decrypt(ciphertext)` → 分离 nonce 后解密
- 密文存入 `settings.value(BLOB)`

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

## 7. 配额周期处理(已实现)

- 5h/周/月**真实配额**(已用/上限/重置)来自 opencode.ai API(第 2 节),**不是**本地分桶。
- 本地 SQLite 提供:累计 token、逐日历史、按 model 分组。
- `fetch_quota()` → `GET /api/budgets/org` → `{ spentMicroCents, limitMicroCents, resetsAt }`
  → micro-cents 转 token(÷100000) → 三窗口共享同一值
- `get_billing_status()` → `GET /api/billing/status` → `{ billingMode, managedInferenceStatus }`
- `get_expire_date()` → `GET /api/billing/seat-billing` → `{ subscription.renewalAt | period.endsAt }`
- 配额缓存: `quota` + `account` 表,5 分钟 TTL
- 无 cookie / API 不可达:配额百分比与重置倒计时取 mock 占位,token 数据仍真实。
- 任何异常 → 降级到 mock,前端永远收到有效 `UsageData`/"—"),token 数据仍真实。

## 8. 测试场景清单(规划第 6 章 + API 场景)

omp 按此清单编写测试,opencode/pi 的实现须全部通过:

**本地 SQLite 采集:**
- [x] 无 OpenCode 环境:`resolve_opencode_db()` 返回 `NotFound`,命令回落 mock。
- [x] 数据为空:db 存在但 session 表无行,`aggregate_local` 返回全零结构(非 panic)。
- [x] 数据损坏:db 非 SQLite / schema 缺列,`OpenFailed`/`QueryFailed`,命令回落 mock。
- [x] 多 session:`aggregate_local` 正确累加,按 model 正确分组。
- [x] 时间窗口分桶:给定固定 `now_ms` 和若干 `time_created`,验证 7d 桶归属正确。
- [x] Windows 权限:目录存在但无读权限,`OpenFailed`,命令降级不崩溃。
- [x] model JSON 解析:合法/缺 variant 的 JSON,`ParseFailed` 容错。

**opencode.ai API(用 mock HTTP server 测试,需 httpmock crate):**
- [ ] 有 cookie 且 API 返回配额:`fetch_quota` 正确解析成 `ApiQuota`,合并进 `UsageData`。
- [ ] 无 cookie:`NoCookie`,配额缺失,token 正常,前端拿到 token-only `UsageData`。
- [ ] cookie 失效(401):`Unauthorized`,配额缺失,token 正常。
- [ ] API 超时/5xx:`ApiError`,降级,token 正常。
- [x] 配额缓存: `get_quota_cache`/`set_quota_cache` 实现,5min TTL,重复 invoke 不重复打 API。

## 9. 文件归属(当前状态)

| 文件 | 负责 agent | 职责 | 状态 |
|---|---|---|---|
| `src-tauri/src/collector/opencode.rs` | opencode | 本地:`resolve_opencode_db`/`read_all_sessions`/`aggregate_local` | ✅ 完成 |
| `src-tauri/src/collector/api.rs` | opencode | `OpenCodeApiClient`: `fetch_quota`/`fetch_account`/`send_get` | ✅ 完成 |
| `src-tauri/src/collector/model.rs` | opencode | `RawSessionUsage`/`LocalAggregate`/`ApiQuota`/`ApiAccount` | ✅ 完成 |
| `src-tauri/src/collector/error.rs` | opencode | `CollectorError` 枚举 | ✅ 完成 |
| `src-tauri/src/database.rs` | opencode | schema + cookie 加密 + quota/account 缓存 | ✅ 完成 |
| `src-tauri/src/lib.rs` | pi | `get_usage_data` 双数据源合并 + fallback;settings 命令 | ✅ 完成 |
| `web/src/providers/tauri-provider.ts` | pi | invoke 通路(未改动) | ✅ 不变 |
| `web/src/pages/Settings.tsx` | pi | auth cookie 粘贴框 | ✅ 完成 |
| 测试(本地) | omp | 第 8 节本地 SQLite 场景(8 个测试) | ✅ 通过 |
| 测试(API) | omp | mock HTTP server,需 httpmock crate | ⏳ 待补 |

## 10. 分派批次(已完成)

**批次 1(本地 SQLite 采集):** ✅
- omp: 本地 SQLite 采集测试 7 项 + API 测试骨架 → **8 测试通过**
- opencode: `opencode.rs`/`model.rs`/`error.rs` + `database.rs` schema → ✅
- pi: Settings + cookie 命令 + `get_usage_data`(本地采集 + mock 兜底) → ✅

**批次 2(opencode.ai API 集成):** ✅
- opencode: `api.rs` — `OpenCodeApiClient` 实现(端点 `budgets/org`, `billing/status`, `billing/seat-billing`) → ✅
- pi: `get_usage_data` 接入 API 配额 + `fetch_api_quota` 缓存逻辑 → ✅
- database: 新增 quota/account 缓存读写(5min TTL) → ✅
- opencode: `api.rs` 预写响应结构体反序列化测试(3 项) → ✅
- omp: API mock 测试(需 httpmock crate) → ⏳ 待补

### 测试总结

```
cargo test
  11 passed; 0 failed
  ├─ collector (本地 SQLite): 8 项
  ├─ api (反序列化): 3 项
  └─ 共 0 忽略,0 测量
```
