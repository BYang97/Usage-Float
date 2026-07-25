// Design tokens — 与 index.css 的 @theme 共享同一套 CSS 变量。
// 修改 token 只动 index.css 的 @theme,这里自动跟随。
// 组件通过 `t.surface` 等以 JS 形式引用,等价于 Tailwind 类 `bg-surface`。

export const t = {
  surface: 'var(--color-surface)',
  surfaceAlt: 'var(--color-surface-alt)',
  surfaceHover: 'var(--color-surface-hover)',
  surfaceBorder: 'var(--color-surface-border)',
  textPrimary: 'var(--color-text-primary)',
  textSecondary: 'var(--color-text-secondary)',
  textTertiary: 'var(--color-text-tertiary)',
  accentBlue: 'var(--color-accent-blue)',
  accentGreen: 'var(--color-accent-green)',
  accentCyan: 'var(--color-accent-cyan)',
  statusOk: 'var(--color-status-ok)',
  statusWarning: 'var(--color-status-warning)',
  statusDanger: 'var(--color-status-danger)',
  glass: 'rgba(26, 27, 30, 0.72)',
  // 字体大小
  fsH1: 'var(--fs-h1)',
  fsH2: 'var(--fs-h2)',
  fsH3: 'var(--fs-h3)',
  fsBody: 'var(--fs-body)',
  fsSecondary: 'var(--fs-secondary)',
  fsWeak: 'var(--fs-weak)',
  fsHero: 'var(--fs-hero)',
} as const;
