# Interaction Review Report

Review date: 2026-07-24  
Target: Usage-Float v0.3  
Focus: interaction logic, navigation, feedback, empty/error/retry, unconnected features, float window

---

## Navigation & Feedback

### P0: Header minimize/close buttons crash on History & Models pages

**Files:** `web/src/pages/History.tsx:79`, `web/src/pages/Models.tsx:45`, `web/src/components/Header.tsx:6-13`

Both pages pass `onMinimize={undefined}` and `onClose={undefined}` to `<Header>`. Header unconditionally renders the minimize (—) and close (✕) buttons, each wired as `onClick={onMinimize}` / `onClick={onClose}`. Clicking either calls `undefined()`, throwing a runtime error.

**Fix:** Either:
- Don't render those buttons when the handler is undefined; or
- Supply no-op handlers; or
- Make Header accept an optional `showWindowControls` flag and conditionally render.

**Also:** Settings page bypasses Header entirely (self-contained modal-like layout), so the bug doesn't affect it — but there's no consistency.

### P1: No keyboard navigation

**Files:** all pages, `AccountDialog.tsx`

- No `Escape` key handler to close the AccountDialog modal.
- No `Enter` to submit the dialog form.
- Sidebar navigation has no keyboard support (arrow keys, no tab trapping).
- No global keyboard shortcuts (e.g. `Ctrl+1`→Dashboard, `Ctrl+2`→History).

### P2: Sidebar active state indicator is weak

**File:** `web/src/components/Sidebar.tsx:24-30`

Active item is distinguished only by background color change (`surfaceHover` vs `transparent`) and text color (`accentBlue` vs `textSecondary`). Common desktop pattern: add a 3px left-border accent bar on the active item for immediate visual scanning.

### P2: No URL-based page routing

**File:** `web/src/App.tsx:31`

Navigation uses `useState('dashboard')` in App.tsx. Pages are not addressable by URL, no browser back/forward support, no deep-linkable sub-pages. Not critical for a Tauri app but limits future extensibility (e.g., opening links from float window to a specific page).

---

## Empty / Error / Retry States

### P0: Error state retry button missing on History & Models

**Files:** `web/src/pages/History.tsx:89-97`, `web/src/pages/Models.tsx:55-63`

Both pages show an error icon + message but no retry button. Dashboard has one — inconsistent. User cannot recover from a transient failure without navigating away and back.

**Fix:** Add a "重试" button calling `loadData()` (or `loadHistory()` for History).

### P1: Auto-refresh failures silently swallowed

**File:** `web/src/services/usage-service.ts:72-78`

```typescript
refreshTimer = setInterval(async () => {
  try {
    await refreshAndNotify();
  } catch { /* silent */ }
}, intervalMs);
```

Any auto-refresh error is caught and completely ignored — no log, no UI indicator. Users see stale data without knowing the refresh failed.

**Fix:** Log the error at minimum; optionally expose a "last refresh status" state for the UI.

### P1: AccountTable per-account refresh failures have no user-facing feedback

**File:** `web/src/components/AccountTable.tsx:44-46`

```typescript
catch (err) {
  log.error(`refresh_one ${accountId} failed:`, err);
}
```

If `refreshOneUsage` fails, the UI shows "点击刷新获取配额数据" indefinitely (same as never-refreshed state). User sees no error indicator and doesn't know whether the data is stale or the account is broken.

**Fix:** Add an error state per account card (inline text in tertiary color or a warning icon).

### P1: History page: two independent load states with no error coordination

**File:** `web/src/pages/History.tsx:62-72`

`loadData()` (tokens) and `loadHistory()` (history items) run independently. If tokens fails but history succeeds, the page could show "数据加载失败" while the history section tries to render beneath it — or vice versa. There's no coordination between the two loading paths.

**Fix:** Either:
- Make the whole page fail on any critical load error; or
- Show partial content with inline error banners for the failed section.

### P2: Main data error states (Dashboard/History/Models) are unreachable in current code

**Files:** `web/src/providers/tauri-provider.ts:18-29`, `web/src/providers/mock-provider.ts:5-28`

TauriProvider catches all `invoke` errors and returns fallback data. MockProvider never throws. This means the `loadState === 'error'` branches in Dashboard/History/Models are **dead code** as written. They're good defensive code to keep, but they'll never be exercised unless the provider architecture changes.

