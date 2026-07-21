import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { ErrorBoundary } from './components/ErrorBoundary'
import { FloatWindow } from './components/FloatWindow'

// ─── Tauri 双窗口路由 ───────────────────────────────────────────────
// 主窗口加载完整 App，悬浮球窗口只渲染 FloatWidget
// 窗口 URL 由 tauri.conf.json 中的 url 字段控制
const params = new URLSearchParams(window.location.search)
const isFloatWindow = params.get('window') === 'float'

if (isFloatWindow) {
  // 悬浮球窗口：body 透明以支持 Tauri 透明窗口效果
  document.documentElement.dataset.window = 'float'
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <ErrorBoundary>
        <FloatWindow />
      </ErrorBoundary>
    </StrictMode>,
  )
} else {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </StrictMode>,
  )
}
