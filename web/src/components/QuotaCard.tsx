import { t } from '../tokens';
import { ProgressBar } from './ProgressBar';

interface Props { title: string; percentage: number; resetTime?: string }

export function QuotaCard({ title, percentage, resetTime }: Props) {
  const percentColor = percentage < 50 ? t.statusOk : percentage <= 80 ? t.statusWarning : t.statusDanger;
  return (
    <div style={{ background: t.surfaceAlt, border: `1px solid ${t.surfaceBorder}`, borderRadius: 12, padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
      <span style={{ fontSize: 13, fontWeight: 600, color: t.textSecondary }}>{title}</span>
      <span style={{ fontSize: 28, fontWeight: 700, color: percentColor }}>{percentage}%</span>
      <ProgressBar percentage={percentage} />
      {resetTime && <span style={{ fontSize: 12, color: t.textTertiary }}>重置：{resetTime}</span>}
    </div>
  );
}
