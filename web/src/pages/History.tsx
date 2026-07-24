import { useEffect, useState, useCallback } from 'react';
import { Sidebar } from '../components/Sidebar';
import { Header } from '../components/Header';
import { TokenUsage } from '../components/TokenUsage';
import { getUsageData, subscribe } from '../services/usage-service';
import type { TokenInfo } from '../types';
import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void }

export function History({ onNavigate }: Props) {
  const [loadState, setLoadState] = useState<'loading' | 'loaded' | 'error'>('loading');
  const [errorMsg, setErrorMsg] = useState('');
  const [tokens, setTokens] = useState<TokenInfo | null>(null);

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

  useEffect(() => {
    let cancelled = false;
    loadData();
    const unsub = subscribe((data) => {
      if (cancelled) return;
      setTokens(data.tokens);
      setErrorMsg('');
      setLoadState('loaded');
    });
    return () => { cancelled = true; unsub(); };
  }, [loadData]);

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="history" onNavigate={onNavigate} />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        <Header onSettings={() => onNavigate('settings')} onMinimize={undefined} onClose={undefined} />

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
          </div>
        )}
      </div>
    </div>
  );
}
