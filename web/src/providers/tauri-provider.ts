import { invoke } from '@tauri-apps/api/core';
import type { UsageProvider, UsageData } from '../types';

/**
 * Tauri 侧 provider:通过 invoke 调用 Rust 的 `get_usage_data` 命令。
 * 仅在 Tauri 运行时内可用;纯 web(`bun run dev` 无 Tauri)请用 MockProvider。
 */
export class TauriProvider implements UsageProvider {
  async getUsageData(): Promise<UsageData> {
    return invoke<UsageData>('get_usage_data');
  }
}
