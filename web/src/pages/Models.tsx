import { useEffect, useState, useCallback } from 'react';
import { Sidebar } from '../components/Sidebar';
import { Header } from '../components/Header';
import { ModelUsage } from '../components/ModelUsage';
import { getUsageData, subscribe } from '../services/usage-service';
import type { ModelUsageData } from '../types';
import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void }

export function Models({ onNavigate }: Props) {
  const [loadState, setLoadState] = useState<'loading' | 'loaded' | 'error'>('loading');
  const [errorMsg, setErrorMsg] = useState('');
  const [models, setModels] = useState<ModelUsageData[] | null>(null);

  const loadData = useCallback(async () => {
    try {
      setLoadState('loading');
      setErrorMsg('');
      const data = await getUsageData();
      setModels(data.models);
      setLoadState('loaded');
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : '加载模型统计失败');
      setLoadState('error');
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    loadData();
    const unsub = subscribe((data) => {
      if (cancelled) return;
      setModels(data.models);
      setErrorMsg('');
      setLoadState('loaded');
    });
    return () => { cancelled = true; unsub(); };
  }, [loadData]);

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="models" onNavigate={onNavigate} />
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
              {errorMsg || '无法获取模型统计数据'}
            </span>
          </div>
        )}

        {loadState === 'loaded' && !models && (
          <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 12 }}>
            <span style={{ fontSize: 13, color: t.textTertiary }}>暂无模型数据</span>
          </div>
        )}

        {loadState === 'loaded' && models && (
          <div style={{ flex: 1, overflowY: 'auto', padding: 24, display: 'flex', flexDirection: 'column', gap: 20 }}>
            <ModelUsage models={models} />
          </div>
        )}
      </div>
    </div>
  );
}
