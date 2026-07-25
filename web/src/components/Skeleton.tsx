import type { CSSProperties } from 'react';
import { t } from '../tokens';

interface Props {
  width?: string | number;
  height?: string | number;
  borderRadius?: string | number;
  variant?: 'text' | 'card' | 'circle';
}

const baseStyle: CSSProperties = {
  background: `linear-gradient(90deg, ${t.surfaceHover} 25%, ${t.surfaceBorder} 50%, ${t.surfaceHover} 75%)`,
  backgroundSize: '200% 100%',
  animation: 'shimmer 1.5s ease-in-out infinite',
};

export function Skeleton({ width, height, borderRadius, variant = 'text' }: Props) {
  const style: CSSProperties = { ...baseStyle };

  if (variant === 'card') {
    style.width = width ?? '100%';
    style.height = height ?? 120;
    style.borderRadius = borderRadius ?? 'var(--radius-2xl)';
  } else if (variant === 'circle') {
    style.width = width ?? 24;
    style.height = height ?? 24;
    style.borderRadius = borderRadius ?? '50%';
  } else {
    // text line
    style.width = width ?? '100%';
    style.height = height ?? 14;
    style.borderRadius = borderRadius ?? 4;
  }

  return <div style={style} />;
}
