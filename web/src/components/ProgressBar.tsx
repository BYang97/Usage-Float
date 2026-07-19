import { t } from '../tokens';

interface Props { percentage: number; height?: number }

export function ProgressBar({ percentage, height = 8 }: Props) {
  const color = percentage >= 50 ? t.statusOk : percentage >= 20 ? t.statusWarning : t.statusDanger;
  return (
    <div style={{ width: '100%', height, borderRadius: height/2, overflow: 'hidden', background: t.surfaceBorder }}>
      <div style={{ width: `${percentage}%`, height: '100%', borderRadius: height/2, background: color, transition: 'width 0.5s ease-in-out' }} />
    </div>
  );
}
