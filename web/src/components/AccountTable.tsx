import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../tokens';
import { AccountDialog } from './AccountDialog';
import { QuotaCard } from './QuotaCard';
import type { Account, UsageResult, AccountWithUsage } from '../types';
import { createLogger } from '../services/logger';

const log = createLogger('AccountTable');

export function AccountTable() {
  const [items, setItems] = useState<AccountWithUsage[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState<Set<string>>(new Set());
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editAccount, setEditAccount] = useState<Account | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const list = await invoke<Account[]>('list_accounts');
      const mapped: AccountWithUsage[] = list.map(a => ({ account: a, usage: null }));
      setItems(mapped);
      // kick off usage refresh for each account
      for (const a of list) {
        refreshOneUsage(a.id);
      }
    } catch (err) {
      log.error('list_accounts failed:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const refreshOneUsage = async (accountId: string) => {
    setRefreshing(prev => new Set(prev).add(accountId));
    try {
      const usage = await invoke<UsageResult>('refresh_one', { accountId });
      setItems(prev => prev.map(i =>
        i.account.id === accountId ? { ...i, usage } : i
      ));
    } catch (err) {
      log.error(`refresh_one ${accountId} failed:`, err);
    } finally {
      setRefreshing(prev => {
        const next = new Set(prev);
        next.delete(accountId);
        return next;
      });
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除此账号？删除后无法恢复。')) return;
    try {
      await invoke('delete_account', { id });
      setItems(prev => prev.filter(i => i.account.id !== id));
    } catch (err) {
      log.error('delete_account failed:', err);
    }
  };

  const openAdd = () => {
    setEditAccount(null);
    setDialogOpen(true);
  };

  const openEdit = (account: Account) => {
    setEditAccount(account);
    setDialogOpen(true);
  };

  const handleSaved = () => {
    setDialogOpen(false);
    load();
  };

  // ── Styles ──────────────────────────────────────────
  const btnBase: React.CSSProperties = {
    height: 28,
    padding: '0 12px',
    borderRadius: 6,
    border: 'none',
    fontSize: 12,
    fontWeight: 500,
    cursor: 'pointer',
    display: 'inline-flex',
    alignItems: 'center',
    gap: 4,
  };

  // ── Loading ─────────────────────────────────────────
  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: 32 }}>
        <span style={{ fontSize: 13, color: t.textTertiary }}>加载中…</span>
      </div>
    );
  }

  // ── Empty state ─────────────────────────────────────
  if (items.length === 0) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12, padding: 40 }}>
        <span style={{ fontSize: 14, color: t.textSecondary }}>还没有添加账号</span>
        <p style={{ margin: 0, fontSize: 12, color: t.textTertiary, lineHeight: 1.5, textAlign: 'center' }}>
          添加 OpenCode Go 账号后即可查看配额和使用情况
        </p>
        <button onClick={openAdd} style={{ ...btnBase, background: t.accentBlue, color: '#fff', height: 32, padding: '0 20px' }}>
          + 添加账号
        </button>
        <AccountDialog open={dialogOpen} account={editAccount} onClose={() => setDialogOpen(false)} onSaved={handleSaved} />
      </div>
    );
  }

  // ── Account list ────────────────────────────────────
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ fontSize: 13, fontWeight: 600, color: t.textPrimary }}>账号管理（{items.length}）</span>
        <button onClick={openAdd} style={{ ...btnBase, background: t.accentBlue, color: '#fff' }}>
          + 添加账号
        </button>
      </div>

      {/* Cards */}
      {items.map(({ account, usage }) => {
        const isRefreshing = refreshing.has(account.id);
        return (
          <div
            key={account.id}
            style={{
              background: t.surfaceAlt,
              border: `1px solid ${t.surfaceBorder}`,
              borderRadius: 8,
              padding: 16,
              display: 'flex',
              flexDirection: 'column',
              gap: 12,
            }}
          >
            {/* Row 1: name + workspaceId + plan */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <span style={{ fontSize: 15, fontWeight: 600, color: t.textPrimary }}>
                  {account.name}
                </span>
                <code style={{ fontSize: 11, color: t.textTertiary }}>
                  {account.workspace_id}
                </code>
              </div>
              {usage && (
                <div style={{
                  display: 'flex', alignItems: 'center', gap: 6,
                  padding: '2px 8px', borderRadius: 4,
                  background: usage.status === 'active' ? 'rgba(52,211,153,0.1)' : 'rgba(251,191,36,0.1)',
                }}>
                  <span style={{
                    width: 6, height: 6, borderRadius: '50%',
                    background: usage.status === 'active' ? t.statusOk : t.statusWarning,
                  }} />
                  <span style={{ fontSize: 11, fontWeight: 500, color: t.textSecondary }}>
                    {usage.plan}
                  </span>
                </div>
              )}
            </div>

            {/* Row 2: quota cards */}
            {usage ? (
              <div style={{ display: 'flex', gap: 12 }}>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <QuotaCard title="滚动配额" percentage={usage.fiveHourPercent} resetTime={usage.fiveHourReset} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <QuotaCard title="周配额" percentage={usage.weeklyPercent} resetTime={usage.weeklyReset} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <QuotaCard title="月配额" percentage={usage.monthlyPercent} />
                </div>
              </div>
            ) : (
              <div style={{ fontSize: 12, color: t.textTertiary, padding: '8px 0' }}>
                {isRefreshing ? '正在获取配额…' : '点击刷新获取配额数据'}
              </div>
            )}

            {/* Row 3: actions */}
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button
                onClick={() => refreshOneUsage(account.id)}
                disabled={isRefreshing}
                style={{
                  ...btnBase,
                  background: t.surfaceHover,
                  color: t.textSecondary,
                  opacity: isRefreshing ? 0.6 : 1,
                  cursor: isRefreshing ? 'not-allowed' : 'pointer',
                }}
              >
                {isRefreshing ? '⟳ 刷新中…' : '⟳ 刷新'}
              </button>
              <button
                onClick={() => openEdit(account)}
                style={{ ...btnBase, background: t.surfaceHover, color: t.textSecondary }}
              >
                编辑
              </button>
              <button
                onClick={() => handleDelete(account.id)}
                style={{ ...btnBase, background: t.surfaceHover, color: t.statusDanger }}
              >
                删除
              </button>
            </div>
          </div>
        );
      })}

      {/* Dialog */}
      <AccountDialog open={dialogOpen} account={editAccount} onClose={() => setDialogOpen(false)} onSaved={handleSaved} />
    </div>
  );
}
