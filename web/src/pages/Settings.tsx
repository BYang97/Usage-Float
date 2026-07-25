import { t } from '../tokens';
import { PageLayout } from '../components/PageLayout';
import { AccountTable } from '../components/AccountTable';

interface Props { onNavigate: (page: string) => void }

export function Settings({ onNavigate }: Props) {
  const disabledToggle = { width: 36, height: 20, borderRadius: 10, background: t.surfaceBorder, position: 'relative', flexShrink: 0, cursor: 'not-allowed', opacity: 0.4 } as const;
  const disabledKnob = { position: 'absolute', width: 16, height: 16, borderRadius: '50%', background: '#fff', top: 2, left: 2 } as const;
  const chipBase = { height: 24, paddingLeft: 8, paddingRight: 8, borderRadius: 4, display: 'flex', alignItems: 'center', fontSize: 12, fontWeight: 500, cursor: 'not-allowed', opacity: 0.4 } as const;

  return (
    <PageLayout active="settings" title="设置" onNavigate={onNavigate}>
      <div className="card" style={{
        width: 480, alignSelf: 'center',
      }}>
        {/* General */}
        <Section title="通用设置">
          <Row label="开机自动启动">
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={disabledToggle}><div style={disabledKnob} /></div>
              <span style={{ fontSize: t.fsWeak, color: t.textTertiary, background: t.surfaceHover, padding: '2px 6px', borderRadius: 3 }}>即将支持</span>
            </div>
          </Row>
          <Row label="刷新频率">
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              <div style={{ display: 'flex', gap: 12 }}>
                <div style={{ ...chipBase, background: t.surfaceHover, color: t.textSecondary }}>5分钟</div>
                <div style={{ ...chipBase, background: t.surfaceHover, color: t.textSecondary }}>30分钟</div>
                <div style={{ ...chipBase, background: t.surfaceHover, color: t.textSecondary }}>60分钟</div>
              </div>
              <span style={{ fontSize: t.fsWeak, color: t.textTertiary, background: t.surfaceHover, padding: '2px 6px', borderRadius: 3 }}>即将支持</span>
            </div>
          </Row>
        </Section>

        <div style={{ height: 1, background: t.surfaceBorder }} />

        {/* Display */}
        <Section title="外观设置">
          <Row label="悬浮球">
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={disabledToggle}><div style={disabledKnob} /></div>
              <span style={{ fontSize: t.fsWeak, color: t.textTertiary, background: t.surfaceHover, padding: '2px 6px', borderRadius: 3 }}>即将支持</span>
            </div>
          </Row>
          <Row label="主题">
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 4, height: 28, paddingLeft: 10, paddingRight: 10, borderRadius: 4, background: t.surfaceHover, cursor: 'not-allowed', opacity: 0.4 }}>
                <span style={{ fontSize: t.fsSecondary, fontWeight: 500, color: t.textPrimary }}>深色模式</span>
                <span style={{ color: t.textTertiary, fontSize: 14 }}>&gt;</span>
              </div>
              <span style={{ fontSize: t.fsWeak, color: t.textTertiary, background: t.surfaceHover, padding: '2px 6px', borderRadius: 3 }}>即将支持</span>
            </div>
          </Row>
        </Section>

        <div style={{ height: 1, background: t.surfaceBorder }} />

        {/* Privacy */}
        <Section title="隐私设置">
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <div style={{ width: 14, height: 16, borderRadius: 2, background: t.textTertiary }} />
            <span style={{ fontSize: t.fsSecondary, color: t.textSecondary }}>数据仅保存在本机 · 不上传用户数据</span>
          </div>
        </Section>

        <div style={{ height: 1, background: t.surfaceBorder }} />

        {/* Accounts */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12, padding: 16 }}>
          <AccountTable />
        </div>
      </div>
    </PageLayout>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingLeft: 20, paddingRight: 20, paddingTop: 16, paddingBottom: 16 }}>
      <span style={{ fontSize: t.fsSecondary, fontWeight: 600, color: t.textSecondary }}>{title}</span>
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
