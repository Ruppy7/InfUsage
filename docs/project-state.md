# Project State

Last checked: 2026-07-06

## Current baseline

- Product name: LimitLens.
- GitHub repo: `https://github.com/Ruppy7/LimitLens`.
- Main development branch: `main`.
- Current merged commit: `7ca4f55` (`Add provider expansion updates (#5)`).
- Active feature branch: `feat/dashboard-completion`.
- Stack: Tauri v2, React, TypeScript, Vite, Rust.
- Package manager: npm.
- Distribution: unsigned Windows NSIS installer, portable zip, and SHA256 checksums through GitHub Releases.
- Latest public release: `v0.1.0`.
- Target release for this branch: `v0.1.1`.

## App state

- Windows tray app with the v0.1 compact undecorated tray-panel structure restored.
- The dashboard/sidebar/provider-page experiment was rolled back on `feat/dashboard-completion`.
- A draggable always-on-top glance window is implemented; it shows compact remaining quota values and opens the main dashboard on click.
- Provider cards, status chips, per-provider refresh, global refresh, optional periodic refresh, theme setting, Focus/Dashboard size toggle, Settings provider setup, and pop-out pin behavior are restored.
- Provider-specific setup lives in the global Settings sheet again.
- The main tray panel is hidden from the normal taskbar; the glance window remains skipped from the normal taskbar.

## Provider state

- Codex: reads Windows-native Codex auth and shows quota/reset summary plus rate-limit reset credits with expiry dates when the dedicated endpoint is available.
- Claude: reads Windows-native Claude credentials and shows session, weekly, and Fable 5 quota/reset summary when exposed.
- DeepSeek: optional API balance check via a saved key in Windows Credential Manager.
- OpenCode Go: experimental cookie-backed quota flow against the authenticated workspace page.
- OpenCode local SQLite spend: documented fallback idea only, not shipped app code.
- Antigravity: running-app language-server path plus Windows Credential Manager / Cloud Code fallback implemented and merged in PR #5.

## Provider expansion state

- PR #5 (`feat/provider-expansion`) is merged into `main`.
- Local OpenUsage reference docs were reviewed for Antigravity, Cursor, Devin, GitHub Copilot, Grok, OpenRouter, and Z.ai.
- Antigravity is implemented with running-app discovery, local Credential Manager fallback, Cloud Code fallback, and OpenUsage-compatible Gemini Pro / Gemini Flash / Claude pools.
- Codex reset credits and expiry dates are implemented.
- Claude Fable 5 quota display is implemented when exposed by Claude's usage response.
- Further provider additions are intentionally paused while the dashboard surface is completed.
- Future provider order remains Cursor first, then Devin/Copilot/Grok/API-key providers as data quality allows.
- Cursor's first slice should focus on live quota/credits; stale spend export and local/estimated token-cost analytics remain deferred until source-quality labels are in place.
- OpenRouter and Z.ai are deferred until DeepSeek's provider-page key flow is generalized into reusable API-key provider management.

## Active dashboard agenda

- Current branch: `feat/dashboard-completion`.
- Goal: roll back the unsuccessful dashboard layout while preserving the useful provider/glance features.
- Preserved features: Codex reset credits and expiry display, Claude Fable 5 limits, Antigravity provider, and glance window.
- Dashboard IA is paused until a better structure is designed.

## Recent cleanup

- `v0.1.0` is released and public.
- LinkedIn launch post is done.
- Audit follow-up PR #2 is merged.
- Completed local/remote work branches were deleted after the v0.1 cleanup; new feature work now happens on conventionally named branches such as `feat/<feature-name>`, `fix/<bug-name>`, `docs/<topic>`, `chore/<task>`, or `refactor/<area>`.
- CSP no longer allows inline styles.
- Snapshot writes are atomic.
- Provider 429 responses return explicit rate-limit messages.

## Branch, PR, and release workflow

- `main` is the stable integration branch.
- Feature work should happen on scoped branches such as `feat/resizable-dashboard`.
- Do not create new `codex/` branches; keep that prefix historical only.
- Each meaningful feature or architecture change gets a PR into `main`; the PR is the review checkpoint with summary, checks, and screenshots where useful.
- Versioned public builds should come from tags such as `v0.1.1` or `v0.2.0`, not directly from feature branches.
- Patch releases (`v0.1.x`) are for fixes and small safe improvements. Minor releases (`v0.2.0`) are for larger user-facing batches such as dashboard evolution or provider expansion.

## Documentation scope

`PLAN.md` and `docs/feature-roadmap.md` are tracked planning docs as of the glance-window branch. `AGENTS.md`, `memory/`, and some other local notes may remain ignored or local-only. Public-facing information belongs in `README.md`, `SECURITY.md`, `PRIVACY.md`, `THREAT_MODEL.md`, and release notes.
