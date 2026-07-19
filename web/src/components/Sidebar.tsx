import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void; active: string }

const items = [
  { id: 'dashboard', label: 'Dashboard' },
  { id: 'history', label: 'Usage History' },
  { id: 'models', label: 'Models' },
  { id: 'settings', label: 'Settings' },
];

export function Sidebar({ active, onNavigate }: Props) {
  return (
    <div style={{ width: 240, height: '100%', background: t.surfaceAlt, display: 'flex', flexDirection: 'column', flexShrink: 0 }}>
      <div style={{ height: 52, display: 'flex', alignItems: 'center', gap: 10, paddingLeft: 16, paddingRight: 16 }}>
        <div style={{ width: 20, height: 20, borderRadius: 5, background: t.accentBlue }} />
        <span style={{ fontSize: 13, fontWeight: 600, color: t.textPrimary }}>OpenCode Usage Float</span>
      </div>
      <div style={{ marginLeft: 16, marginRight: 16, height: 1, background: t.surfaceBorder }} />
      <div style={{ padding: 10, display: 'flex', flexDirection: 'column', gap: 4 }}>
        {items.map(item => {
          const isActive = active === item.id;
          return (
            <button key={item.id} onClick={() => onNavigate(item.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 10, padding: '8px 12px', borderRadius: 6, border: 'none', cursor: 'pointer',
                fontSize: 14, fontWeight: 500, width: '100%',
                background: isActive ? t.surfaceHover : 'transparent',
                color: isActive ? t.accentBlue : t.textSecondary,
              }}>
              <div style={{ width: 16, height: 16, borderRadius: 3, background: isActive ? t.accentBlue : t.textTertiary }} />
              <span>{item.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
