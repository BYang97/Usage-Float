import { t } from '../tokens';

interface Props { percentage: number; size?: number; strokeWidth?: number }

export function QuotaRing({ percentage, size = 72, strokeWidth = 6 }: Props) {
  const r = (size - strokeWidth) / 2;
  const c = 2 * Math.PI * r;
  const offset = c - (percentage / 100) * c;
  const color = percentage >= 50 ? t.statusOk : percentage >= 20 ? t.statusWarning : t.statusDanger;

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <circle cx={size/2} cy={size/2} r={r} fill="none" stroke={t.surfaceBorder} strokeWidth={strokeWidth} />
      <circle cx={size/2} cy={size/2} r={r} fill="none" stroke={color} strokeWidth={strokeWidth}
        strokeDasharray={c} strokeDashoffset={offset} strokeLinecap="round"
        transform={`rotate(-90 ${size/2} ${size/2})`}
        style={{ transition: 'stroke-dashoffset 0.6s ease-in-out' }} />
      <text x={size/2} y={size/2} textAnchor="middle" dominantBaseline="central"
        fill={t.textPrimary} fontSize={18} fontWeight="700" fontFamily="Inter, sans-serif">
        {percentage}%
      </text>
    </svg>
  );
}
