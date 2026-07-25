import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void; active: string }

const items = [
  { id: 'dashboard', label: '首页' },
  { id: 'history', label: '使用记录' },
  { id: 'models', label: '模型统计' },
  { id: 'settings', label: '设置' },
];

export function Sidebar({ active, onNavigate }: Props) {
  return (
    <div style={{ width: 240, height: '100%', background: t.surfaceAlt, display: 'flex', flexDirection: 'column', flexShrink: 0 }}>
      <div style={{ height: 56, display: 'flex', alignItems: 'center', gap: 10, paddingLeft: 16, paddingRight: 16 }}>
        <div style={{ width: 20, height: 20, borderRadius: 5, background: t.accentBlue }} />
        <span style={{ fontSize: t.fsBody, fontWeight: 600, color: t.textPrimary }}>OpenCode Usage Float</span>
      </div>
      <div style={{ marginLeft: 16, marginRight: 16, height: 1, background: t.surfaceBorder }} />
      <div style={{ padding: 10, display: 'flex', flexDirection: 'column', gap: 4 }}>
        {items.map(item => {
          const isActive = active === item.id;
          return (
            <button key={item.id} onClick={() => onNavigate(item.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 10, padding: '8px 12px', borderRadius: 8, border: 'none', cursor: 'pointer',
                fontSize: t.fsBody, fontWeight: 500, width: '100%',
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
