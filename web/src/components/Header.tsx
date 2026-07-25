import { t } from '../tokens';

interface Props {
  title?: string;
  onSettings?: () => void;
  onMinimize?: () => void;
  onClose?: () => void;
}

export function Header({ title = 'OpenCode Go', onSettings, onMinimize, onClose }: Props) {
  const btn = {
    width: 28, height: 28,
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    borderRadius: 6, border: 'none', background: 'transparent',
    color: t.textSecondary, cursor: 'pointer', fontSize: 12,
  } as const;
  return (
    <div style={{
      height: 56, display: 'flex', alignItems: 'center',
      justifyContent: 'space-between',
      paddingLeft: 16, paddingRight: 16,
      flexShrink: 0, background: t.surface,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <button style={btn} onClick={onSettings}>&#9881;</button>
        <span style={{ fontSize: t.fsH2, fontWeight: 600, color: t.textPrimary }}>
          {title}
        </span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
        <button style={btn} onClick={onMinimize}>&#9472;</button>
        <button style={btn} onClick={onClose}>&#10005;</button>
      </div>
    </div>
  );
}
