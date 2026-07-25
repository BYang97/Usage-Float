import { useState } from 'react';
import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void; active: string }

const items = [
  { id: 'dashboard', label: '首页' },
  { id: 'history', label: '使用记录' },
  { id: 'models', label: '模型统计' },
  { id: 'settings', label: '设置' },
];

export function Sidebar({ active, onNavigate }: Props) {
  const [collapsed, setCollapsed] = useState(false);
  const sidebarWidth = collapsed ? 64 : 240;

  return (
    <div
      className="glass"
      style={{
        width: sidebarWidth,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        flexShrink: 0,
        borderRadius: 0,
        borderTop: 'none',
        borderBottom: 'none',
        borderLeft: 'none',
        transition: 'width 0.2s ease',
        overflow: 'hidden',
      }}
    >
      {/* Logo / Brand */}
      <div
        style={{
          height: 56,
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          paddingLeft: collapsed ? 0 : 16,
          paddingRight: collapsed ? 0 : 16,
          justifyContent: collapsed ? 'center' : 'flex-start',
        }}
      >
        {collapsed ? (
          <div style={{ width: 20, height: 20, borderRadius: 5, background: t.accentBlue, flexShrink: 0 }} />
        ) : (
          <>
            <div style={{ width: 20, height: 20, borderRadius: 5, background: t.accentBlue, flexShrink: 0 }} />
            <span style={{ fontSize: t.fsBody, fontWeight: 600, color: t.textPrimary, whiteSpace: 'nowrap' }}>
              OpenCode Usage Float
            </span>
          </>
        )}
      </div>

      <div style={{ marginLeft: 16, marginRight: 16, height: 1, background: t.surfaceBorder }} />

      {/* Nav items */}
      <div style={{ padding: 10, display: 'flex', flexDirection: 'column', gap: 4 }}>
        {items.map(item => {
          const isActive = active === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                padding: collapsed ? '8px' : '8px 12px',
                borderRadius: 8,
                border: 'none',
                cursor: 'pointer',
                fontSize: t.fsBody,
                fontWeight: 500,
                width: '100%',
                justifyContent: collapsed ? 'center' : 'flex-start',
                background: isActive
                  ? 'linear-gradient(135deg, var(--color-accent-blue), var(--color-accent-cyan))'
                  : 'transparent',
                color: isActive ? '#ffffff' : t.textSecondary,
              }}
            >
              <div
                style={{
                  width: 16,
                  height: 16,
                  borderRadius: 3,
                  flexShrink: 0,
                  background: isActive ? '#ffffff' : t.textTertiary,
                }}
              />
              {!collapsed && <span style={{ whiteSpace: 'nowrap' }}>{item.label}</span>}
            </button>
          );
        })}
      </div>

      {/* Toggle button at bottom */}
      <div style={{ marginTop: 'auto', padding: 10 }}>
        <button
          onClick={() => setCollapsed(prev => !prev)}
          title={collapsed ? '展开侧栏' : '折叠侧栏'}
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: '100%',
            padding: '8px',
            borderRadius: 8,
            border: 'none',
            cursor: 'pointer',
            background: 'transparent',
            color: t.textTertiary,
            fontSize: 14,
            transition: 'color 0.15s',
          }}
        >
          {collapsed ? '▶' : '◀'}
        </button>
      </div>
    </div>
  );
}
