# OpenCode Usage Float — UI 设计规范

> 直接用于 React + TailwindCSS + Tauri 前端开发。

---

## 1. Design Tokens

### 1.1 色彩系统

```ts
// tailwind.config.ts
export default {
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#1a1b1e',   // 主背景
          alt:     '#222327',   // 卡片背景
          hover:   '#2c2e33',   // hover 态
          border:  '#2f3036',   // 边框
        },
        text: {
          primary:   '#e4e5e7',
          secondary: '#8b8d97',
          tertiary:  '#5c5e66',
          inverse:   '#0d0d0f',
        },
        accent: {
          blue:  '#4a9eff',
          green: '#34d399',
          cyan:  '#22d3ee',
        },
        status: {
          ok:       '#34d399',  // 正常  > 50%
          warning:  '#fbbf24',  // 警告  20–50%
          danger:   '#ef4444',  // 危险  < 20%
        },
        glass: {
          bg:    'rgba(26, 27, 30, 0.72)',
          blur:  'rgba(255, 255, 255, 0.04)',
          edge:  'rgba(255, 255, 255, 0.06)',
        },
      },
      fontFamily: {
        mono:  ['JetBrains Mono', 'Cascadia Code', 'Consolas', 'monospace'],
        sans:  ['Inter', 'Segoe UI', '-apple-system', 'sans-serif'],
      },
    },
  },
}
```

### 1.2 字体规范

| Token | Size | Weight | Line Height | Font |
|-------|------|--------|-------------|------|
| `text-hero`   | 32px | 700 | 1.2 | sans |
| `text-h1`     | 20px | 600 | 1.3 | sans |
| `text-h2`     | 16px | 600 | 1.4 | sans |
| `text-body`   | 13px | 400 | 1.5 | sans |
| `text-small`  | 11px | 500 | 1.4 | sans |
| `text-mono-lg`| 24px | 600 | 1.2 | mono |
| `text-mono`   | 13px | 500 | 1.5 | mono |
| `text-mono-sm`| 11px | 500 | 1.4 | mono |

### 1.3 间距

```
space-1:   2px
space-2:   4px
space-3:   8px
space-4:   12px
space-5:   16px
space-6:   20px
space-7:   24px
space-8:   32px
space-9:   40px
space-10:  48px
```

### 1.4 圆角

```
rounded-sm:   4px
rounded-md:   6px
rounded-lg:   8px
rounded-xl:   12px
rounded-full: 9999px
```

### 1.5 阴影

```css
/* 卡片阴影 */
box-shadow: 0 1px 3px rgba(0,0,0,0.3), 0 1px 2px rgba(0,0,0,0.2);
/* 悬浮球阴影 */
box-shadow: 0 8px 32px rgba(0,0,0,0.4);
/* 弹窗阴影 */
box-shadow: 0 16px 48px rgba(0,0,0,0.5);
```

### 1.6 玻璃效果 (Float Widget)

```css
background: rgba(26, 27, 30, 0.72);
backdrop-filter: blur(20px);
-webkit-backdrop-filter: blur(20px);
border: 1px solid rgba(255, 255, 255, 0.06);
```

---

## 2. 组件树

```
App
├── FloatWidget          (AlwaysOnTop 窗口, 320×180)
│   ├── ProviderBadge
│   ├── QuotaRing
│   └── WidgetFooter
│
├── Dashboard            (主窗口, 1000×700)
│   ├── Header
│   ├── PlanCard
│   ├── QuotaGrid
│   │   ├── QuotaCard    (5H Window)
│   │   ├── QuotaCard    (Weekly Window)
│   │   └── QuotaCard    (Monthly)
│   ├── TokenUsage
│   │   └── TokenChart   (折线图, 7天)
│   └── ModelStats
│       └── ModelBar
│
└── Settings             (Modal / 页面)
    ├── SettingsSection  (General)
    ├── SettingsSection  (Display)
    └── SettingsSection  (Privacy)
```

---

## 3. 组件接口定义

### 3.1 `ProviderBadge`

```tsx
interface ProviderBadgeProps {
  name: string          // "OpenCode GO"
  status: 'active' | 'expired' | 'error'
}
```

左侧圆点指示器 + 名称。圆点 6×6，status 对应 `bg-status-*`。

### 3.2 `QuotaRing`

