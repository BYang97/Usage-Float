import type { UsageProvider, UsageData } from '../types';
import { usage } from '../data/mock';

export class MockProvider implements UsageProvider {
  async getUsageData(): Promise<UsageData> {
    return {
      account: {
        plan: usage.plan,
        status: usage.status,
        expireDate: usage.expireDate,
      },
      quota: {
        fiveHourPercent: usage.fiveHourPercent,
        fiveHourReset: usage.fiveHourReset,
        weeklyPercent: usage.weeklyPercent,
        weeklyReset: usage.weeklyReset,
        monthlyPercent: usage.monthlyPercent,
      },
      tokens: {
        tokenToday: usage.tokenToday,
        token7d: usage.token7d,
        token30d: usage.token30d,
        tokenHistory: usage.tokenHistory,
      },
      models: usage.models,
    };
  }
}
