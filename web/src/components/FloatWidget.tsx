import { t } from '../tokens';
import { ProviderBadge } from './ProviderBadge';
import { QuotaRing } from './QuotaRing';
import type { CSSProperties } from 'react';

interface Props {
  percentage: number;
  resetTime: string;
  onOpenDashboard: () => void;
  onClose: () => void;
  /** 可选的自定义样式（用于父组件定位） */
  style?: CSSProperties;
}

/**
 * 悬浮球内容组件（纯展示，不含拖拽逻辑）
 *
 * 拖拽由父组件决定策略：
 * - 在 FloatWindow 中通过 data-tauri-drag-region 实现 OS 窗口拖拽
 * - 在主窗口叠加层中通过 DraggableWrapper 实现 JS 拖拽
 */
export function FloatWidget({ percentage, resetTime, onOpenDashboard, onClose, style }: Props) {
  return (
    <div
      className="glass"
      style={{
        width: 320, height: 180, borderRadius: 12,
        boxShadow: '0 8px 32px rgba(0,0,0,0.4)', overflow: 'hidden',
        position: 'relative', display: 'flex', flexDirection: 'column', alignItems: 'center',
        ...style,
      }}>
      <div style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: 36 }} />
      <div style={{ position: 'absolute', top: 10, left: 16 }}><ProviderBadge /></div>
      <button onClick={onClose}
        style={{ position: 'absolute', top: 8, right: 10, width: 24, height: 24, display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'transparent', border: 'none', color: t.textTertiary, cursor: 'pointer', fontSize: 12 }}>
        &#10005;
      </button>
      <div style={{ position: 'absolute', top: 8, right: 36, width: 24, height: 24, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={{ width: 10, height: 2, borderRadius: 1, background: t.textTertiary }} />
      </div>
      <div style={{ marginTop: 16 }}><QuotaRing percentage={percentage} /></div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 8 }}>
        <span style={{ fontSize: t.fsSecondary, fontWeight: 500, color: t.textSecondary }}>5小时额度</span>
        <span style={{ width: 3, height: 3, borderRadius: '50%', background: t.textTertiary }} />
        <span style={{ fontSize: t.fsSecondary, color: t.textTertiary }}>重置：{resetTime}</span>
      </div>
      <button onClick={onOpenDashboard}
        style={{ marginTop: 2, fontSize: t.fsWeak, fontWeight: 500, color: t.accentBlue, background: 'transparent', border: 'none', cursor: 'pointer' }}>
        打开仪表盘 &gt;
      </button>
    </div>
  );
}
