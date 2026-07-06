# Feature Roadmap

This is the living tracker for LimitLens feature ideas, implementation plans, research notes, and completion status.
`PLAN.md` remains the source of truth for major architecture decisions and the decision log; this file tracks the product backlog and the path from idea to shipped feature.

## Status Key

- **Idea** - Captured but not evaluated.
- **Research** - Needs provider/API/UX verification before implementation.
- **Planned** - Direction is clear enough to implement.
- **In Progress** - Actively being built.
- **Implemented** - Shipped in the app.
- **Deferred** - Worth keeping, but not part of the near-term path.

## Current Direction

LimitLens should evolve from a tray quota viewer into a unified AI usage dashboard:

- A tiny draggable always-on-top **glance window** for high-priority limits.
- A resizable **dashboard window** for all-provider analytics, provider details, setup, history, and app settings.
- A normalized metric model that can separate exact provider-reported usage from inferred, estimated, or local-only usage.

## Near-Term Implementation Plan

### Development Workflow

**Status:** Active

LimitLens now uses a normal open-source feature flow:

- `main` is the stable integration branch.
- Build features on scoped branches with conventional prefixes: `feat/`, `fix/`, `docs/`, `chore/`, or `refactor/`.
- Do not create new `codex/` branches; older `codex/*` branches are historical only.
- Open a PR for each meaningful feature, provider integration, or architecture change.
- Treat the PR as the checkpoint: include summary, checks, screenshots for UI changes, and any decision notes.
- Merge after review/smoke testing, then start the next feature from updated `main`.
- Create GitHub Releases only from version tags such as `v0.1.1` or `v0.2.0`.

Versioning guidance:

- Patch releases (`v0.1.x`) are for fixes and small improvements.
- Minor releases (`v0.2.0`) are for larger public feature batches, such as the resizable dashboard or provider expansion.
- Feature branches are not releases; they are reviewable increments toward the next release.

### F1 - Draggable Glance Window

**Status:** Implemented

Goal: add a second, tiny always-visible window that acts as the new focus surface.

Initial display format:

```text
[provider icon] 34% | 62%
[provider icon] $0.85
```

Meaning:

- `34%` - current/session remaining percent.
- `62%` - weekly remaining percent.
- `$0.85` - direct balance for providers that do not expose session/weekly limits.

First slice:

- Windows taskbar overlay was rejected as too brittle; use a draggable always-on-top glance window instead. Implemented.
- Clicking the glance window opens the existing main popup/dashboard. Implemented.
- Store the setting locally as something like `limitlens.glanceEnabled`. Implemented.
- Reuse latest saved provider snapshots; do not create a separate refresh pipeline. Implemented.
- Show remaining percentage by default. Implemented.
- Support a draggable remembered position for the floating glance widget. Implemented.
- Show Codex, Claude, and OpenCode in a compact 2x2 layout sized for four providers. Implemented.
- Keep the display intentionally terse: provider icon, bold current/session remaining percent, divider, normal weekly remaining percent. Implemented.

Follow-up:

- Add glance provider priority settings.
- Render as many enabled providers as fit, in priority order.
- Drop lower-priority providers when space runs out instead of shrinking text into unreadability.

Possible future setting:

- Let users toggle between remaining percentage and consumed percentage.

### F2 - Dashboard-Only Main Window

**Status:** Deferred

Branch: `feat/dashboard-completion`

Goal: remove the current Focus view and make the main window the full dashboard/settings surface.

Rollback note:

- The first dashboard/sidebar layout did not work well enough and was rolled back to the v0.1 tray-panel structure.
- Preserve the glance window, Codex reset credits, Claude Fable 5, and Antigravity provider work.
- Revisit dashboard IA after a better design structure is chosen.

Plan:

- Make the main window resizable with a sensible minimum size. Implemented in first slice.
- Remove the main-window pin control and main-window always-on-top behavior. Implemented in first slice.
- Show the main dashboard as a normal taskbar-visible app window. Implemented in first slice.
- Replace the scaled-up tray card list with an app shell: top bar, provider sidebar, and main dashboard content grid. Implemented in first slice.
- Add an All view for cross-provider usage, token, model, and price summaries. Implemented as the current overview with analytics placeholders.
- Make provider sidebar clicks replace the main content with that provider's page. Implemented.
- Move provider-specific setup out of Settings and onto provider detail pages. Implemented.
- Add starred providers, sort them first in the sidebar, and use starred providers for the glance window when present. Implemented.
- Hide explicitly disconnected providers from the sidebar and All view. Implemented.
- Add a sidebar Add Provider menu for restoring explicitly disconnected providers. Rolled back with the dashboard layout.
- Keep compact behavior as a responsive small-window layout rather than a manual Focus mode.
- Keep tray icon click as the main dashboard launcher.
- Use responsive breakpoints: compact layouts can still behave like the old focus/dashboard views when the window is small, while larger sizes become a full app-style dashboard.
- Reserve the larger dashboard for analytics that need room: token spend, usage spend, reset banks, history, and provider setup.

