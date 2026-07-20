import { useEffect, useState } from 'react';
import { Sidebar } from '../components/Sidebar';
import { Header } from '../components/Header';
import { PlanCard } from '../components/PlanCard';
import { QuotaCard } from '../components/QuotaCard';
import { TokenUsage } from '../components/TokenUsage';
import { ModelUsage } from '../components/ModelUsage';
import { getAccount, getQuota, getTokens, getModels } from '../services/usage-service';
import type { AccountInfo, QuotaInfo, TokenInfo, ModelUsageData } from '../types';

interface Props { onNavigate: (page: string) => void; onMinimize?: () => void; onClose?: () => void }

export function Dashboard({ onNavigate, onMinimize, onClose }: Props) {
  const [account, setAccount] = useState<AccountInfo | null>(null);
  const [quota, setQuota] = useState<QuotaInfo | null>(null);
  const [tokens, setTokens] = useState<TokenInfo | null>(null);
  const [models, setModels] = useState<ModelUsageData[] | null>(null);

  useEffect(() => {
    getAccount().then(setAccount);
    getQuota().then(setQuota);
    getTokens().then(setTokens);
    getModels().then(setModels);
  }, []);

  if (!account || !quota || !tokens || !models) return null;

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
