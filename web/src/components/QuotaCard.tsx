import { t } from '../tokens';
import { ProgressBar } from './ProgressBar';

interface Props { title: string; percentage: number; resetTime?: string }

export function QuotaCard({ title, percentage, resetTime }: Props) {
  const percentColor = percentage < 50 ? t.statusOk : percentage <= 80 ? t.statusWarning : t.statusDanger;
  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 12, padding: 16 }}>
      <span style={{ fontSize: t.fsSecondary, fontWeight: 600, color: t.textSecondary }}>{title}</span>
      <span style={{ fontSize: t.fsHero, fontWeight: 700, color: percentColor }}>{percentage}%</span>
      <ProgressBar percentage={percentage} />
      {resetTime && <span style={{ fontSize: t.fsWeak, color: t.textTertiary }}>重置：{resetTime}</span>}
    </div>
  );
}
