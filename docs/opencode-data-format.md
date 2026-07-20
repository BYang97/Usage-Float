# OpenCode 本地数据存储格式调研报告

> 调研对象:`sst/opencode`(GitHub 重定向至 `anomalyco/opencode`,仓库 ID 975734319,默认分支 `dev`)。
> 全部结论基于 dev 分支源码直接核对(128 次工具调用)。本文件是 collector 实现的事实依据。

## 总览

OpenCode 现在的本地存储是**单个 SQLite 文件(`opencode.db`)**,而非 JSONL,也非按 session 分目录的 JSON。历史上曾有过基于 JSON 文件的 `storage/` 目录与旧 wrapper `packages/opencode/src/storage/db.ts`,但后者已删除(spec `specs/storage/remove-opencode-db.md`),session 用量回填改由 core 迁移 `20260510033149_session_usage.ts` 完成。

## 1. 数据目录

源码:`packages/core/src/global.ts` + `packages/core/src/database/database.ts`。来自 npm `xdg-basedir` 的 `xdgData`(`XDG_DATA_HOME || ~/.local/share`,**无** macOS/Windows 特殊回退):

| 平台 | 实际路径(默认) |
|---|---|
| Linux | `$XDG_DATA_HOME/opencode` 或 `~/.local/share/opencode` |
| macOS | `~/.local/share/opencode`(不是 `~/Library/Application Support`,xdg-basedir 不做 macOS 回退) |
| Windows | `%USERPROFILE%\.local\share\opencode`(即 `C:\Users\<user>\.local\share\opencode`;**不是** `%APPDATA%`/`%LOCALAPPDATA%`,也不是 `~/.opencode`) |

可覆盖:
- `XDG_DATA_HOME` 覆盖整条路径(数据目录变 `$XDG_DATA_HOME/opencode`)。
- `OPENCODE_DB` 整体覆盖 db 文件:绝对路径直接用;`:memory:` 内存库;相对路径则相对 `Global.Path.data`。
- `OPENCODE_DISABLE_CHANNEL_DB=1|true` 强制用 `opencode.db`(忽略渠道)。
- CLI 打印路径:`opencode db path`;`opencode db` 打开 sqlite3 shell。

db 文件名默认按渠道: `opencode.db`(prod)、`opencode-beta.db`、`opencode-latest.db`;其他渠道 `opencode-<channel>.db`。

## 2. 文件格式与表

格式: SQLite(单文件),WAL 模式(`PRAGMA journal_mode=WAL` 等)。扁平结构,无分目录。

关键表(Drizzle 定义在 `packages/core/src/session/sql.ts`,DDL 见 `packages/core/src/database/schema.gen.ts`):

| 表名 | 主键 | 作用 |
|---|---|---|
| `session` | `id` | 一行一个 session,**预聚合** `cost`/`tokens_*` 列、`title`、`project_id`、`workspace_id`、`parent_id`、`directory`、`model`(JSON)、`agent`、`time_*`、`metadata` |
| `message` | `id` | V1 message,`data`(JSON),`role='assistant'` 行带 cost/tokens。FK 到 session |
| `part` | `id` | V1 part,`type="step-finish"` 的 part 携带 usage |
| `session_message` | `id` | V2 message 投影,`(session_id, seq)` 唯一索引 |

命名: 不按日期/hash 分文件。`session.id` 为字符串品牌类型;`message.id` 为 `msg_` 开头。session 表另有 `slug`、`title`、`version`、`time_created`/`time_updated`(integer,epoch 毫秒)、可选 `time_archived`/`time_compacting`。

## 3. token usage 字段

### 3.1 session 表(预聚合,collector 直接读最省事)

| 列名 | 类型 | 含义 |
|---|---|---|
| `cost` | real | session 累计美元成本 |
| `tokens_input` | integer | 累计非缓存输入 token(`nonCachedInputTokens`) |
| `tokens_output` | integer | 累计可见输出 token(`visibleOutputTokens`) |
| `tokens_reasoning` | integer | 累计思考 token(`reasoningTokens`) |
| `tokens_cache_read` | integer | 累计缓存读 token |
| `tokens_cache_write` | integer | 累计缓存写 token |
| `model` | text(JSON `{id, providerID, variant?}`) | session 当前模型 |
| `time_created`/`time_updated` | integer(epoch ms) | 时间戳 |

TS 层映射(`packages/schema/src/session.ts`):
```ts
tokens: Schema.Struct({
  input: Schema.Finite,
  output: Schema.Finite,
  reasoning: Schema.Finite,
  cache: Schema.Struct({ read: Schema.Finite, write: Schema.Finite }),
})
```

