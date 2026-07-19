import { useMemo } from 'react';
import { t } from '../tokens';
import { Sidebar } from '../components/Sidebar';
import { Header } from '../components/Header';
import { PlanCard } from '../components/PlanCard';
import { QuotaCard } from '../components/QuotaCard';
import { TokenUsage } from '../components/TokenUsage';
import { ModelUsage } from '../components/ModelUsage';
import { usage } from '../data/mock';

interface Props { onNavigate: (page: string) => void; onMinimize?: () => void; onClose?: () => void }

export function Dashboard({ onNavigate, onMinimize, onClose }: Props) {
  const chartData = useMemo(() => usage.tokenHistory.map(d => ({ ...d })), []);
  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active="dashboard" onNavigate={onNavigate} />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        <Header onSettings={() => onNavigate('settings')} onMinimize={onMinimize} onClose={onClose} />
        <div style={{ flex: 1, overflowY: 'auto', padding: 24, display: 'flex', flexDirection: 'column', gap: 20 }}>
          <PlanCard plan={usage.plan} status={usage.status} expireDate={usage.expireDate} />
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
            <QuotaCard title="5 Hour Window" percentage={usage.fiveHourPercent} resetTime={usage.fiveHourReset} />
            <QuotaCard title="Weekly Window" percentage={usage.weeklyPercent} resetTime={usage.weeklyReset} />
            <QuotaCard title="Monthly" percentage={usage.monthlyPercent} />
          </div>
          <div style={{ display: 'flex', gap: 16 }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <TokenUsage data={chartData} today={usage.tokenToday} week={usage.token7d} month={usage.token30d} />
            </div>
            <div style={{ width: 296, flexShrink: 0 }}>
              <ModelUsage models={usage.models} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
