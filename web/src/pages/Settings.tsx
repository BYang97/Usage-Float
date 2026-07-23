import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../tokens';
import { Sidebar } from '../components/Sidebar';

interface Props { onNavigate: (page: string) => void }

export function Settings({ onNavigate }: Props) {
  const [workspaceId, setWorkspaceId] = useState('');
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState('');
  const cookieRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // 尝试加载已保存的 cookie + workspace_id
    invoke<string>('get_opencode_cookie')
      .then(val => { if (val && cookieRef.current) cookieRef.current.value = val; })
      .catch(() => { /* ignore, cookie not set */ });
    invoke<string>('get_opencode_workspace_id')
      .then(val => { if (val) setWorkspaceId(val); })
      .catch(() => { /* ignore, workspace_id not set */ });
  }, []);

  const handleSave = async () => {
    setSaved(false);
    setError('');
    try {
      const cookieVal = cookieRef.current?.value ?? '';
      await invoke('set_opencode_cookie', { cookie: cookieVal });
      await invoke('set_opencode_workspace_id', { workspaceId });
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (e: any) {
      setError(typeof e === 'string' ? e : e?.message || '保存失败');
    }
  };

  const toggle = { width: 36, height: 20, borderRadius: 10, background: t.accentBlue, position: 'relative', flexShrink: 0, cursor: 'pointer' } as const;
  const knob = { position: 'absolute', width: 16, height: 16, borderRadius: '50%', background: '#fff', top: 2, right: 2 } as const;
  const chipBase = { height: 24, paddingLeft: 8, paddingRight: 8, borderRadius: 4, display: 'flex', alignItems: 'center', fontSize: 12, fontWeight: 500, cursor: 'pointer' } as const;

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="settings" onNavigate={onNavigate} />
      <div style={{ flex: 1, background: 'rgba(0,0,0,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'auto' }}>
        <div style={{ background: t.surfaceAlt, borderRadius: 12, boxShadow: '0 16px 48px rgba(0,0,0,0.5)', width: 480, overflow: 'hidden', maxHeight: '90vh', overflowY: 'auto' }}>
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

          <div style={{ height: 1, background: t.surfaceBorder }} />

          {/* OpenCode Go Auth */}
          <Section title="OpenCode Go 认证">
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              <p style={{ margin: 0, fontSize: 12, color: t.textSecondary, lineHeight: 1.5 }}>
                登录 opencode.ai 后,从浏览器 DevTools Application 面板复制 <code style={{ color: t.accentBlue }}>auth</code> cookie 值(以 Fe26. 开头),粘贴到下方。该 cookie 仅保存于本机,不会发送给任何第三方。
              </p>
              <div>
                <label style={{ fontSize: 12, color: t.textSecondary, display: 'block', marginBottom: 4 }}>Workspace ID</label>
                <input
                  value={workspaceId}
                  onChange={e => { setWorkspaceId(e.target.value); setSaved(false); }}
                  placeholder="wrk_xxxxxxxx(从 opencode.ai 工作区 URL 获取)"
                  style={{
                    width: '100%',
                    boxSizing: 'border-box',
                    padding: '8px 12px',
                    borderRadius: 6,
                    border: `1px solid ${t.surfaceBorder}`,
                    background: t.surface,
                    color: t.textPrimary,
                    fontSize: 12,
                    fontFamily: 'monospace',
                    outline: 'none',
                  }}
                />
              </div>
              <input
                type="text"
                ref={cookieRef}
                defaultValue=""
                placeholder="粘贴 Cookie 值，形如 Fe26.2..."
                autoComplete="off"
                style={{
                  width: '100%',
                  boxSizing: 'border-box',
                  padding: '8px 12px',
                  borderRadius: 6,
                  border: `1px solid ${t.surfaceBorder}`,
                  background: t.surface,
                  color: t.textPrimary,
                  fontSize: 12,
                  fontFamily: 'monospace',
                  outline: 'none',
                  WebkitTextSecurity: 'disc',
                }}
              />
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <button
                  onClick={handleSave}
                  style={{
                    height: 30,
                    paddingLeft: 16,
                    paddingRight: 16,
                    borderRadius: 6,
                    border: 'none',
                    background: t.accentBlue,
                    color: '#fff',
                    fontSize: 13,
                    fontWeight: 500,
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: 4,
                  }}>
                  保存
                </button>
                {saved && (
                  <span style={{ fontSize: 12, color: t.accentGreen, fontWeight: 500 }}>
                    已保存
                  </span>
                )}
                {error && (
                  <span style={{ fontSize: 12, color: t.statusDanger, fontWeight: 500 }}>
                    {error}
                  </span>
                )}
              </div>
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
