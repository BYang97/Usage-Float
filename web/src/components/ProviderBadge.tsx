import { t } from '../tokens';

export function ProviderBadge({ status = 'active' }: { status?: 'active' | 'expired' | 'error' }) {
  const dotColor = status === 'active' ? t.statusOk : status === 'expired' ? t.statusWarning : t.statusDanger;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
      <span style={{ width: 8, height: 8, borderRadius: '50%', background: dotColor }} />
      <span style={{ fontSize: 13, fontWeight: 600, color: t.textPrimary }}>OpenCode GO</span>
    </div>
  );
}
