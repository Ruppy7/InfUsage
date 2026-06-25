# InfUsage handoff

## Current branch

```text
codex/tray-design-refresh
```

Remote:

```text
https://github.com/Ruppy7/InfUsage
```

## Dev workflow

Work from the native Windows project folder:

```powershell
cd path\to\InfUsage
git switch codex/tray-design-refresh
npm install
npm run tauri dev
```

## Locked decisions

- D1: Tauri v2.
- D2: React + TypeScript + Vite.
- D3: Rust inside Tauri; no sidecar/framework.
- D4: QuickJS via `rquickjs` for provider plugins.
- D6: JSON latest snapshots only; no usage history UI for now.
- D7: Windows Credential Manager via `keyring`.
- D8: OpenCode quota cookie paste path is the main experimental implementation.
- D9: OpenCode Go limits are the active app path; local SQLite device spend is archived optional/reference code.
- D10-D16: Current tray panel baseline is compact Focus/Dashboard mode with provider icons, status chips, global/per-provider refresh, optional periodic refresh, light/dark mode, and floating pop-out.

## Current app state

- Tauri app launches as a Windows tray utility.
- Tray icon appears; left-click toggles the window.
- Closing the window hides it; tray menu has Show and Quit.
- Popup positions near the bottom-right and avoids the taskbar.
- UI uses compact provider cards, status chips, provider icons, and icon buttons.
- Provider list scrolls inside the popup.
- OpenCode shows Go limits only in the app.

## Provider state

- DeepSeek balance works with one saved key in Windows Credential Manager and shows USD remaining.
- Codex reads local Codex auth, refreshes once when needed, and shows sanitized quota/reset fields.
- Claude reads local Claude Code credentials from the user's WSL/native setup, refreshes once when needed, and shows sanitized quota/reset fields.
- OpenCode quota uses a Credential Manager cookie path for the server-rendered `/workspace/{workspaceId}/go` data contract.
- Local OpenCode `opencode.db` spend is safer but this-device-only; it remains optional/reference code and is not called by the app.
- Antigravity is still pending.

## Next likely step

Continue Phase 6 design polish:

- refine tray icon assets
- tighten color, typography, spacing, and readable density
- review provider row hierarchy and empty/error/stale states
- package with `npm run tauri -- build`
