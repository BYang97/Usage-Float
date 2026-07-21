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
    const [account, quota, tokens] = await Promise.all([
      this.safeInvokeAccount(),
      this.safeInvokeQuota(),
      this.safeInvokeTokens(),
    ]);

    return {
      account,
      quota,
      tokens,
      models: this.getModelUsage(),
    };
  }

  private async safeInvokeAccount(): Promise<AccountInfo> {
    try {
      return await invoke<AccountInfo>('get_account_info');
    } catch (err) {
      console.error('[TauriProvider] get_account_info 失败，使用降级数据:', err);
      return this.getFallbackAccount();
    }
  }

  private async safeInvokeQuota(): Promise<QuotaInfo> {
    try {
      return await invoke<QuotaInfo>('get_quota_info');
    } catch (err) {
      console.error('[TauriProvider] get_quota_info 失败，使用降级数据:', err);
      return this.getFallbackQuota();
    }
  }

  private async safeInvokeTokens(): Promise<TokenInfo> {
    try {
      return await invoke<TokenInfo>('get_token_records');
    } catch (err) {
      console.error('[TauriProvider] get_token_records 失败，使用降级数据:', err);
      return this.getFallbackTokens();
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
