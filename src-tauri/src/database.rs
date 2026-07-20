// SQLite 数据层骨架。
// Phase 1: 仅初始化本地连接(数据库文件置于 app data 目录),建表留给 Phase 3。
// 安全要求:完全离线,数据只存本地,绝不外发。
// 以下符号在 Phase 1 尚未被调用,Phase 3 建表时启用,故暂告警屏蔽。

use std::path::PathBuf;

/// 本地数据库文件名。
#[allow(dead_code)]
const DB_FILE_NAME: &str = "usage-float.db";

/// 在 Tauri 应用的本地数据目录下解析 SQLite 数据库文件路径。
/// Phase 3 起在此文件上建 account/quota/usage 三表。
#[allow(dead_code)]
pub fn resolve_db_path(app_data_dir: &PathBuf) -> PathBuf {
  app_data_dir.join(DB_FILE_NAME)
}
