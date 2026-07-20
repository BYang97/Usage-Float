mod database;
mod mock;
mod models;

use models::UsageData;

/// 返回用量数据。
/// Phase 1 返回 mock;Phase 2 起由 collector 采集 + database 缓存。
#[tauri::command]
fn get_usage_data() -> UsageData {
  mock::mock_usage_data()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![get_usage_data])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
