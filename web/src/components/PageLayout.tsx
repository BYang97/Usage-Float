import { type ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { Header } from './Header';

interface Props {
  active: string;
  title?: string;
  onNavigate: (page: string) => void;
  onMinimize?: () => void;
  onClose?: () => void;
  children: ReactNode;
}

/**
 * 统一页面布局：Sidebar(240) + Header(56) + 内容区(p-4 gap-3)
 *
 * 所有主窗口页面(Dashboard / History / Models / Settings)统一使用此布局。
 */
export function PageLayout({ active, title, onNavigate, onMinimize, onClose, children }: Props) {
  return (
    <div style={{ display: 'flex', height: '100%' }}>
      <Sidebar active={active} onNavigate={onNavigate} />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        <Header
          title={title}
          onSettings={() => onNavigate('settings')}
          onMinimize={onMinimize}
          onClose={onClose}
        />
        <div style={{
          flex: 1, overflowY: 'auto',
          padding: 16, display: 'flex', flexDirection: 'column', gap: 12,
        }}>
          {children}
        </div>
      </div>
    </div>
  );
}