Next slice:

- Pause dashboard implementation work.
- Keep the v0.1 compact tray panel as the app's main surface while a better dashboard design is worked out.
- Do not move provider setup out of Settings again until the new information architecture is clear.

Open decision:

- Exact breakpoint behavior and whether users should get a manual override later.
- How much of the existing compact provider rail remains after the dashboard shell matures.
- Whether Settings remains as a small app-preferences sheet or becomes a dedicated settings page once more customization exists.

### F3 - Glance Provider Priority

**Status:** Deferred

Goal: let users pick which providers appear in the glance window.

Model:

```text
provider_id
enabled_on_glance: boolean
priority: number
```

Rules:

- Higher-priority providers render first.
- The glance window renders only providers with enough usable quota fields.
- Providers with no current data are skipped or shown as a compact warning only if they are high priority.

Current slice:

- The glance window is preserved as a draggable always-on-top surface.
- It uses the default compact provider set for now.
- Starred-provider priority was rolled back with the dashboard/sidebar experiment.

Follow-up:

- Replace boolean stars with explicit glance priority/order once provider count grows.
- Expand Add Provider from "restore disconnected provider" into a full provider picker for newly supported providers.

## Metric Model Evolution

### F4 - Structured Provider Metrics

**Status:** Planned

Current app state: `ProviderSnapshot` is display-first, carrying `MetricLine { label, value }`.

Needed future state: `ProviderSnapshot` should carry display lines plus structured metrics.

Candidate shape:

```text
provider_id
display_name
plan
display_lines
quota_windows
token_usage
cost_usage
reset_banks
source_quality
refreshed_at
warning
```

This should be added gradually rather than in one large rewrite.

Why this matters:

- The dashboard can aggregate usage across providers.
- The glance window can read session/weekly values without parsing strings.
- The app can label data honestly as exact, estimated, inferred, or local-only.

### F5 - Source Quality Labels

**Status:** Planned

Every non-trivial metric should eventually carry source quality:

- **Exact** - Provider reports the value directly.
- **Provider-derived** - Computed from provider-reported fields.
- **Local-only** - Derived from local logs/databases and may miss other devices.
- **Estimated** - Computed from public pricing, token logs, or incomplete data.
- **Unavailable** - Provider does not expose the value.

Rule: do not aggregate estimated and exact usage without preserving the distinction in the UI.

## Provider Roadmap

### F6 - Provider Expansion Branch

**Status:** Implemented

Branch: `feat/provider-expansion`

Goal: add more providers without jumping straight into the full token/cost dashboard. Each provider should first deliver the most reliable quota/balance snapshot it can expose, then token/cost analytics can layer on top once the structured metric model is ready.

Merged in PR #5:

- Antigravity provider with language-server discovery and Credential Manager / Cloud Code fallback.
- Claude Fable 5 quota display.
- Codex reset credits and expiry dates.
- Glance support for three-value providers and Antigravity's Gemini Pro / Gemini Flash / Claude pools.

Research pass completed from local `robinebers/openusage` reference clone:

- OpenUsage provider docs reviewed: Antigravity, Cursor, Devin, Grok, Copilot, OpenRouter, and Z.ai.
- OpenUsage provider shape confirmed: auth store -> usage client -> mapper -> normalized provider snapshot.
- For LimitLens, keep the existing host/plugin boundary: Rust owns credential/API access, JS plugin normalizes sanitized host data into display lines.
- Add provider docs beside each implementation as providers land.

Recommended implementation order:

