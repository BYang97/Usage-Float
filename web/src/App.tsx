import { useState, useEffect } from 'react';
import { Dashboard } from './pages/Dashboard';
import { Settings } from './pages/Settings';
import { FloatWidget } from './components/FloatWidget';
import { getQuota } from './services/usage-service';
import type { QuotaInfo } from './types';

export default function App() {
  const [page, setPage] = useState('dashboard');
  const [floatVisible, setFloatVisible] = useState(true);
  const [quota, setQuota] = useState<QuotaInfo | null>(null);

  useEffect(() => {
    getQuota().then(setQuota);
  }, []);

  return (
    <div style={{ width: '100%', height: '100%', background: '#1a1b1e' }}>
      {page === 'dashboard' && (
        <Dashboard onNavigate={p => setPage(p)} onMinimize={() => setFloatVisible(true)} onClose={() => window.close()} />
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
