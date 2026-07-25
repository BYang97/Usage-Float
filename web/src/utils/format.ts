/**
 * 将重置秒数格式化为 "Xh Ym" / "Xm" / "-"。
 * 与 Rust 侧 `format_reset` 保持一致。
 */
export function formatReset(sec: number): string {
  if (sec <= 0) return '-';
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${sec}s`;
}
