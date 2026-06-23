# InfUsage

Windows-native system-tray app for tracking AI inference usage and spend.

## Current status

- D1 shell: Tauri v2.
- D2 frontend: React + TypeScript + Vite.
- D3 backend: Rust inside Tauri.
- Scaffold: official Tauri React TypeScript template using npm.
- Active branch: `phase-1-shell`.

Core providers planned:

- OpenAI Codex
- Anthropic Claude / Claude Code
- OpenCode Go
- Antigravity (AGY)

Optional/backlog:

- Xiaomi MiMo Token Plan Lite
- DeepSeek API balance tracking

## Development

Run from the project folder:

```powershell
cd C:\Users\rupes\Documents\InfUsage
git switch phase-1-shell
npm install
npm run tauri dev
```

Install dependencies:

```bash
npm install
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

- `PLAN.md` — scope, phases, and decision log.
- `docs/handoff.md` — current branch/status handoff.
- `AGENTS.md` — collaboration rules for AI agents.
- `memory/` — condensed project facts.

## License

MIT
