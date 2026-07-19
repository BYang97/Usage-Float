import { t } from '../tokens';

interface Props { onSettings?: () => void; onMinimize?: () => void; onClose?: () => void }

export function Header({ onSettings, onMinimize, onClose }: Props) {
  const btn = { width: 28, height: 28, display: 'flex', alignItems: 'center', justifyContent: 'center', borderRadius: 6, border: 'none', background: 'transparent', color: t.textSecondary, cursor: 'pointer', fontSize: 12 } as const;
  return (
    <div style={{ height: 52, display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingLeft: 24, paddingRight: 20, flexShrink: 0, background: t.surface }}>
      <span style={{ fontSize: 16, fontWeight: 600, color: t.textPrimary }}>OpenCode Go</span>
      <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
        <button style={btn} onClick={onSettings}>&#9881;</button>
        <button style={btn} onClick={onMinimize}>&#9472;</button>
        <button style={btn} onClick={onClose}>&#10005;</button>
      </div>
    </div>
  );
}
