# InfUsage

Windows-native system-tray app for tracking AI inference usage and spend.

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
- OpenCode Go: local WSL/Windows `opencode.db` spend filtered to `opencode-go`; dev-only quota cookie validation path.
- Antigravity (AGY): pending.

Optional/backlog:

- Xiaomi MiMo Token Plan Lite

## Development

Run from the project folder:

```powershell
cd C:\Users\rupes\Documents\InfUsage
git switch codex/tray-design-refresh
npm install
npm run tauri dev
```

Build the web frontend:

```bash
npm run build
```

Run the Tauri desktop app:

```bash
npm run tauri dev
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