```tsx
interface QuotaRingProps {
  percentage: number    // 0–100
  size?: number         // 默认 72
  strokeWidth?: number  // 默认 6
  label?: string        // 默认 "82%"
}
```

SVG circle 实现。底色 `stroke` 使用 `surface-border`，前景色动态根据 percentage 映射到 `status-*`。

### 3.3 `QuotaCard`

```tsx
interface QuotaCardProps {
  title: string         // "5 Hour Window" | "Weekly Window" | "Monthly"
  percentage: number    // 0–100
  resetTime?: string    // "01:42:30" | "Friday 09:00"
  status: 'ok' | 'warning' | 'danger'
  progressVariant?: 'bar' | 'ring'  // 5H 用 bar, Weekly/Monthly 用 bar
}
```

| Status | 阈值 | 色值 |
|--------|------|------|
| `ok`      | ≥ 50% | `#34d399` |
| `warning` | 20–50% | `#fbbf24` |
| `danger`  | < 20% | `#ef4444` |

### 3.4 `TokenChart`

```tsx
interface TokenChartProps {
  days: 7 | 30
  data: { date: string; tokens: number }[]
}
```

使用 Recharts `<AreaChart>`：
- 填充渐变：`url(#tokenGradient)` — 从 `accent-blue / 0.15` 到透明
- 线条：`accent-blue`，宽度 2
- 网格：虚线 `surface-border`
- 无图例，Y 轴 label 如 "8.5M"

### 3.5 `ModelBar`

```tsx
interface ModelBarProps {
  models: { name: string; percentage: number; color: string }[]
}
// models: [
//   { name: 'GPT',    percentage: 60, color: '#4a9eff' },
//   { name: 'Claude', percentage: 40, color: '#d97706' },
// ]
```

水平堆叠条，高度 8px，圆角，末端显示百分比数字。

### 3.6 `FloatWidget`

```tsx
interface FloatWidgetProps {
  percentage: number     // 整体额度百分比
  remainingLabel: string // "5h Window"
  resetCountdown: string // "01:42:30"
}
```

固定在桌面右下角附近 (x: viewportWidth - 340, y: viewportHeight - 220)。

Tauri 侧配置：
```rust
window.set_always_on_top(true);
window.set_decorations(false);
window.set_transparent(true);
window.set_skip_taskbar(true);
```

### 3.7 `ProgressBar`

```tsx
interface ProgressBarProps {
  percentage: number
  status: StatusColor
  height?: number // 默认 8
  showLabel?: boolean
}
```

背景 `surface-border`，前景填满对应 status 色，带动画过渡 `transition-all duration-500`。

---

## 4. 页面布局

### 4.1 Float Widget (320×180)

```
┌─────────────────────────────────┐
│ [●] OpenCode GO          ✕  ── │   ← ProviderBadge + window controls
│                                   │
│              ┌───────┐          │
│              │  82%  │          │   ← QuotaRing (size: 72, centered)
│              └───────┘          │
│                                   │
│  5h Window    ·    Reset         │
│  82%             01:42:30        │   ← compact row
│                                   │
│        打开 Dashboard  ›         │   ← clickable footer
└─────────────────────────────────┘
```

布局：
- 上下 padding: `space-5` (16px)
- 左右 padding: `space-4` (12px)
- 圆环居中，上方留 `space-4` 间距
- 底部 footer 居中对齐，`text-small` 字号，hover 变 `accent-blue`

### 4.2 Dashboard (1000×700)

