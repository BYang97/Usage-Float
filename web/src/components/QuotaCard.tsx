import { useEffect, useState } from 'react';
import { t } from '../tokens';
import { ProgressBar } from './ProgressBar';

interface Props { title: string; percentage: number; resetTime?: string }

export function QuotaCard({ title, percentage, resetTime }: Props) {
  const [displayValue, setDisplayValue] = useState(0);
  const percentColor = percentage < 50 ? t.statusOk : percentage <= 80 ? t.statusWarning : t.statusDanger;

  useEffect(() => {
    let start: number | null = null;
    const duration = 1000;
    const from = 0;
    const to = percentage;
    if (from === to) { setDisplayValue(to); return; }

    function step(ts: number) {
      if (!start) start = ts;
      const elapsed = ts - start;
      const progress = Math.min(elapsed / duration, 1);
      // ease-out quad
      const eased = 1 - (1 - progress) * (1 - progress);
      setDisplayValue(Math.round(from + (to - from) * eased));
      if (progress < 1) requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
  }, [percentage]);

  return (
    <div
      className="card"
      style={{
        display: 'flex', flexDirection: 'column', gap: 12, padding: 16,
      }}
    >
      <span style={{ fontSize: t.fsSecondary, fontWeight: 600, color: t.textSecondary }}>
        {title}
      </span>
      <span
        style={{
          fontSize: 36, fontWeight: 700, color: percentColor,
          fontVariantNumeric: 'tabular-nums',
        }}
      >
        {displayValue}%
      </span>
      <ProgressBar
        percentage={percentage}
        color={`linear-gradient(135deg, ${t.accentBlue}, ${t.accentCyan})`}
      />
      {resetTime && (
        <span style={{ fontSize: t.fsWeak, color: t.textTertiary }}>
          重置：{resetTime}
        </span>
      )}
    </div>
  );
}
