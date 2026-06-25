# InfUsage

Windows-native system-tray app for tracking AI inference usage and limits.

## Current status

- D1 shell: Tauri v2.
- D2 frontend: React + TypeScript + Vite.
- D3 backend: Rust inside Tauri.
- D4 plugin runtime: QuickJS via `rquickjs`.
- D6 storage: JSON file with latest provider snapshots only.
- D7 secrets: Windows Credential Manager via `keyring`.
- Active branch: `codex/tray-design-refresh`.

Current provider state:

- DeepSeek: optional balance provider, one saved key, USD remaining only.
- OpenAI Codex: local Codex auth plus sanitized quota summary.
- Anthropic Claude / Claude Code: local Claude Code auth plus sanitized quota summary.
- OpenCode Go: experimental cookie-backed quota path for authenticated Go limits.
- Antigravity (AGY): pending.

Optional/backlog:

- Xiaomi MiMo Token Plan Lite

## OpenCode Go usage

InfUsage shows OpenCode Go limits from the authenticated Go page after you link your browser session. This is the main OpenCode path today, but it is still marked experimental because it depends on OpenCode's web page data shape and session cookie behavior.

To view the source page yourself, open:

```text
https://opencode.ai/workspace/<your-workspace-id>/go
```

The workspace id starts with `wrk_` and is visible in the OpenCode URL.

To link it in the app, open Settings, paste either the workspace URL or `wrk_...` id, then paste the `Cookie` request header from your logged-in browser request to that Go page. The cookie is stored in Windows Credential Manager and is only used by the Rust host to fetch sanitized quota fields.

Safer fallback: the repo still contains a read-only local `opencode.db` device-spend reader in `src-tauri/src/providers/opencode_db.rs`. It does not require a web session cookie, but it is less accurate for quota because it only sees local device spend, especially if you use OpenCode from WSL or other machines. It is optional reference code and is not shown or called by the current app UI.

## Development

Run from the project folder:

```powershell
cd path\to\InfUsage
git switch codex/tray-design-refresh
npm install
npm run tauri dev
```

Build the web frontend:

```bash
npm run build
```

Run the Tauri desktop app in development:

```bash
npm run tauri dev
```

Build a Windows app and installer:

```bash
npm run tauri -- build
```

Release artifacts are written under:

```text
src-tauri\target\release\infusage.exe
src-tauri\target\release\bundle\msi\
src-tauri\target\release\bundle\nsis\
```

`npm run tauri dev` requires Rust/Cargo and OS-specific Tauri prerequisites.

For Windows setup, see [docs/windows-dev-setup.md](docs/windows-dev-setup.md).

## Project docs

- `PLAN.md` - scope, phases, and decision log.
- `docs/handoff.md` - current branch/status handoff.
- `AGENTS.md` - collaboration rules for AI agents.
- `memory/` - durable project facts.

## License

MIT