### 3.2 token 来源映射(`packages/core/src/session/runner/publish-llm-event.ts`)

| OpenCode 内部 | ← provider usage |
|---|---|
| `input` | `usage.nonCachedInputTokens` |
| `output` | `usage.visibleOutputTokens` |
| `reasoning` | `usage.reasoningTokens` |
| `cache.read` | `usage.cacheReadInputTokens` |
| `cache.write` | `usage.cacheWriteInputTokens` |

注意:provider 的 `totalTokens` 未映射进表。`input` 是**非缓存**输入,所以"等价于 Anthropic 的 input tokens" = `input + cache.read`,非只取 `input`。

### 3.3 聚合逻辑

- **逐 message 写入**: `packages/core/src/session/runner/publish-llm-event.ts`
- **session 表累加**: `packages/core/src/session/projector.ts` 的 `applyUsage()`(第 92-108 行),`UPDATE session SET cost = cost + value.cost, tokens_input = tokens_input + value.tokens.input, ...`
- **回填迁移**: `packages/core/src/database/migration/20260510033149_session_usage.ts`(从 `message.data` 用 `json_extract` 聚合)
- **CLI 聚合样例**: `packages/opencode/src/cli/cmd/stats.ts`

## 4. 配额周期("5h/weekly/monthly")— 关键结论

**这些周期不是 OpenCode 本地数据原概念,OpenCode 也不计算配额周期。本地只存累计 cost + tokens + 时间戳,无任何 5h/weekly/monthly 重置时间字段或表。**

证据:
1. `session` 表无 `window`/`reset_at`/`quota`/`period` 列。
2. 用量超限 UI(`packages/app/src/pages/session/usage-exceeded-dialogs.tsx`)只对 provider 为 `opencode`/`opencode-go` 的 `SessionStatus.retry.action` 反应,`action.reason` 为 `"free_tier_limit"`/`"account_rate_limit"` —— **配额来自 provider 服务端**,OpenCode 只透传 `action.{title, message, label, link}` 弹窗。`GO_UPSELL_WINDOW = 86_400_000`(24h)是弹窗去抖,非配额周期。
3. `SessionStatus` schema 无结构化重置时间字段,重置时间以文本塞在 `action.message` 透传。
4. `formatResetTime` 在 `packages/console/app/src/lib/format-reset-time.ts`(Web 控制台/SaaS 计费前端),**非本地 CLI**。

**对 collector 的含义**: 本地只能输出"累计 cost + 各 token 类别 + 时间戳";5h/周/月窗口需 collector 自己用 `time_*` 分桶模拟,或从 provider quota API 另取。

## 5. session/message/turn 层级与聚合

```
project (project_id)
  └─ session (id, parent_id 可成树/子 session)
       ├─ session_input        (用户 prompt 输入队列)
       ├─ session_context_epoch(上下文快照)
       ├─ todo                 (session 级 TODO)
       └─ message (V1) / session_message (V2)
             └─ part (V1, 多个 part 组成一条 message)
                  └─ type="step-finish" 的 part 携带 {cost, tokens}  ← 单次 turn 用量源头
```

"turn": 一次 LLM 调用 = 一个 assistant message = 一个 "step"。

**从 message 聚合某 session 总用量** — 两条等价路径:
- (A) 直接读预聚合列(推荐): `SELECT cost, tokens_input, tokens_output, tokens_reasoning, tokens_cache_read, tokens_cache_write FROM session WHERE id=?`
- (B) 从 message 重算(交叉验证):
```sql
SELECT
  coalesce(sum(json_extract(message.data,'$.cost')),0) AS cost,
  coalesce(sum(json_extract(message.data,'$.tokens.input')),0)     AS tokens_input,
  coalesce(sum(json_extract(message.data,'$.tokens.output')),0)    AS tokens_output,
  coalesce(sum(json_extract(message.data,'$.tokens.reasoning')),0) AS tokens_reasoning,
  coalesce(sum(json_extract(message.data,'$.tokens.cache.read')),0) AS tokens_cache_read,
  coalesce(sum(json_extract(message.data,'$.tokens.cache.write')),0)AS tokens_cache_write
FROM message
WHERE session_id = ? AND json_extract(message.data,'$.role')='assistant';
```

每 session 总 token(CLI `stats.ts` 口径): `total = input + output + reasoning + cache.read + cache.write`。

## 仓库追溯

`github.com/sst/opencode` 在 GitHub 重定向到 `github.com/anomalyco/opencode`(仓库 ID 975734319,默认分支 `dev`)。GitHub 链接会自动跟随重定向。
