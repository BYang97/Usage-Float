# 任务:cache-ttl-test - 缓存读写 + TTL 单测

## 目标

补 `database.rs` 的 `get_quota_cache`/`set_quota_cache`/`get_account_cache`/`set_account_cache` 读写单测。

## 背景

- 缓存逻辑:`set_quota_cache(window, usage_percent, reset_in_sec)` 写 quota 表;`get_quota_cache(window)` 读,带 5min TTL(`CACHE_TTL_MS`)。
- account 缓存:`set_account_cache(plan)` / `get_account_cache()` 返回 `Option<String>`(plan)。
- 当前无单测(只有 api httpmock 测试 + collector 测试)。

## 改动

### `src-tauri/tests/cache_test.rs` - 新建

用 `tempfile` 建临时 db,测试:

```rust
use app_lib::database;
use rusqlite::Connection;

fn open_temp_db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    // 注意:dir 作用域结束后自动删,测试内保持
    database::init_schema(&db_path).unwrap();
    Connection::open(&db_path).unwrap()
    // 注意:tempdir 的 dir 要在测试函数内保持存活,用 Keep 或 leak
}
```

测试用例:
1. `set_quota_cache` 写入 `("five_hour", 87.0, 481129)`,`get_quota_cache("five_hour")` 返回 `Some((87.0, 481129))`
2. 三个窗口(five_hour/weekly/monthly)各写入不同值,各读取验证
3. `get_quota_cache` 未写入的窗口返回 `None`
4. `set_account_cache("go-monthly")` 写入,`get_account_cache()` 返回 `Some("go-monthly")`
5. `get_account_cache` 未写入返回 `None`

## TTL 过期测试(可选,难注入时间)

`get_quota_cache` 用 `SystemTime::now()` 判断 TTL,测试难模拟时间前进。**可选跳过**,或:
- 测试只验证写入后立即读取命中(未过期)
- TTL 过期逻辑通过代码审查确认(不做单测)

## 验证

```bash
cd src-tauri && cargo test --test cache_test
```

## 注意

- `database::init_schema(db_path: &Path)` 是 pub,可直接调建表。
- `open_db` 要 `app_data_dir`(Tauri 路径),测试不用 `open_db`,直接 `init_schema` + `Connection::open`。
- tempfile 的 tempdir 作用域:在测试函数内保持 dir 存活(不提前 drop),或用 `tempfile::tempdir().keep()` 保留。
- 不依赖网络,纯本地 SQLite。
- `database` 模块要在 lib.rs 是 `pub mod database;`(当前是 `mod database;` 私有)-- 如果测试访问不了,改 `pub mod database;`。
