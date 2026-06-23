# Provider data feasibility

| Provider | Status | Notes |
|---|---|---|
| OpenAI Codex | Fragile | Reuse local Codex `auth.json`; undocumented `https://chatgpt.com/backend-api/wham/usage`; keep tokens in Rust and expose only sanitized summary JSON. |
| Anthropic Claude / Claude Code | Fragile-works | Shared limits; reuse local Claude Code `.credentials.json`; undocumented `https://api.anthropic.com/api/oauth/usage`; keep tokens in Rust and expose only sanitized summary JSON. |
| OpenCode Go | Fragile-feasible | Later path: app-owned OpenCode browser session, then authenticated `/_server` SolidStart RPC for `lite.subscription.get` / Go quota fields. Avoid visible DOM/page-text parsing except as manual probe. Backlog upstream read-only usage API. |
| Antigravity (AGY) | Fragile-feasible | Local language-server integration; loopback `GetUserStatus`; stale cache when closed. |
| Xiaomi MiMo Token Plan Lite | Backlog optional | Known dashboard endpoints: `/tokenPlan/detail`, `/tokenPlan/usage`. Need sanitized response shapes, reset semantics, and `tp-…` key authorization test. |
| DeepSeek API balance | Solid optional | Official `/user/balance`; balance tracking only, not exact usage/spend. |

