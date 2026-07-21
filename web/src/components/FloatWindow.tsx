import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { FloatWidget } from './FloatWidget';
import { getQuota, subscribe, startAutoRefresh, stopAutoRefresh } from '../services/usage-service';
import type { QuotaInfo } from '../types';

/**
 * 悬浮球窗口根组件
 *
 * 统一通过 usage-service 获取数据，消除硬编码。
 * 支持自动刷新：启动时加载数据，订阅数据变更，定时刷新。
 *
 * 窗口交互：
 * - data-tauri-drag-region：CSS 区域拖拽整个 OS 窗口（无边框窗口必需）
 * - 关闭按钮调用 Tauri Window.hide() 隐藏窗口而非退出应用
 */
export function FloatWindow() {
  const [quota, setQuota] = useState<QuotaInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        setError(null);
        const data = await getQuota();
        if (!cancelled) setQuota(data);
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : '加载失败');
        }
      }
    }

    // 首次加载
    load();

    // 订阅数据变更
    const unsub = subscribe((data) => {
      if (!cancelled) {
        setQuota(data.quota);
        setError(null);
      }
    });

    // 启动定时刷新（5 分钟）
    startAutoRefresh(5 * 60 * 1000);

    return () => {
      cancelled = true;
      unsub();
      stopAutoRefresh();
    };
  }, []);

  /** 隐藏悬浮窗口，不退出应用 */
  const handleClose = async () => {
    try {
      await getCurrentWindow().hide();
    } catch (err) {
      console.error('[FloatWindow] hide failed:', err);
    }
  };

  /** 打开仪表盘 — 在主窗口显示并聚焦（后续可完善为 IPC 通信） */
  const handleOpenDashboard = () => {
    // 可通过 WebviewWindow.getByLabel('main') 调起主窗口（后续实现）
  };

  // 加载中：未出错且尚无数据，最小占位
  if (!quota && !error) {
    return null;
  }

  // 错误状态：显示降级浮窗
  if (error) {
    return (
      <div data-tauri-drag-region style={{ width: '100%', height: '100%' }}>
        <FloatWidget
          percentage={0}
          resetTime="—"
          onOpenDashboard={handleOpenDashboard}
          onClose={handleClose}
        />
      </div>
    );
  }

  // 正常渲染
  return (
    <div data-tauri-drag-region style={{ width: '100%', height: '100%' }}>
      <FloatWidget
        percentage={quota!.fiveHourPercent}
        resetTime={quota!.fiveHourReset}
        onOpenDashboard={handleOpenDashboard}
        onClose={handleClose}
      />
    </div>
  );
}