**Recommendation:** Keep them as defensive code but consider adding a note that they're fallback-only. Alternatively, if the `getFallbackAccount()` returns `status: 'error'`, the UI could use that to show a degraded-but-not-broken state.

---

## Unconnected Features

### P0: Settings toggles and chips are purely visual — no interaction

**File:** `web/src/pages/Settings.tsx:26-50`

The following UI controls render as interactive elements but have **no event handlers, no state, no backend wiring**:

| Control | Line | Visual | Behavior |
|---------|------|--------|----------|
| 开机自动启动 toggle | 27 | ON-state (blue + knob right) | No onClick, no state, no Tauri `set_autostart` call |
| 刷新频率 chips | 31-33 | "5分钟" active, others inactive | No onClick, no state, no `startAutoRefresh` interval change |
| 悬浮球 toggle | 43 | ON-state (blue + knob right) | No onClick, no state, no float window show/hide |
| 主题 selector | 46-49 | Shows "深色模式" with arrow | No onClick, no theme switching |

**Fix for each:** Pick one strategy per control:
- **Implement** the backend + state (preferred for essential features);
- **Hide** the control entirely if the feature is not planned;
- **Show as disabled** with a tooltip "即将推出" if the feature is planned for later.

### P2: Toggles show ON position by default, misleading the user

**File:** `web/src/pages/Settings.tsx:8-9`

```typescript
const toggle = { ... background: t.accentBlue, ... };
const knob = { ... right: 2 };
```

The toggle style renders a filled-blue background with the knob at `right: 2` — visually "ON". But there's no state controlling this; it's always ON-looking. If the feature is not wired, this is misleading.

**Fix:** Either implement the feature or render the toggle in OFF position (gray background, knob left) as a visual affordance that it's not active.

---

## Float Window Interaction

### P1: Auto-refresh timer race between App.tsx and FloatWindow.tsx

**Files:** `web/src/services/usage-service.ts:69-79,82-87`, `web/src/App.tsx:33-37`, `web/src/components/FloatWindow.tsx:49-55`

Both App.tsx and FloatWindow.tsx call `startAutoRefresh(5 * 60 * 1000)` on mount and `stopAutoRefresh()` on cleanup.

- First mount starts the timer (module-level `refreshTimer`).
- Second mount sees `refreshTimer !== null` and skips — safe.
- **If FloatWindow unmounts first:** `stopAutoRefresh()` clears the timer and sets it to `null`. Now App.tsx loses its auto-refresh.
- **If App.tsx unmounts first:** main window closes, FloatWindow keeps a dead timer.

**Fix:** Make `startAutoRefresh` / `stopAutoRefresh` reference-counted, or move the timer lifecycle to a single-owner module that exposes only a subscribe/unsubscribe pattern (no start/stop). The simplest fix: remove `startAutoRefresh`/`stopAutoRefresh` from FloatWindow.tsx — it already subscribes to notifications, so the App.tsx timer suffices to push updates to all windows.

### P2: FloatWindow loading state is invisible

**File:** `web/src/components/FloatWindow.tsx:81-83`

```typescript
if (!quota && !error) {
  return null;
}
```

During initial data load, the float window renders nothing. A Tauri window with zero content appears as a blank/transparent box. Brief window flash.

**Fix:** Show a minimal spinner or skeleton before the first data arrives.

### P2: Error state drag region covers the close button

**File:** `web/src/components/FloatWindow.tsx:93-94`

Error state renders `data-tauri-drag-region` covering `calc(100% - 48px)` from the left top — leaving 48px on the right for buttons. The FloatWidget's close button is at `top: 8, right: 10` (~28px from right edge), so it's clickable. This is **acceptable** but fragile: if FloatWidget's layout changes, the drag region / close-button overlap could resurface. The normal state (line 108-109) has the same setup.

**Recommendation:** Document the drag region boundaries in a comment referencing button positions (already partially done). Consider using a `data-tauri-drag-region` on the content area with `pointer-events: none` on the drag region, but that's a broader Tauri API concern.

---

## Summary

| Priority | Count | Key items |
|----------|-------|-----------|
| **P0** | 3 | Header buttons crash on History/Models; Settings controls are purely visual; error/retry buttons missing on History/Models |
| **P1** | 5 | No keyboard nav; auto-refresh failures silent; AccountTable refresh errors invisible; History load states uncoordinated; auto-refresh timer race |
| **P2** | 5 | Weak sidebar active indicator; no URL routing; unreachable error branches; toggles misleading ON state; float window loading invisible |
