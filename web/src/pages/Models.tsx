import { useEffect, useState, useCallback } from 'react';
import { PageLayout } from '../components/PageLayout';
import { ModelUsage } from '../components/ModelUsage';
import { getUsageData, subscribe } from '../services/usage-service';
import type { ModelUsageData } from '../types';
import { Spinner } from '../components/Spinner';
import { t } from '../tokens';

interface Props { onNavigate: (page: string) => void; onMinimize?: () => void; onClose?: () => void }

export function Models({ onNavigate, onMinimize, onClose }: Props) {
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
    <PageLayout active="models" title="模型统计" onNavigate={onNavigate} onMinimize={onMinimize} onClose={onClose}>

      {loadState === 'loading' && (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <Spinner />
        </div>
      )}

      {loadState === 'error' && (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 16, padding: 40 }}>
          <div style={{ width: 44, height: 44, borderRadius: '50%', background: t.surfaceHover, display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 20 }}>&#9888;</div>
          <span style={{ fontSize: t.fsH2, fontWeight: 600, color: t.textPrimary }}>数据加载失败</span>
          <span style={{ fontSize: t.fsSecondary, color: t.textTertiary, textAlign: 'center', maxWidth: 360, lineHeight: 1.6 }}>
            {errorMsg || '无法获取模型统计数据'}
          </span>
          <button onClick={() => loadData()}
            style={{ marginTop: 4, padding: '8px 20px', borderRadius: 6, border: 'none', background: t.accentBlue, color: '#fff', fontSize: t.fsBody, fontWeight: 500, cursor: 'pointer' }}>
            重试
          </button>
        </div>
      )}

      {loadState === 'loaded' && !models && (
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 12 }}>
          <span style={{ fontSize: t.fsBody, color: t.textTertiary }}>暂无模型数据</span>
        </div>
      )}

      {loadState === 'loaded' && models && (
        <ModelUsage models={models} />
      )}
    </PageLayout>
  );
}
