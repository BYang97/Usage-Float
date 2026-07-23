export type PlanStatus = 'active' | 'expired' | 'error';

export interface TokenRecord {
  date: string;
  tokens: number;
}

export interface ModelUsageData {
  name: string;
  percentage: number;
  color: string;
}

export interface AccountInfo {
  plan: string;
  status: PlanStatus;
  expireDate: string;
}

export interface QuotaInfo {
  fiveHourPercent: number;
  fiveHourReset: string;
  weeklyPercent: number;
  weeklyReset: string;
  monthlyPercent: number;
}

export interface TokenInfo {
  tokenToday: string;
  token7d: string;
  token30d: string;
  tokenHistory: TokenRecord[];
}

export interface UsageData {
  account: AccountInfo;
  quota: QuotaInfo;
  tokens: TokenInfo;
  models: ModelUsageData[];
}

export interface UsageProvider {
  getUsageData(): Promise<UsageData>;
}

export interface Account {
  id: string;
  name: string;
  workspace_id: string;
  auth_cookie: string;
  notes: string;
  created_at: number;
  updated_at: number;
}

export interface AccountForm {
  name: string;
  workspace_id: string;
  auth_cookie: string;
  notes: string;
}

export interface UsageResult {
  plan: string;
  status: PlanStatus;
  expireDate: string;
  fiveHourPercent: number;
  fiveHourReset: string;
  weeklyPercent: number;
  weeklyReset: string;
  monthlyPercent: number;
}

export interface AccountWithUsage {
  account: Account;
  usage: UsageResult | null;
}
