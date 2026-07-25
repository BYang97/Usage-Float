import { t } from '../tokens';

interface Props {
  label?: string;
}

export function Spinner({ label = '加载中…' }: Props) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      flexDirection: 'column', gap: 12,
    }}>
      <div style={{
        width: 24, height: 24,
        border: '2px solid rgba(255,255,255,0.08)',
        borderTopColor: t.accentBlue,
        borderRadius: '50%',
        animation: 'spin 0.8s linear infinite',
      }} />
      <span style={{ fontSize: 13, color: t.textTertiary }}>{label}</span>
    </div>
  );
}
