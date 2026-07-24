import { t } from '../tokens';

interface Props { models: { name: string; percentage: number; color: string }[] }

export function ModelUsage({ models }: Props) {
  return (
    <div style={{ background: t.surfaceAlt, border: `1px solid ${t.surfaceBorder}`, borderRadius: 8, padding: 20, display: 'flex', flexDirection: 'column', gap: 16 }}>
      <span style={{ fontSize: 14, fontWeight: 600, color: t.textPrimary }}>模型使用情况</span>
      {models.map(m => (
        <div key={m.name} style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 13, color: t.textPrimary }}>{m.name || '未知'}</span>
            <span style={{ fontSize: 13, fontWeight: 500, color: m.color }}>{m.percentage.toFixed(1)}%</span>
          </div>
          <div style={{ width: '100%', height: 8, borderRadius: 4, overflow: 'hidden', background: t.surfaceBorder }}>
            <div style={{ width: `${m.percentage}%`, height: '100%', borderRadius: 4, background: m.color, transition: 'width 0.5s ease-in-out' }} />
          </div>
        </div>
      ))}
    </div>
  );
}
