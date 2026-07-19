import { t } from '../tokens';
import { Sidebar } from '../components/Sidebar';

interface Props { onNavigate: (page: string) => void }

export function Settings({ onNavigate }: Props) {
  const toggle = { width: 36, height: 20, borderRadius: 10, background: t.accentBlue, position: 'relative', flexShrink: 0, cursor: 'pointer' } as const;
  const knob = { position: 'absolute', width: 16, height: 16, borderRadius: '50%', background: '#fff', top: 2, right: 2 } as const;
  const chipBase = { height: 24, paddingLeft: 8, paddingRight: 8, borderRadius: 4, display: 'flex', alignItems: 'center', fontSize: 12, fontWeight: 500, cursor: 'pointer' } as const;

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="settings" onNavigate={onNavigate} />
      <div style={{ flex: 1, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={{ background: t.surfaceAlt, borderRadius: 12, boxShadow: '0 16px 48px rgba(0,0,0,0.5)', width: 480, overflow: 'hidden' }}>
          {/* Title */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingLeft: 20, paddingRight: 16, paddingTop: 14, paddingBottom: 14 }}>
            <span style={{ fontSize: 16, fontWeight: 600, color: t.textPrimary }}>设置</span>
            <button onClick={() => onNavigate('dashboard')} style={{ background: 'transparent', border: 'none', color: t.textTertiary, cursor: 'pointer', fontSize: 14 }}>&#10005;</button>
          </div>
          <div style={{ height: 1, background: t.surfaceBorder }} />

          {/* General */}
          <Section title="通用设置">
            <Row label="开机自动启动">
              <div style={toggle}><div style={knob} /></div>
            </Row>
            <Row label="刷新频率">
              <div style={{ display: 'flex', gap: 12 }}>
                <div style={{ ...chipBase, background: t.accentBlue, color: '#fff' }}>5分钟</div>
                <div style={{ ...chipBase, background: t.surfaceHover, color: t.textSecondary }}>30分钟</div>
                <div style={{ ...chipBase, background: t.surfaceHover, color: t.textSecondary }}>60分钟</div>
              </div>
            </Row>
          </Section>

          <div style={{ height: 1, background: t.surfaceBorder }} />

          {/* Display */}
          <Section title="外观设置">
            <Row label="悬浮球">
              <div style={toggle}><div style={knob} /></div>
            </Row>
            <Row label="主题">
              <div style={{ display: 'flex', alignItems: 'center', gap: 4, height: 28, paddingLeft: 10, paddingRight: 10, borderRadius: 4, background: t.surfaceHover, cursor: 'pointer' }}>
                <span style={{ fontSize: 12, fontWeight: 500, color: t.textPrimary }}>深色模式</span>
                <span style={{ color: t.textTertiary, fontSize: 14 }}>&gt;</span>
              </div>
            </Row>
          </Section>

          <div style={{ height: 1, background: t.surfaceBorder }} />

          {/* Privacy */}
          <Section title="隐私设置">
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={{ width: 14, height: 16, borderRadius: 2, background: t.textTertiary }} />
              <span style={{ fontSize: 12, color: t.textSecondary }}>数据仅保存在本机 · 不上传用户数据</span>
            </div>
          </Section>
        </div>
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingLeft: 20, paddingRight: 20, paddingTop: 16, paddingBottom: 16 }}>
      <span style={{ fontSize: 13, fontWeight: 600, color: t.textSecondary }}>{title}</span>
      {children}
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', height: 32 }}>
      <span style={{ fontSize: 13, color: t.textPrimary }}>{label}</span>
      {children}
    </div>
  );
}
