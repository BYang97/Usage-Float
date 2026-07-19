import { useState } from 'react';
import { Dashboard } from './pages/Dashboard';
import { Settings } from './pages/Settings';
import { FloatWidget } from './components/FloatWidget';
import { usage } from './data/mock';

export default function App() {
  const [page, setPage] = useState('dashboard');
  const [floatVisible, setFloatVisible] = useState(true);

  return (
    <div style={{ width: '100%', height: '100%', background: '#1a1b1e' }}>
      {page === 'dashboard' && (
        <Dashboard onNavigate={p => setPage(p)} onMinimize={() => setFloatVisible(true)} onClose={() => window.close()} />
      )}
      {page === 'settings' && <Settings onNavigate={p => setPage(p)} />}
      {(page === 'history' || page === 'models') && <Dashboard onNavigate={p => setPage(p)} />}

      {floatVisible && (
        <FloatWidget
          percentage={usage.fiveHourPercent}
          resetTime={usage.fiveHourReset}
          onOpenDashboard={() => { setPage('dashboard'); setFloatVisible(false); }}
          onClose={() => setFloatVisible(false)}
        />
      )}
    </div>
  );
}
