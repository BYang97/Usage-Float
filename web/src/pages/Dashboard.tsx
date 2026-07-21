import { useEffect, useState, useCallback } from 'react';
import { Sidebar } from '../components/Sidebar';
import { Header } from '../components/Header';
import { PlanCard } from '../components/PlanCard';
import { QuotaCard } from '../components/QuotaCard';
import { TokenUsage } from '../components/TokenUsage';
import { ModelUsage } from '../components/ModelUsage';
import { getUsageData, subscribe, refreshAndNotify } from '../services/usage-service';
import type { AccountInfo, QuotaInfo, TokenInfo, ModelUsageData } from '../types';
import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void; onMinimize?: () => void; onClose?: () => void }

type LoadState = 'loading' | 'loaded' | 'error';

export function Dashboard({ onNavigate, onMinimize, onClose }: Props) {
  const [loadState, setLoadState] = useState<LoadState>('loading');
  const [errorMsg, setErrorMsg] = useState<string>('');
  const [account, setAccount] = useState<AccountInfo | null>(null);
  const [quota, setQuota] = useState<QuotaInfo | null>(null);
  const [tokens, setTokens] = useState<TokenInfo | null>(null);
  const [models, setModels] = useState<ModelUsageData[] | null>(null);

  // 加载数据
  const loadData = useCallback(async () => {
    try {
      setLoadState('loading');
      setErrorMsg('');
      const data = await getUsageData();
      setAccount(data.account);
      setQuota(data.quota);
      setTokens(data.tokens);
      setModels(data.models);
      setLoadState('loaded');
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : '加载仪表盘数据失败');
      setLoadState('error');
    }
  }, []);

  // 初次加载 + 订阅刷新 + 启动自动刷新
  useEffect(() => {
    let cancelled = false;

    loadData();

    const unsub = subscribe((data) => {
      if (cancelled) return;
      setAccount(data.account);
      setQuota(data.quota);
      setTokens(data.tokens);
      setModels(data.models);
      setErrorMsg('');
      setLoadState('loaded');
    });

    return () => {
      cancelled = true;
      unsub();
    };
  }, [loadData]);

  // ─── 加载状态 ─────────────────────────────────────────────────
  if (loadState === 'loading') {
    return (
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="dashboard" onNavigate={onNavigate} />
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <Header onSettings={() => onNavigate('settings')} onMinimize={onMinimize} onClose={onClose} />
          <div style={{
            flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center',
            flexDirection: 'column', gap: 12,
          }}>
            <div style={{
              width: 24, height: 24, border: '2px solid rgba(255,255,255,0.08)',
              borderTopColor: t.accentBlue, borderRadius: '50%',
              animation: 'spin 0.8s linear infinite',
            }} />
            <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
            <span style={{ fontSize: 13, color: t.textTertiary }}>加载中…</span>
          </div>
        </div>
      </div>
    );
  }

  // ─── 错误状态 ─────────────────────────────────────────────────
  if (loadState === 'error') {
    return (
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="dashboard" onNavigate={onNavigate} />
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <Header onSettings={() => onNavigate('settings')} onMinimize={onMinimize} onClose={onClose} />
          <div style={{
            flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center',
            flexDirection: 'column', gap: 16, padding: 40,
          }}>
            <div style={{
              width: 44, height: 44, borderRadius: '50%',
              background: t.surfaceHover, display: 'flex', alignItems: 'center',
              justifyContent: 'center', fontSize: 20,
            }}>&#9888;</div>
            <span style={{ fontSize: 15, fontWeight: 600, color: t.textPrimary }}>数据加载失败</span>
            <span style={{ fontSize: 12, color: t.textTertiary, textAlign: 'center', maxWidth: 360, lineHeight: 1.6 }}>
              {errorMsg || '无法获取使用数据，请检查 OpenCode 是否正常运行'}
            </span>
            <button
              onClick={() => { refreshAndNotify().catch(() => loadData()); }}
              style={{
                marginTop: 4, padding: '8px 20px', borderRadius: 6,
                border: 'none', background: t.accentBlue, color: '#fff',
                fontSize: 13, fontWeight: 500, cursor: 'pointer',
              }}
            >
              重试
            </button>
          </div>
        </div>
      </div>
    );
  }

  // ─── 空数据状态 ───────────────────────────────────────────────
  const noData = !account || !quota || !tokens || !models;
  if (noData) {
    return (
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="dashboard" onNavigate={onNavigate} />
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <Header onSettings={() => onNavigate('settings')} onMinimize={onMinimize} onClose={onClose} />
          <div style={{
            flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center',
            flexDirection: 'column', gap: 12,
          }}>
            <span style={{ fontSize: 13, color: t.textTertiary }}>暂无使用数据</span>
          </div>
        </div>
      </div>
    );
  }

  // ─── 正常展示 ─────────────────────────────────────────────────
  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="dashboard" onNavigate={onNavigate} />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        <Header onSettings={() => onNavigate('settings')} onMinimize={onMinimize} onClose={onClose} />
        <div style={{ flex: 1, overflowY: 'auto', padding: 24, display: 'flex', flexDirection: 'column', gap: 20 }}>
          <PlanCard plan={account.plan} status={account.status} expireDate={account.expireDate} />
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
            <QuotaCard title="5小时额度" percentage={quota.fiveHourPercent} resetTime={quota.fiveHourReset} />
            <QuotaCard title="本周额度" percentage={quota.weeklyPercent} resetTime={quota.weeklyReset} />
            <QuotaCard title="本月额度" percentage={quota.monthlyPercent} />
          </div>
          <div style={{ display: 'flex', gap: 16 }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <TokenUsage data={tokens.tokenHistory} today={tokens.tokenToday} week={tokens.token7d} month={tokens.token30d} />
            </div>
            <div style={{ width: 296, flexShrink: 0 }}>
              <ModelUsage models={models} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