```
┌──────────────────────────────────────────────────────────────┐
│ OpenCode Usage Float                     [⚙] [─] [✕]       │   ← Header
├──────────────────────────────────────────────────────────────┤
│ ┌─ OpenCode GO ──────────────────────────────────────────┐  │
│ │  Plan: GO Monthly    Status: ● Active                  │  │
│ │  Expire: 2026-08-20                                    │  │   ← PlanCard
│ └────────────────────────────────────────────────────────┘  │
│                                                              │
│ ┌────────────┐ ┌────────────┐ ┌────────────┐               │
│ │ 5h Window  │ │ Weekly     │ │ Monthly    │               │   ← QuotaGrid
│ │ ██████░░ 82%│ │ █████░░ 63%│ │ ████░░ 45%│               │     (3 columns, gap-5)
│ │ Reset:     │ │ Reset:     │ │            │               │
│ │ 01:42:30   │ │ Fri 09:00  │ │            │               │
│ └────────────┘ └────────────┘ └────────────┘               │
│                                                              │
│ ┌─ Token Usage ─────────────────────────────────────────┐  │
│ │  Today  7 Days  30 Days                               │  │
│ │  8.5M   42M     180M            ────────              │  │   ← TokenChart
│ │                                /        \             │  │     (area chart)
│ │                      ─────────/          ────         │  │
│ └───────────────────────────────────────────────────────┘  │
│                                                              │
│ ┌─ Model Usage ────────────────────────────────────────┐  │
│ │  GPT    ████████████████████████████░░░░ 60%         │  │   ← ModelBar
│ │  Claude ██████████████████░░░░░░░░░░░░ 40%           │  │
│ └───────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

布局网格：
```
grid-template-columns: repeat(3, 1fr)  — QuotaGrid
col-span-2                              — TokenChart (占2列)
col-span-1                              — ModelStats (占1列)
```

行间距：`space-7` (24px)  
列间距：`space-5` (16px)

### 4.3 Settings (Modal overlay, centered)

覆盖在 Dashboard 之上，半透明背景遮罩 `rgba(0,0,0,0.5)`。

Modal 尺寸：480px × 520px

```
┌──────────────────────────────────────┐
│ Settings                         ✕   │   ← 标题行
├──────────────────────────────────────┤
│                                      │
│ General                              │
│ ┌──────────────────────────────────┐ │
│ │ 启动时运行                [开关] │ │
│ │ 自动刷新        [5m] [30m] [60m] │ │   ← radio group
│ └──────────────────────────────────┘ │
│                                      │
│ Display                              │
│ ┌──────────────────────────────────┐ │
│ │ 悬浮球                     [开关]│ │   ← toggle switch
│ │ 主题                    Dark  ›  │ │   ← select
│ └──────────────────────────────────┘ │
│                                      │
│ Privacy                              │
│ ┌──────────────────────────────────┐ │
│ │ 🔒 所有数据只保存在本地          │ │   ← icon + text
│ │     Local Only                   │ │
│ └──────────────────────────────────┘ │
│                                      │
└──────────────────────────────────────┘
```

---

## 5. 动画 & 交互

| 元素 | 行为 | 时长 | Easing |
|------|------|------|--------|
| 悬浮球 hover | 略微放大 scale(1.02) | 150ms | ease-out |
| 卡片 enter | fadeIn + translateY(4→0) | 200ms | ease-out |
| 进度条填充 | width 动画 | 600ms | ease-in-out |
| 折线图绘制 | 路径 stroke-dashoffset | 800ms | ease-in-out |
| 切换页面 | opacity 过渡 | 150ms | ease |
| 玻璃层 hover | 亮度微增 brightness(1.05) | 100ms | ease |

---

## 6. TailwindCSS 配置参考

```ts
// tailwind.config.ts
import type { Config } from 'tailwindcss'

