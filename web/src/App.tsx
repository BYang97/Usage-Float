import { useState, useEffect, useCallback } from 'react';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Dashboard } from './pages/Dashboard';
import { Settings } from './pages/Settings';
import { FloatWidget } from './components/FloatWidget';
import { getUsageData, subscribe, startAutoRefresh, stopAutoRefresh } from './services/usage-service';
import type { QuotaInfo } from './types';

/** 检测是否运行在 Tauri 环境 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** 显示 Tauri 悬浮窗口（从主窗口唤醒） */
async function showFloatWindow() {
  if (!isTauri()) return;
  try {
    const floatWin = await WebviewWindow.getByLabel('float');
    if (floatWin) {
      await floatWin.show();
      await floatWin.setFocus();
    }
  } catch (err) {
    console.error('[App] showFloatWindow failed:', err);
  }
}

export default function App() {
  const [page, setPage] = useState('dashboard');
  const [floatVisible, setFloatVisible] = useState(true);
  const [quota, setQuota] = useState<QuotaInfo | null>(null);

  const loadQuota = useCallback(async () => {
    try {
      const data = await getUsageData();
      setQuota(data.quota);
    } catch {
      // 失败时保留上次数据，Dashboard 内部会处理错误状态
    }
  }, []);

  useEffect(() => {
    // 首次加载
    loadQuota();

    // 订阅数据变更
    const unsub = subscribe((data) => {
      setQuota(data.quota);
    });

    // 启动自动刷新（5 分钟）
    startAutoRefresh(5 * 60 * 1000);

    return () => {
      unsub();
      stopAutoRefresh();
    };
  }, [loadQuota]);

  return (
    <div style={{ width: '100%', height: '100%', background: '#1a1b1e' }}>
      {page === 'dashboard' && (
        <Dashboard
          onNavigate={p => setPage(p)}
          onMinimize={() => {
            setFloatVisible(true);
            showFloatWindow();
          }}
          onClose={() => window.close()}
        />
      )}
      {page === 'settings' && <Settings onNavigate={p => setPage(p)} />}
      {(page === 'history' || page === 'models') && <Dashboard onNavigate={p => setPage(p)} />}

      {floatVisible && quota && (
        <FloatWidget
          percentage={quota.fiveHourPercent}
          resetTime={quota.fiveHourReset}
          onOpenDashboard={() => { setPage('dashboard'); setFloatVisible(false); }}
          onClose={() => setFloatVisible(false)}
        />
      )}
    </div>
  );
}
