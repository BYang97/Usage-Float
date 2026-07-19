import { t } from '../tokens';

type PlanStatus = 'active' | 'expired' | 'error';
interface Props { plan: string; status: PlanStatus; expireDate: string }

export function PlanCard({ plan, status, expireDate }: Props) {
  const dotColor = status === 'active' ? t.statusOk : t.statusWarning;
  return (
    <div style={{ background: t.surfaceAlt, border: `1px solid ${t.surfaceBorder}`, borderRadius: 8, padding: 16, display: 'flex', alignItems: 'center', gap: 40 }}>
      <div style={{ width: 48, height: 48, borderRadius: 8, background: t.accentBlue, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
        <div style={{ width: 24, height: 24, borderRadius: 4, background: t.accentCyan }} />
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <span style={{ fontSize: 16, fontWeight: 600, color: t.textPrimary }}>{plan}</span>
        <span style={{ fontSize: 13, color: t.textSecondary }}>OpenCode Go - 正常</span>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <span style={{ fontSize: 11, color: t.textTertiary }}>到期时间</span>
        <span style={{ fontSize: 14, fontWeight: 500, color: t.textPrimary }}>{expireDate}</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '4px 8px', borderRadius: 4, background: 'rgba(52,211,153,0.1)' }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: dotColor }} />
        <span style={{ fontSize: 11, fontWeight: 500, color: t.statusOk, textTransform: 'capitalize' }}>{status}</span>
      </div>
    </div>
  );
}
