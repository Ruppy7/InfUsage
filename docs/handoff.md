# InfUsage handoff

## Current branch

```text
phase-1-shell
```

Remote:

```text
https://github.com/Ruppy7/InfUsage
```

## Dev workflow

Work from the native Windows project folder:

```powershell
cd C:\Users\rupes\Documents\InfUsage
git switch phase-1-shell
npm install
npm run tauri dev
```

## Locked decisions

- D1: Tauri v2.
- D2: React + TypeScript + Vite.
- D3: Rust inside Tauri; no sidecar/framework.

Pending decisions:

- D4 plugin runtime.
- D5 state management.
- D6 storage.
- D7 secret storage.

## Phase 1 status

Passed on Windows:

- Tauri app launches.
- Tray icon appears.
- Left-click tray icon toggles the window.
- Closing the window hides it.
- Tray menu `Show` restores it.
- Tray menu `Quit` exits.
- Static provider panel renders on `phase-1-shell`.

Current UI:

- Static tray-panel style window.
- Provider placeholders:
  - Codex
  - Claude / Claude Code
  - OpenCode Go
  - Antigravity
- Footer says no providers are connected.

## Next likely step

Make the current window behave more like a tray popup:

- smaller default size
- non-resizable
- no maximize button if Tauri/Windows supports it cleanly
- position near bottom-right on tray click

Skip for now:

- settings window
- global shortcuts
- updater/autostart
- plugin runtime
- provider integrations
- state library
