import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip } from 'recharts';
import { t } from '../tokens';

interface Props { data: { date: string; tokens: number }[]; today: string; week: string; month: string }

export function TokenUsage({ data, today, week, month }: Props) {
  return (
    <div style={{ background: t.surfaceAlt, border: `1px solid ${t.surfaceBorder}`, borderRadius: 8, padding: 20, display: 'flex', flexDirection: 'column', gap: 16 }}>
      <span style={{ fontSize: 14, fontWeight: 600, color: t.textPrimary }}>Token 消耗</span>
      <div style={{ display: 'flex', gap: 16 }}>
        {[
          { label: '今日消耗', value: today },
          { label: '近7天', value: week },
          { label: '近30天', value: month },
        ].map(s => (
          <div key={s.label} style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <span style={{ fontSize: 11, color: t.textTertiary }}>{s.label}</span>
            <span style={{ fontSize: 22, fontWeight: 700, color: t.textPrimary }}>{s.value}</span>
          </div>
        ))}
      </div>
      <div style={{ background: t.surface, borderRadius: 6, height: 140 }}>
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 20, right: 15, bottom: 25, left: 5 }}>
            <defs>
              <linearGradient id="g" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#4a9eff" stopOpacity={0.15} />
                <stop offset="100%" stopColor="#4a9eff" stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis dataKey="date" axisLine={false} tickLine={false} tick={{ fill: '#5c5e66', fontSize: 9 }} dy={8} />
            <YAxis hide />
            <Tooltip contentStyle={{ background: '#222327', border: '1px solid #2f3036', borderRadius: 6, color: '#e4e5e7', fontSize: 12 }} formatter={(v: number) => [`${v}M`, 'Token']} />
            <Area type="monotone" dataKey="tokens" stroke="#4a9eff" strokeWidth={2} fill="url(#g)" dot={false} activeDot={{ r: 3, fill: '#4a9eff' }} />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
