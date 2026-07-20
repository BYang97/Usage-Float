import { isTauri } from '@tauri-apps/api/core';
import type { UsageProvider, UsageData } from '../types';
import { MockProvider } from '../providers/mock-provider';
import { TauriProvider } from '../providers/tauri-provider';

// Tauri 运行时内走 Rust 命令;纯 web 预览回落 mock。
let provider: UsageProvider = isTauri() ? new TauriProvider() : new MockProvider();

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
