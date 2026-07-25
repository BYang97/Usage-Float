import { t } from '../tokens';

type PlanStatus = 'active' | 'expired' | 'error';
interface Props { plan: string; status: PlanStatus; expireDate: string }

export function PlanCard({ plan, status, expireDate }: Props) {
  const dotColor = status === 'active' ? t.statusOk : t.statusWarning;
  return (
    <div className="card" style={{ display: 'flex', alignItems: 'center', gap: 40, padding: 16 }}>
      <div style={{ width: 48, height: 48, borderRadius: 8, background: t.accentBlue, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
        <div style={{ width: 24, height: 24, borderRadius: 4, background: t.accentCyan }} />
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <span style={{ fontSize: t.fsH2, fontWeight: 600, color: t.textPrimary }}>{plan}</span>
      </div>
      {expireDate && expireDate !== '—' && expireDate !== '-' && (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <span style={{ fontSize: t.fsWeak, color: t.textTertiary }}>到期时间</span>
        <span style={{ fontSize: t.fsH3, fontWeight: 500, color: t.textPrimary }}>{expireDate}</span>
      </div>
      )}
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '4px 8px', borderRadius: 4, background: 'rgba(52,211,153,0.1)' }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: dotColor }} />
        <span style={{ fontSize: t.fsWeak, fontWeight: 500, color: t.statusOk }}>{status === 'active' ? '正常' : status === 'expired' ? '已过期' : '异常'}</span>
      </div>
    </div>
  );
}