1. **Antigravity** - first because it is already in core scope and quota-only. Use the running-app language-server path first, then local Credential Manager token plus Cloud Code fallback.
2. **Cursor** - high user value and the next target after Antigravity. Start with live quota/credits; keep stale spend exports out of the first slice.
3. **Devin** - local CLI/app credential provider with daily/weekly quota and extra balance. Similar shape to the quota providers we already support.
4. **GitHub Copilot** - local editor/GitHub CLI token provider with premium/chat/completion quota. Useful, but only if Windows token paths are straightforward.
5. **Grok / xAI** - monthly credits plus optional local log token/cost spend. Defer until source-quality labels are in place for local/estimated spend.
6. **OpenRouter / Z.ai** - API-key providers. Implement after we generalize the current DeepSeek-specific key UI into reusable provider-page key management.
7. **Ollama** - defer until we decide what "usage" means for local models without subscription limits.

Provider integration categories:

| Category | Providers | First LimitLens behavior |
|---|---|---|
| Local app/language-server quota | Antigravity | Discover local app/server, call quota endpoint, cache last snapshot when unavailable |
| Local CLI/app credential quota | Devin, Copilot, Cursor, Grok | Read existing Windows credentials or local app state; refresh tokens only when safe and understood |
| User-supplied API key | DeepSeek, OpenRouter, Z.ai | Store keys in Windows Credential Manager and manage from provider pages |
| Local logs / estimated spend | Claude/Codex follow-ups, Grok, Cursor if restored | Label as local-only or estimated; do not aggregate as exact spend |
| Local runtime without subscription quota | Ollama | Research-only until token counting or request wrapping has a clear product meaning |

Next provider slice after dashboard completion:

- Cursor, starting with live quota/credits.
- Older stale spend exports remain deferred until source-quality labels exist.
- Add provider docs and parser/plugin tests beside the implementation.

### Current Providers

| Provider | Current Status | Near-Term Work |
|---|---|---|
| Codex | Implemented for session/weekly summary | Research reset banks, extra usage, local token spend, structured metrics |
| Claude / Claude Code | Implemented for quota summary, including Fable 5 when exposed | Research local token/cost spend via logs or ccusage-style tooling |
| DeepSeek | Implemented for API balance | Keep as balance provider; token usage only if a documented usage API exists |
| OpenCode Go | Implemented via experimental console cookie | Replace pasted cookie with app-owned session or upstream read-only API if possible |
| Antigravity | Implemented with fallback | Validate language-server and Credential Manager behavior against more Windows installs |

### Candidate Providers

| Provider | Status | Likely Data Source | Research Needed |
|---|---|---|---|
| Antigravity | Implemented with fallback | Local language-server first; Credential Manager / Cloud Code fallback | More Windows process/port/CSRF validation and response captures |
| Cursor | Next | Cursor app local state, dashboard endpoints, usage APIs | Credential source on Windows, token refresh, live usage fields, stale spend export behavior |
| Devin | Planned after Cursor | CLI credentials or app local state; `GetUserStatus` style quota endpoint | Windows credential/config paths and API server behavior |
| GitHub Copilot | Planned after Cursor | Local Copilot auth/session and quota APIs | Windows token paths and whether user plan exposes meaningful limits |
| xAI / Grok | Planned later | Grok CLI auth and billing endpoints; local logs for token spend | Windows CLI paths, token refresh, log format stability |
| OpenRouter | Planned after reusable key management | User-supplied API key and documented credit/spend endpoints | Adapt DeepSeek key flow into reusable provider-page key management |
| Z.ai | Planned after reusable key management | User-supplied API key and coding plan quotas | Validate API shape and relevance to LimitLens users |
| Ollama | Idea | Local runtime/API logs, model metadata | Define what "usage" means without subscription quota; token counts may require wrapping/proxying calls |
| Xiaomi MiMo | Deferred | Dashboard/private endpoints or token-plan API | Verify stable read path before any implementation |

## Unified Token and Cost Accounting

### F7 - Token Usage Dashboard

**Status:** Research

Goal: make LimitLens a single dashboard for token usage across subscriptions, models, and provider tools.

Required concepts:

- Provider
- Account/workspace
- Model
- Input tokens
- Output tokens
- Cache read/write tokens where available
- Requests
- Cost
- Time window
- Source quality

Key constraint:

Accurate token aggregation is only possible when the provider or local tool exposes trustworthy token counts. When the app estimates from logs or pricing manifests, the dashboard must label that clearly.

### F8 - Cost and Subscription Usage

**Status:** Research

Goal: show extra usage, pay-as-you-go usage, and provider-reported cost where available.

Rules:

