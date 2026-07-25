import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from '../components/Sidebar';
import { Header } from '../components/Header';
import { TokenUsage } from '../components/TokenUsage';
import { getUsageData, subscribe } from '../services/usage-service';
import type { TokenInfo, UsageHistoryItem, Account } from '../types';
import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void; onMinimize?: () => void; onClose?: () => void }

function formatTime(ts: number): string {
  if (!ts) return '—';
  const d = new Date(ts * 1000);
  const pad = (n: number) => n.toString().padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function History({ onNavigate, onMinimize, onClose }: Props) {
  const [loadState, setLoadState] = useState<'loading' | 'loaded' | 'error'>('loading');
  const [errorMsg, setErrorMsg] = useState('');
  const [tokens, setTokens] = useState<TokenInfo | null>(null);
  const [historyItems, setHistoryItems] = useState<UsageHistoryItem[]>([]);
  const [historyLoadState, setHistoryLoadState] = useState<'loading' | 'loaded' | 'error'>('loading');
  const [historyError, setHistoryError] = useState('');
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [selectedAccountId, setSelectedAccountId] = useState('');
  const cursorRef = useRef(0);
  const [hasMore, setHasMore] = useState(true);

  const loadData = useCallback(async () => {
    try {
      setLoadState('loading');
      setErrorMsg('');
      const data = await getUsageData();
      setTokens(data.tokens);
      setLoadState('loaded');
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : '加载使用记录失败');
      setLoadState('error');
    }
  }, []);

  const loadHistory = useCallback(async (accountId?: string, append?: boolean) => {
    try {
      setHistoryLoadState('loading');
      setHistoryError('');
      let accs = accounts;
      if (accs.length === 0) {
        accs = await invoke<Account[]>('list_accounts');
        setAccounts(accs);
      }
      const accId = accountId ?? selectedAccountId ?? accs[0]?.id;
      if (!accId) {
        setHistoryItems([]);
        setHistoryLoadState('loaded');
        return;
      }
      const isSwitch = accountId != null && accountId !== selectedAccountId;
      if (isSwitch) {
        setSelectedAccountId(accountId);
        cursorRef.current = 0;
        setHasMore(true);
      }
      const effectiveCursor = (append && !isSwitch) ? cursorRef.current : 0;
      const items = await invoke<UsageHistoryItem[]>('get_usage_history', {
        accountId: accId,
        cursor: effectiveCursor,
      });
      if (append && !isSwitch) {
        setHistoryItems(prev => [...prev, ...items]);
      } else {
        setHistoryItems(items);
      }
      cursorRef.current = effectiveCursor + 50;
      setHasMore(items.length >= 50);
      setHistoryLoadState('loaded');
    } catch (err) {
      setHistoryError(err instanceof Error ? err.message : '加载用量历史失败');
      setHistoryLoadState('error');
    }
  }, [accounts, selectedAccountId]);

  useEffect(() => {
    let cancelled = false;
    loadData();
    loadHistory();
    const unsub = subscribe((data) => {
      if (cancelled) return;
      setTokens(data.tokens);
      setErrorMsg('');
      setLoadState('loaded');
    });
    return () => { cancelled = true; unsub(); };
  }, [loadData, loadHistory]);

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="history" onNavigate={onNavigate} />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        <Header onSettings={() => onNavigate('settings')} onMinimize={onMinimize} onClose={onClose} />

        {loadState === 'loading' && (
          <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 12 }}>
            <div style={{ width: 24, height: 24, border: '2px solid rgba(255,255,255,0.08)', borderTopColor: t.accentBlue, borderRadius: '50%', animation: 'spin 0.8s linear infinite' }} />
            <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
            <span style={{ fontSize: 13, color: t.textTertiary }}>加载中…</span>
          </div>
        )}

        {loadState === 'error' && (
          <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 16, padding: 40 }}>
            <div style={{ width: 44, height: 44, borderRadius: '50%', background: t.surfaceHover, display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 20 }}>&#9888;</div>
            <span style={{ fontSize: 15, fontWeight: 600, color: t.textPrimary }}>数据加载失败</span>
            <span style={{ fontSize: 12, color: t.textTertiary, textAlign: 'center', maxWidth: 360, lineHeight: 1.6 }}>
              {errorMsg || '无法获取使用记录数据'}
            </span>
            <button
              onClick={() => loadData()}
              style={{
                marginTop: 4, padding: '8px 20px', borderRadius: 6,
                border: 'none', background: t.accentBlue, color: '#fff',
                fontSize: 13, fontWeight: 500, cursor: 'pointer',
              }}
            >
              重试
            </button>
          </div>
        )}

        {loadState === 'loaded' && !tokens && (
          <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 12 }}>
            <span style={{ fontSize: 13, color: t.textTertiary }}>暂无使用记录</span>
          </div>
        )}

        {loadState === 'loaded' && tokens && (
          <div style={{ flex: 1, overflowY: 'auto', padding: 24, display: 'flex', flexDirection: 'column', gap: 20 }}>
            <TokenUsage data={tokens.tokenHistory} today={tokens.tokenToday} week={tokens.token7d} month={tokens.token30d} />

            {/* ── 用量历史明细 ── */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12 }}>
                <div style={{ fontSize: 13, fontWeight: 600, color: t.textSecondary }}>用量历史</div>
                {accounts.length > 0 && (
                  <select
                    value={selectedAccountId}
                    onChange={e => loadHistory(e.target.value)}
                    style={{ background: t.surface, color: t.textPrimary, border: `1px solid ${t.surfaceBorder}`, borderRadius: 4, padding: '4px 8px', fontSize: 12 }}
                  >
                    {accounts.map(a => <option key={a.id} value={a.id}>{a.name || a.workspace_id}</option>)}
                  </select>
                )}
              </div>

              {historyLoadState === 'loading' && (
                <span style={{ fontSize: 12, color: t.textTertiary }}>加载中…</span>
              )}
              {historyLoadState === 'error' && (
                <span style={{ fontSize: 12, color: t.textTertiary }}>{historyError || '加载失败'}</span>
              )}
              {historyLoadState === 'loaded' && historyItems.length === 0 && (
                <span style={{ fontSize: 12, color: t.textTertiary }}>暂无用量历史</span>
              )}
              {historyLoadState === 'loaded' && historyItems.length > 0 && (
                <div style={{ overflowX: 'auto' }}>
                  <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
                    <thead>
                      <tr style={{ color: t.textTertiary, borderBottom: `1px solid ${t.surfaceBorder}` }}>
                        <th style={{ textAlign: 'left', padding: '6px 8px', fontWeight: 500 }}>时间</th>
                        <th style={{ textAlign: 'left', padding: '6px 8px', fontWeight: 500 }}>模型</th>
                        <th style={{ textAlign: 'left', padding: '6px 8px', fontWeight: 500 }}>Provider</th>
                        <th style={{ textAlign: 'right', padding: '6px 8px', fontWeight: 500 }}>Input</th>
                        <th style={{ textAlign: 'right', padding: '6px 8px', fontWeight: 500 }}>Output</th>
                        <th style={{ textAlign: 'right', padding: '6px 8px', fontWeight: 500 }}>Reasoning</th>
                        <th style={{ textAlign: 'right', padding: '6px 8px', fontWeight: 500 }}>Cache</th>
                        <th style={{ textAlign: 'right', padding: '6px 8px', fontWeight: 500 }}>费用</th>
                      </tr>
                    </thead>
                    <tbody>
                      {historyItems.map((item) => (
                        <tr key={item.id} style={{ borderBottom: `1px solid ${t.surfaceBorder}`, color: t.textPrimary }}>
                          <td style={{ padding: '6px 8px', whiteSpace: 'nowrap', color: t.textTertiary }}>
                            {formatTime(item.time_created)}
                          </td>
                          <td style={{ padding: '6px 8px', maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                            {item.model}
                          </td>
                          <td style={{ padding: '6px 8px' }}>{item.provider}</td>
                          <td style={{ padding: '6px 8px', textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                            {item.input_tokens.toLocaleString()}
                          </td>
                          <td style={{ padding: '6px 8px', textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                            {item.output_tokens.toLocaleString()}
                          </td>
                          <td style={{ padding: '6px 8px', textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                            {item.reasoning_tokens > 0 ? item.reasoning_tokens.toLocaleString() : '—'}
                          </td>
                          <td style={{ padding: '6px 8px', textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                            {item.cache_read_tokens > 0 ? item.cache_read_tokens.toLocaleString() : '—'}
                          </td>
                          <td style={{ padding: '6px 8px', textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                            {item.cost.toFixed(6)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                {hasMore && (
                  <div style={{ display: 'flex', justifyContent: 'center', padding: '12px 0' }}>
                    <button
                      onClick={() => loadHistory(undefined, true)}
                      disabled={historyLoadState === 'loading'}
                      style={{
                        padding: '6px 20px', borderRadius: 6,
                        border: `1px solid ${t.surfaceBorder}`, background: t.surface,
                        color: t.textPrimary, fontSize: 12, cursor: 'pointer',
                        opacity: historyLoadState === 'loading' ? 0.5 : 1,
                      }}
                    >
                      {historyLoadState === 'loading' ? '加载中…' : '加载更多'}
                    </button>
                  </div>
                )}
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
