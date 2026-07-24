import { useState, useEffect } from 'react';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { History } from './pages/History';
import { Models } from './pages/Models';
import { Dashboard } from './pages/Dashboard';
import { Settings } from './pages/Settings';
import { startAutoRefresh, stopAutoRefresh } from './services/usage-service';

/** 检测是否运行在 Tauri 环境 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** 显示悬浮窗口(独立 OS 窗口,可拖到桌面任意位置;由 tauri.conf.json 静态配置 visible:false) */
async function showFloatWindow() {
  if (!isTauri()) return;
  try {
    const floatWin = await WebviewWindow.getByLabel('float');
    if (floatWin) {
      await floatWin.show();
      await floatWin.setFocus();
    } else {
      console.warn('[App] float window not found');
    }
  } catch (err) {
    console.error('[App] showFloatWindow failed:', err);
  }
}

export default function App() {
  const [page, setPage] = useState('dashboard');

  useEffect(() => {
    // 启动定时刷新(5 分钟),触发 refreshAndNotify 通知 Dashboard + FloatWindow
    startAutoRefresh(5 * 60 * 1000);
    return () => stopAutoRefresh();
  }, []);

  return (
    <div style={{ width: '100%', height: '100%', background: '#1a1b1e' }}>
      {page === 'dashboard' && (
        <Dashboard
          onNavigate={p => setPage(p)}
          onMinimize={() => { showFloatWindow(); }}
          onClose={() => window.close()}
        />
      )}
      {page === 'settings' && <Settings onNavigate={p => setPage(p)} />}
      {page === 'history' && <History onNavigate={p => setPage(p)} onMinimize={() => { showFloatWindow(); }} onClose={() => window.close()} />}
      {page === 'models' && <Models onNavigate={p => setPage(p)} onMinimize={() => { showFloatWindow(); }} onClose={() => window.close()} />}

      {/* 悬浮窗为独立 OS 窗口(label=float, tauri.conf.json 配置无边框+透明+置顶+skipTaskbar),
          不再在主窗口内叠加;由 tray 或 showFloatWindow 控制 show/hide */}
    </div>
  );
}
