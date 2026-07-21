import { Component, type ErrorInfo, type ReactNode } from 'react';
import { t } from '../tokens';

interface Props {
  children: ReactNode;
  /** 可选的降级 UI，不传则使用默认错误提示 */
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/**
 * React 错误边界
 *
 * 捕获子组件树中的渲染异常，防止整个应用白屏。
 * 显示友好的错误提示，并提供重试按钮。
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[ErrorBoundary] 捕获异常:', error, errorInfo.componentStack);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div style={{
          width: '100%', height: '100%',
          display: 'flex', flexDirection: 'column',
          alignItems: 'center', justifyContent: 'center',
          gap: 16,
          background: t.surface,
          color: t.textSecondary,
          padding: 40,
        }}>
          <div style={{
            width: 48, height: 48, borderRadius: '50%',
            background: t.surfaceHover,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            fontSize: 22,
          }}>
            &#9888;
          </div>
          <div style={{ fontSize: 15, fontWeight: 600, color: t.textPrimary }}>
            应用出现异常
          </div>
          <div style={{ fontSize: 12, textAlign: 'center', maxWidth: 400, lineHeight: 1.6 }}>
            {this.state.error?.message || '发生了意外错误，请尝试刷新'}
          </div>
          <button
            onClick={this.handleRetry}
            style={{
              marginTop: 8, padding: '8px 20px', borderRadius: 6,
              border: 'none', background: t.accentBlue, color: '#fff',
              fontSize: 13, fontWeight: 500, cursor: 'pointer',
            }}
          >
            重试
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
