import type { UsageProvider, UsageData } from '../types';
import { MockProvider } from '../providers/mock-provider';

let provider: UsageProvider = new MockProvider();

export function setProvider(p: UsageProvider) {
  provider = p;
}

let cached: UsageData | null = null;

export async function getUsageData(): Promise<UsageData> {
  if (!cached) {
    cached = await provider.getUsageData();
  }
  return cached;
}

export function refreshCache() {
  cached = null;
}

export async function getAccount() {
  const data = await getUsageData();
  return data.account;
}

export async function getQuota() {
  const data = await getUsageData();
  return data.quota;
}

export async function getTokens() {
  const data = await getUsageData();
  return data.tokens;
}

export async function getModels() {
  const data = await getUsageData();
  return data.models;
}
