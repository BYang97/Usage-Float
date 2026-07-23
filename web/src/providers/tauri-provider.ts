import { invoke } from '@tauri-apps/api/core';
import type { UsageProvider, UsageData, AccountInfo, QuotaInfo, TokenInfo, ModelUsageData } from '../types';

/**
 * TauriProvider - 通过 Tauri invoke 调用 Rust command 获取数据
 *
 * 调用链：
 *   React 组件 → usage-service → TauriProvider.invoke → Rust command → 模拟数据返回
 *
 * 安全设计：
 * - 每个 invoke 独立 try/catch，单 command 失败不影响其他
 * - 所有 command 失败时返回 fallback 数据，应用不白屏
 */
export class TauriProvider implements UsageProvider {
  async getUsageData(): Promise<UsageData> {
    try {
      return await invoke<UsageData>('get_usage_data');
    } catch (err) {
      console.error('[TauriProvider] get_usage_data 失败，使用降级数据:', err);
      return {
        account: this.getFallbackAccount(),
        quota: this.getFallbackQuota(),
        tokens: this.getFallbackTokens(),
        models: this.getModelUsage(),
      };
    }
  }

  private getFallbackAccount(): AccountInfo {
    return { plan: '未知套餐', status: 'error', expireDate: '—' };
  }

  private getFallbackQuota(): QuotaInfo {
    return {
      fiveHourPercent: 0, fiveHourReset: '—',
      weeklyPercent: 0, weeklyReset: '—',
      monthlyPercent: 0,
    };
  }

  private getFallbackTokens(): TokenInfo {
    return {
      tokenToday: '—', token7d: '—', token30d: '—',
      tokenHistory: [
        { date: '暂无数据', tokens: 0 },
      ],
    };
  }

  /**
   * 模型用量数据暂在 TS 侧组装
   * 后续可迁移为独立的 Rust command
   */
  private getModelUsage(): ModelUsageData[] {
    return [
      { name: 'GPT 系列', percentage: 60, color: '#4a9eff' },
      { name: 'Claude 系列', percentage: 40, color: '#d97706' },
    ];
  }
}
