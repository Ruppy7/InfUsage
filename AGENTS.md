# AGENTS.md — InfUsage

## What this project is

**InfUsage** is a Windows-native system-tray app that tracks AI inference usage and spend. The initial product targets OpenAI Codex, Anthropic Claude/Claude Code (shared usage limits), OpenCode Go, and Antigravity (AGY), with backlog-optional Xiaomi MiMo Token Plan Lite and optional DeepSeek API balance integrations. Fork of / inspired by [openusage](https://github.com/robinebers/openusage). It is **both a real app the user will use and a learning project** — read `PLAN.md`, the source of truth for scope, phases, and the decision log.

## How to work on this project (important)

This is a **pair-programming learning project**. The user knows JS/React/Python and some architecture but is not a full-time coder. The AI writes the code; **the user drives the decisions and learns the reasoning.** Optimize for the user's understanding, not just working code.

The session rhythm:

```text
Decision → Concept → Build → Checkpoint
```

Rules:

- **No silent technical choices.** Before writing code for something, have the decision conversation: name 2–3 real alternatives, give a recommendation **with reasoning**, let the user overrule.
- Tech choices are treated as **open and re-derived together**, even where `PLAN.md` shows a leaning.
- Surface **key concepts in context** and connect them to transferable ideas.
- **Rust = read-fluency.** Annotate unfamiliar syntax inline the first few times, with JS/Python analogies.
- Keep the decision log in `PLAN.md` updated as choices land.

## Current stack decisions

- **D1 Shell:** Tauri v2.
- **D2 Frontend:** React + TypeScript + Vite.
- **Package manager:** npm, because it is installed in the current environment while pnpm/yarn are not.

## Leaning stack, still to be decided

- **Backend:** Rust via Tauri.
- **Plugin runtime:** QuickJS via `rquickjs`.
- **Storage:** SQLite.
- **Secrets:** Windows Credential Manager.
- **HTTP:** `reqwest`.

## Architectural centerpiece

A sandboxed **plugin system**: each provider is a self-contained `.js` plugin that can only touch the system through an injected `ctx.host` object (capability-based security). Understanding the **host/guest boundary** is the core learning goal of the project.

## Key locations

- `PLAN.md` — full plan, phases, decision log, concept index.
- `memory/` — durable facts about the user, working style, and project.
- `src-tauri/` — Rust backend.
- `src/` — React frontend.