export default {
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#1a1b1e',
          alt:     '#222327',
          hover:   '#2c2e33',
          border:  '#2f3036',
        },
        text: {
          primary:   '#e4e5e7',
          secondary: '#8b8d97',
          tertiary:  '#5c5e66',
          inverse:   '#0d0d0f',
        },
        accent: {
          blue:  '#4a9eff',
          green: '#34d399',
          cyan:  '#22d3ee',
        },
        status: {
          ok:      '#34d399',
          warning: '#fbbf24',
          danger:  '#ef4444',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Cascadia Code', 'Consolas', 'monospace'],
        sans: ['Inter', 'Segoe UI', '-apple-system', 'sans-serif'],
      },
      fontSize: {
        hero:   ['32px', { lineHeight: '1.2', fontWeight: '700' }],
        h1:     ['20px', { lineHeight: '1.3', fontWeight: '600' }],
        h2:     ['16px', { lineHeight: '1.4', fontWeight: '600' }],
        body:   ['13px', { lineHeight: '1.5', fontWeight: '400' }],
        small:  ['11px', { lineHeight: '1.4', fontWeight: '500' }],
        'mono-lg': ['24px', { lineHeight: '1.2', fontWeight: '600', fontFamily: 'JetBrains Mono' }],
        'mono':    ['13px', { lineHeight: '1.5', fontWeight: '500', fontFamily: 'JetBrains Mono' }],
        'mono-sm': ['11px', { lineHeight: '1.4', fontWeight: '500', fontFamily: 'JetBrains Mono' }],
      },
      spacing: {
        1: '2px',
        2: '4px',
        3: '8px',
        4: '12px',
        5: '16px',
        6: '20px',
        7: '24px',
        8: '32px',
        9: '40px',
        10: '48px',
      },
      borderRadius: {
        sm:  '4px',
        md:  '6px',
        lg:  '8px',
        xl:  '12px',
      },
      boxShadow: {
        card:  '0 1px 3px rgba(0,0,0,0.3), 0 1px 2px rgba(0,0,0,0.2)',
        float: '0 8px 32px rgba(0,0,0,0.4)',
        modal: '0 16px 48px rgba(0,0,0,0.5)',
      },
      backdropBlur: {
        glass: '20px',
      },
    },
  },
  plugins: [],
} satisfies Config
```

---

## 7. 全局样式 (index.css)

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  * { @apply m-0 p-0 box-border; }

  html {
    @apply bg-surface text-text-primary font-sans text-body
           antialiased select-none;
  }

  /* 隐藏默认标题栏 — Tauri 无边框窗口 */
  body { @apply overflow-hidden; }
  ::-webkit-scrollbar { @apply w-1; }
  ::-webkit-scrollbar-track { @apply bg-transparent; }
  ::-webkit-scrollbar-thumb { @apply bg-surface-border rounded-full; }
}

@layer components {
  .glass {
    background: rgba(26, 27, 30, 0.72);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .card {
    @apply bg-surface-alt border border-surface-border rounded-lg shadow-card;
  }

  .card-hover {
    @apply card transition-colors duration-150;
  }
  .card-hover:hover {
    @apply bg-surface-hover;
  }

  .tag {
    @apply inline-flex items-center gap-1 px-2 py-0.5
           rounded-sm text-small font-medium;
  }
  .tag-ok      { @apply tag bg-status-ok/10 text-status-ok; }
  .tag-warning { @apply tag bg-status-warning/10 text-status-warning; }
  .tag-danger  { @apply tag bg-status-danger/10 text-status-danger; }
}
```

---

## 8. 图标

使用 [Lucide React](https://lucide.dev) 图标库（Tree-shakable，与 Tauri 友好）。

| 用途 | 图标名 |
|------|--------|
| 设置 | `Settings` |
| 关闭 | `X` |
| 最小化 | `Minus` |
| 最大化 | `Maximize2` |
| 打开链接 | `ExternalLink` |
| 锁 | `Lock` |
| 时钟 | `Clock` |
| 数据 | `BarChart3` |
| 模型 | `Cpu` |
| 刷新 | `RefreshCw` |
| 拖拽点 | `GripVertical` |

---

## 9. Tauri 窗口配置

```json
// src-tauri/tauri.conf.json (相关部分)
{
  "windows": [
    {
      "label": "float",
      "title": "OpenCode Usage Float",
      "width": 320,
      "height": 180,
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true,
      "skipTaskbar": true,
      "resizable": false,
      "center": false,           // 通过 JS 设置到右下角
      "visible": true
    },
    {
      "label": "main",
      "title": "OpenCode Usage Float",
      "width": 1000,
      "height": 700,
      "decorations": false,
      "transparent": false,
      "center": true,
      "resizable": true,
      "minWidth": 800,
      "minHeight": 600,
      "visible": false           // 默认隐藏，由悬浮球触发显示
    }
  ]
}
```

---

## 10. 状态管理 (Zustand store 参考)

```ts
interface QuotaStore {
  // 数据
  plan: string
  status: 'active' | 'expired' | 'error'
  expireDate: string
  fiveHourPercent: number
  fiveHourReset: string
  weeklyPercent: number
  weeklyReset: string
  monthlyPercent: number
  tokenToday: number
  token7d: number
  token30d: number
  tokenHistory: { date: string; tokens: number }[]
  models: { name: string; percentage: number }[]
  
  // UI
  isFloatVisible: boolean
  isSettingsOpen: boolean
  autoRefreshMinutes: number
  launchAtStartup: boolean

  // Actions
  refresh: () => Promise<void>
  toggleFloat: () => void
  toggleSettings: () => void
}
```