- Prefer provider-reported cost over calculated cost.
- Calculated cost from public pricing is useful, but must be labeled as estimated.
- Subscription "value accounting" is subjective and should be deferred until raw usage and cost are solid.

## Codex Follow-Ups

### F9 - Codex Reset Banks

**Status:** Implemented

Goal: show available reset banks and expiry dates.

Expected display:

```text
Resets available: 2
Expiring: 12-Jul-26, 19-Jul-26
```

Implementation notes:

- Treat reset banks as their own structured metric, not as a normal session/weekly usage row.
- Dashboard can show expiry dates.
- Glance window can show a compact `B2` later if useful.
- Implemented for Codex as `Rate Limit Resets`, shown below the Weekly row.
- Count comes from `rate_limit_reset_credits.available_count` in the usage body, or from the dedicated reset-credit endpoint when available.
- Expiry dates come from `GET https://chatgpt.com/backend-api/wham/rate-limit-reset-credits` with the Codex desktop headers.
- If the dedicated endpoint fails, LimitLens falls back to the count only.

### F10 - Codex Local Token Spend

**Status:** Research

Goal: show Today, Yesterday, and Last 30 Days token/cost usage for Codex.

Research needed:

- Whether to reuse `ccusage` through an installed JS runner.
- Whether to parse logs directly in Rust.
- How Windows Codex logs differ from macOS/Linux examples.

Recommendation:

Start by researching the data shape, then decide between direct Rust parsing and invoking existing tooling.

## Upstream OpenUsage Research

Research date: 2026-06-29

Upstream project: [robinebers/openusage](https://github.com/robinebers/openusage)

Useful takeaways:

- OpenUsage currently supports Antigravity, Claude, Codex, Cursor, Devin, GitHub Copilot, Grok, OpenRouter, and Z.ai.
- Its current implementation is native macOS Swift/SwiftUI, not a drop-in Tauri/Rust implementation.
- The provider architecture is still worth copying conceptually: auth store, usage client, mapper, normalized provider snapshot.
- Its metric model is ahead of ours: progress meters, raw numeric values, badges, charts, reset expiry metadata, and source-aware display choices.
- Provider docs beside implementations are valuable and should be mirrored in LimitLens as providers grow.

Provider references:

- [OpenUsage README - Supported Providers](https://github.com/robinebers/openusage#supported-providers)
- [Adding a Provider](https://github.com/robinebers/openusage/blob/main/docs/adding-a-provider.md)
- [Codex provider docs](https://github.com/robinebers/openusage/blob/main/docs/providers/codex.md)
- [Antigravity provider docs](https://github.com/robinebers/openusage/blob/main/docs/providers/antigravity.md)
- [Cursor provider docs](https://github.com/robinebers/openusage/blob/main/docs/providers/cursor.md)
- [Devin provider docs](https://github.com/robinebers/openusage/blob/main/docs/providers/devin.md)
- [GitHub Copilot provider docs](https://github.com/robinebers/openusage/blob/main/docs/providers/copilot.md)
- [Grok provider docs](https://github.com/robinebers/openusage/blob/main/docs/providers/grok.md)

LimitLens reuse strategy:

- Do copy provider concepts, endpoint clues, metric vocabulary, parser test discipline, and provider documentation structure.
- Do not copy macOS-specific keychain, SwiftUI, Sparkle, menu bar, or app lifecycle code.
- Reimplement provider clients in Rust and keep secrets inside the trusted host.
- Preserve the QuickJS plugin boundary unless a provider clearly belongs as trusted host-only code.

## Suggested Build Order

1. Stabilize the restored v0.1 tray-panel structure with the preserved provider additions.
2. Revisit dashboard IA before rebuilding a full dashboard surface.
3. Add F4 structured provider metrics before deeper token/cost aggregation.
4. Add Cursor after dashboard direction is clearer, starting with live quota/credits and deferring stale spend exports.
5. Add provider docs and tests beside each new provider implementation.
6. Generalize API-key management before OpenRouter or Z.ai.
7. Add Grok only after source-quality labels can distinguish exact quota from local-only or estimated spend.
8. Add unified token/cost dashboard features only after multiple providers expose structured source-quality metadata.

## Parking Lot

- Global shortcut for dashboard open.
- Local loopback API for other tools to read usage.
- Provider-specific quick links.
- Notifications for expiring reset banks or high burn rate.
- Share/export cards.
- Full usage history after storage moves beyond latest snapshots.
