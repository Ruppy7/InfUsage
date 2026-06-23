# Provider data feasibility

| Provider | Status | Notes |
|---|---|---|
| OpenAI Codex | Fragile | Reuse local Codex `auth.json`; undocumented `https://chatgpt.com/backend-api/wham/usage`; keep tokens in Rust and expose only sanitized summary JSON. |
| Anthropic Claude / Claude Code | Fragile-works | Shared limits; reuse local Claude Code `.credentials.json`; undocumented `https://api.anthropic.com/api/oauth/usage`; keep tokens in Rust and expose only sanitized summary JSON. |
| OpenCode Go | Fragile-feasible | Verified 2026-06-24: local `~/.local/share/opencode/auth.json` `opencode-go` key is a static `sk-…` **inference** key (`opencode.ai/zen/(go/)v1`) only — cannot read usage. Usage = console session-cookie auth: GET `https://opencode.ai/workspace/{workspaceId}/go` → `{ mine, useBalance, rollingUsage, weeklyUsage, monthlyUsage }`, each `*Usage` = `{ status, resetInSec, usagePercent }`. `lite.subscription.get` `/_server` RPC carries same data. Needs app-owned console session (PLAN D8). |
| Antigravity (AGY) | Fragile-feasible | Local language-server integration; loopback `GetUserStatus`; stale cache when closed. |
| Xiaomi MiMo Token Plan Lite | Backlog optional | Known dashboard endpoints: `/tokenPlan/detail`, `/tokenPlan/usage`. Need sanitized response shapes, reset semantics, and `tp-…` key authorization test. |
| DeepSeek API balance | Solid optional | Official `/user/balance`; balance tracking only, not exact usage/spend. |

