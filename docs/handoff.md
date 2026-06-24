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
cd C:\Users\rupes\Documents\InfUsage
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
- D8: OpenCode quota cookie paste path is dev-only validation.
- D9: OpenCode local SQLite spend is primary, filtered to `providerID = "opencode-go"`.
- D10: Current tray panel baseline is a compact undecorated 400x540 popup with `lucide-react` icons.

## Current app state

- Tauri app launches as a Windows tray utility.
- Tray icon appears; left-click toggles the window.
- Closing the window hides it; tray menu has Show and Quit.
- Popup positions near the bottom-right and avoids the taskbar.
- UI uses compact provider cards, status chips, a custom draggable title bar, and icon buttons.
- Provider list scrolls inside the popup.
- OpenCode shows either Spend or Quota, never both at the same time.

## Provider state

- DeepSeek balance works with one saved key in Windows Credential Manager and shows USD remaining.
- Codex reads local Codex auth, refreshes once when needed, and shows sanitized quota/reset fields.
- Claude reads local Claude Code credentials from the user's WSL/native setup, refreshes once when needed, and shows sanitized quota/reset fields.
- OpenCode reads local `opencode.db` spend/tokens read-only, including WSL paths, filtered to OpenCode Go provider usage only.
- OpenCode quota has a dev-only Credential Manager cookie path for validating the server-rendered `/workspace/{workspaceId}/go` data contract.
- Antigravity is still pending.

## Next likely step

Continue Phase 6 design polish:

- refine tray icon assets
- tighten color, typography, spacing, and readable density
- review provider row hierarchy and empty/error/stale states
- then revisit Windows packaging
