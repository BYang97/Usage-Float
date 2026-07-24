import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../tokens';
import type { Account, AccountForm } from '../types';
import { createLogger } from '../services/logger';

const log = createLogger('AccountDialog');

interface Props {
  open: boolean;
  account?: Account | null;
  onClose: () => void;
  onSaved: () => void;
}

export function AccountDialog({ open, account, onClose, onSaved }: Props) {
  const [name, setName] = useState('');
  const [workspaceId, setWorkspaceId] = useState('');
  const [notes, setNotes] = useState('');
  const cookieRef = useRef<HTMLInputElement>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    if (open) {
      setName(account?.name ?? '');
      setWorkspaceId(account?.workspace_id ?? '');
      if (cookieRef.current) cookieRef.current.value = '';
      setNotes(account?.notes ?? '');
      setError('');
    }
  }, [open, account]);

  const handleSave = async () => {
    setError('');
    setSaving(true);
    try {
      const form: AccountForm = {
        name,
        workspace_id: workspaceId,
        auth_cookie: cookieRef.current?.value ?? '',
        notes,
      };
      if (account) {
        await invoke('update_account', { id: account.id, form });
      } else {
        await invoke('create_account', { form });
      }
      log.info(account ? `update_account: ${account.id}` : 'create_account');
      onSaved();
      onClose();
    } catch (e: unknown) {
      log.error('保存账号失败:', e);
      setError(typeof e === 'string' ? e : (e as Error)?.message || '保存失败');
    } finally {
      setSaving(false);
    }
  };

  if (!open) return null;

  const inputBase = {
    width: '100%',
    boxSizing: 'border-box' as const,
    padding: '8px 12px',
    borderRadius: 6,
    border: `1px solid ${t.surfaceBorder}`,
    background: t.surface,
    color: t.textPrimary,
    fontSize: 12,
    outline: 'none',
  };

  return (
    <div
      style={{
        position: 'fixed', inset: 0, zIndex: 1000,
        background: 'rgba(0,0,0,0.6)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: t.surfaceAlt,
          borderRadius: 12,
          boxShadow: '0 16px 48px rgba(0,0,0,0.5)',
          width: 440,
          maxHeight: '90vh',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '14px 16px 14px 20px',
        }}>
          <span style={{ fontSize: 16, fontWeight: 600, color: t.textPrimary }}>
            {account ? '编辑账号' : '添加账号'}
          </span>
          <button
            onClick={onClose}
            style={{
              background: 'transparent', border: 'none',
              color: t.textTertiary, cursor: 'pointer', fontSize: 14,
            }}
          >
            &#10005;
          </button>
        </div>

        <div style={{ height: 1, background: t.surfaceBorder }} />

        {/* Body */}
        <div style={{ padding: 20, display: 'flex', flexDirection: 'column', gap: 16, overflowY: 'auto' }}>
          {/* Name */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 12, color: t.textSecondary }}>名称</label>
            <input
              value={name}
              onChange={e => { setName(e.target.value); setError(''); }}
              placeholder="默认"
              style={inputBase}
            />
          </div>

          {/* Workspace ID */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 12, color: t.textSecondary }}>Workspace ID</label>
            <input
              value={workspaceId}
              onChange={e => { setWorkspaceId(e.target.value); setError(''); }}
              placeholder="wrk_xxxxxxxx"
              style={{ ...inputBase, fontFamily: 'monospace' }}
            />
          </div>

          {/* Auth Cookie */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 12, color: t.textSecondary }}>Auth Cookie</label>
            <input
              type="text"
              ref={cookieRef}
              defaultValue=""
              onChange={e => { setError(''); }}
              placeholder="Fe26.2..."
              autoComplete="off"
              style={{
                ...inputBase,
                fontFamily: 'monospace',
                WebkitTextSecurity: 'disc',
              }}
            />
            <p style={{ margin: 0, fontSize: 11, color: t.textTertiary, lineHeight: 1.4 }}>
              登录 opencode.ai 后，从浏览器 DevTools Application 面板复制 auth cookie 值
            </p>
          </div>

          {/* Notes */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <label style={{ fontSize: 12, color: t.textSecondary }}>备注</label>
            <textarea
              value={notes}
              onChange={e => setNotes(e.target.value)}
              placeholder="可选备注"
              style={{
                ...inputBase,
                resize: 'vertical',
                minHeight: 60,
                fontFamily: 'inherit',
              }}
            />
          </div>

          {error && (
            <div style={{ fontSize: 12, color: t.statusDanger, fontWeight: 500 }}>
              {error}
            </div>
          )}
        </div>

        <div style={{ height: 1, background: t.surfaceBorder }} />

        {/* Footer */}
        <div style={{
          display: 'flex', justifyContent: 'flex-end', gap: 8,
          padding: '12px 20px',
        }}>
          <button
            onClick={onClose}
            style={{
              height: 30, padding: '0 16px', borderRadius: 6,
              border: 'none', background: t.surfaceHover,
              color: t.textSecondary, fontSize: 13, cursor: 'pointer',
            }}
          >
            取消
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            style={{
              height: 30, padding: '0 16px', borderRadius: 6,
              border: 'none', background: t.accentBlue, color: '#fff',
              fontSize: 13, fontWeight: 500, cursor: saving ? 'not-allowed' : 'pointer',
              opacity: saving ? 0.6 : 1,
            }}
          >
            {saving ? '保存中…' : '保存'}
          </button>
        </div>
      </div>
    </div>
  );
}
