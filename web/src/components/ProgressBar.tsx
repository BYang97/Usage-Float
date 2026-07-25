import { t } from '../tokens';

interface Props { percentage: number; height?: number; color?: string }

export function ProgressBar({ percentage, height = 8, color }: Props) {
  const barColor = color ?? (percentage < 50 ? t.statusOk : percentage <= 80 ? t.statusWarning : t.statusDanger);
  return (
    <div style={{ width: '100%', height, borderRadius: height / 2, overflow: 'hidden', background: t.surfaceBorder }}>
      <div style={{ width: `${Math.min(percentage, 100)}%`, height: '100%', borderRadius: height / 2, background: barColor, transition: 'width 0.5s ease-in-out' }} />
    </div>
  );
}
