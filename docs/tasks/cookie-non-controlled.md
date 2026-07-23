# 任务:cookie-non-controlled - cookie textarea 改非受控

## 目标
Settings.tsx 的 cookie textarea 改非受控（defaultValue + useRef），让或ca computer 的 set-value 能工作（不受 React 受控 state 阻断）。workspaceId input 保持受控（已 work）。

## 背景
或ca computer 驱动 Tauri webview，cookie textarea 受控（value={cookie}）时 set-value 设 DOM value 但 React state 不更新（mismatch），保存空 cookie。改非受控（defaultValue + ref）让 set-value 直接生效。

## 改动
### web/src/pages/Settings.tsx
- 加 `useRef` import
- 加 `const cookieRef = useRef<HTMLTextAreaElement>(null)`
- cookie textarea: 去掉 `value={cookie}` 和 `onChange`，改 `ref={cookieRef} defaultValue={cookie}`
- `handleSave`: 读 `const cookieVal = cookieRef.current?.value ?? ''`，调 `set_opencode_cookie` 用 `cookieVal`
- 保留 `cookie` state（仅作初始加载值给 defaultValue，useEffect setCookie 加载已保存值）
- workspaceId 保持受控（已 work，不动）

## 验证
```bash
cd web && bun run build
```

## 注意
- 只改 cookie textarea，workspaceId 不动
- defaultValue 只首次渲染生效（React 非受控），cookie state 变不影响 textarea（但 useEffect 加载后 cookie state 用于 defaultValue）
- handleSave 读 ref.value（非 state）
