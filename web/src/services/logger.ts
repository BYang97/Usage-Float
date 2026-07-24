/**
 * 简单的日志封装，用于替代直接调 console。
 *
 * 后续可在 Tauri 环境下将日志转发到 Rust 侧统一写入文件。
 * 用法:
 *   import { logger } from '../services/logger';
 *   logger.info('msg');
 *   logger.warn('msg', err);
 *   logger.error('msg', err);
 */

type LogFn = (msg: string, ...args: unknown[]) => void;

export interface Logger {
  info: LogFn;
  warn: LogFn;
  error: LogFn;
}

/** 全局默认 logger（带 [UsageFloat] 前缀） */
export const logger: Logger = {
  info: (msg, ...args) => console.info(`[UsageFloat] ${msg}`, ...args),
  warn: (msg, ...args) => console.warn(`[UsageFloat] ${msg}`, ...args),
  error: (msg, ...args) => console.error(`[UsageFloat] ${msg}`, ...args),
};

/**
 * 创建带指定 scope 前缀的 Logger，便于区分日志来源。
 *
 *   const log = createLogger('AccountTable');
 *   log.error('load failed:', err);  // → "[AccountTable] load failed: ..."
 */
export function createLogger(scope: string): Logger {
  return {
    info: (msg, ...args) => console.info(`[${scope}] ${msg}`, ...args),
    warn: (msg, ...args) => console.warn(`[${scope}] ${msg}`, ...args),
    error: (msg, ...args) => console.error(`[${scope}] ${msg}`, ...args),
  };
}
