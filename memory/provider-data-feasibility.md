# Provider data feasibility

| Provider | Status | Notes |
|---|---|---|
| OpenAI Codex | Fragile | Reuse `~/.codex/auth.json`; undocumented usage endpoint. |
| Anthropic Claude / Claude Code | Fragile-works | Shared limits; reuse `~/.claude/.credentials.json`; undocumented usage endpoint plus local JSONL where useful. |
| OpenCode Go | Fragile-feasible | Embedded login; authenticated workspace `/go`; extract server-rendered rolling/weekly/monthly usage. Backlog upstream read-only usage API. |
| Antigravity (AGY) | Fragile-feasible | Local language-server integration; loopback `GetUserStatus`; stale cache when closed. |
| Xiaomi MiMo Token Plan Lite | Backlog optional | Known dashboard endpoints: `/tokenPlan/detail`, `/tokenPlan/usage`. Need sanitized response shapes, reset semantics, and `tp-…` key authorization test. |
| DeepSeek API balance | Solid optional | Official `/user/balance`; balance tracking only, not exact usage/spend. |

