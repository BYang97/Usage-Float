import { isTauri } from '@tauri-apps/api/core';
import type { UsageProvider, UsageData } from '../types';
import { MockProvider } from '../providers/mock-provider';
import { TauriProvider } from '../providers/tauri-provider';
import { createLogger } from './logger';

const log = createLogger('usage-service');

// Tauri 运行时内走 Rust 命令;纯 web 预览回落 mock。
let provider: UsageProvider = isTauri() ? new TauriProvider() : new MockProvider();

export function setProvider(p: UsageProvider) {
  provider = p;
}

// ─── 缓存 ───────────────────────────────────────────────────────

let cached: UsageData | null = null;

export function refreshCache() {
  cached = null;
}

// ─── 订阅/通知（数据变更时通知所有消费者） ─────────────────────

type DataCallback = (data: UsageData) => void;
let listeners: DataCallback[] = [];

/** 订阅数据变更，返回取消订阅函数 */
export function subscribe(cb: DataCallback): () => void {
  listeners.push(cb);
  return () => {
    listeners = listeners.filter(l => l !== cb);
  };
}

function notifyAll(data: UsageData) {
  listeners.forEach(cb => {
    try { cb(data); } catch (e) { log.error('通知回调异常:', e); }
  });
}

// ─── 核心数据获取 ───────────────────────────────────────────────

export async function getUsageData(): Promise<UsageData> {
  if (!cached) {
    cached = await provider.getUsageData();
  }
  return cached;
}

/**
 * 强制刷新并通知所有订阅者
 * 相当于 refreshCache() + 重新获取 + notifyAll
 */
export async function refreshAndNotify(): Promise<UsageData> {
  refreshCache();
  const data = await provider.getUsageData();
  cached = data;
  notifyAll(data);
  return data;
}

// ─── 自动刷新 ───────────────────────────────────────────────────

let refreshTimer: ReturnType<typeof setInterval> | null = null;

/** 启动定时刷新（自动跳过异常，不崩溃） */
export function startAutoRefresh(intervalMs: number) {
  stopAutoRefresh();
  refreshTimer = setInterval(async () => {
    try {
      await refreshAndNotify();
    } catch (err) {
      log.error('自动刷新失败:', err);
      // 静默失败，下次重试
    }
  }, intervalMs);
}

/** 停止定时刷新 */
export function stopAutoRefresh() {
  if (refreshTimer !== null) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
}

// ─── 便捷查询方法 ───────────────────────────────────────────────

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
