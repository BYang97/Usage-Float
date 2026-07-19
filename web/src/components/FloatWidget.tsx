import { useState, useCallback, type MouseEvent } from 'react';
import { t } from '../tokens';
import { ProviderBadge } from './ProviderBadge';
import { QuotaRing } from './QuotaRing';

interface Props { percentage: number; resetTime: string; onOpenDashboard: () => void; onClose: () => void }

export function FloatWidget({ percentage, resetTime, onOpenDashboard, onClose }: Props) {
  const [pos, setPos] = useState({ x: window.innerWidth - 340, y: window.innerHeight - 220 });
  const [dragging, setDragging] = useState(false);
  const [offset, setOffset] = useState({ x: 0, y: 0 });

  const onMouseDown = useCallback((e: MouseEvent) => {
    setDragging(true);
    setOffset({ x: e.clientX - pos.x, y: e.clientY - pos.y });
  }, [pos]);

  const onMouseMove = useCallback((e: MouseEvent) => {
    if (!dragging) return;
    setPos({ x: e.clientX - offset.x, y: e.clientY - offset.y });
  }, [dragging, offset]);

  const onMouseUp = useCallback(() => setDragging(false), []);

  return (
    <div
      onMouseDown={onMouseDown} onMouseMove={onMouseMove} onMouseUp={onMouseUp} onMouseLeave={onMouseUp}
      style={{
        position: 'fixed', zIndex: 50, cursor: dragging ? 'grabbing' : 'grab',
        left: pos.x, top: pos.y, width: 320, height: 180, borderRadius: 12,
        boxShadow: '0 8px 32px rgba(0,0,0,0.4)', overflow: 'hidden',
      }}>
      <div style={{
        width: '100%', height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center',
        background: t.glass, backdropFilter: 'blur(20px)', WebkitBackdropFilter: 'blur(20px)',
        border: '1px solid rgba(255,255,255,0.06)', position: 'relative',
      }}>
        <div style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: 36 }} />
        <div style={{ position: 'absolute', top: 10, left: 16 }}><ProviderBadge /></div>
        <button onClick={onClose} style={{ position: 'absolute', top: 8, right: 10, width: 24, height: 24, display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'transparent', border: 'none', color: t.textTertiary, cursor: 'pointer', fontSize: 12 }}>&#10005;</button>
        <div style={{ position: 'absolute', top: 8, right: 36, width: 24, height: 24, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <div style={{ width: 10, height: 2, borderRadius: 1, background: t.textTertiary }} />
        </div>
        <div style={{ marginTop: 16 }}><QuotaRing percentage={percentage} /></div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 8 }}>
          <span style={{ fontSize: 12, fontWeight: 500, color: t.textSecondary }}>5h Window</span>
          <span style={{ width: 3, height: 3, borderRadius: '50%', background: t.textTertiary }} />
          <span style={{ fontSize: 12, color: t.textTertiary }}>Reset: {resetTime}</span>
        </div>
        <button onClick={onOpenDashboard} style={{ marginTop: 2, fontSize: 11, fontWeight: 500, color: t.accentBlue, background: 'transparent', border: 'none', cursor: 'pointer' }}>
          Open Dashboard &gt;
        </button>
      </div>
    </div>
  );
}
