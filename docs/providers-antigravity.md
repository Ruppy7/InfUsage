# Antigravity Provider Notes

## Current implementation slice

- Status: running-app integration plus local credential fallback.
- Source order: Antigravity first, Cursor second, then the rest of the provider backlog.
- Primary data source: a running Antigravity `language_server` process or `agy` process.
- Fallback data source: local `agy` / Antigravity token from Windows Credential Manager, service `gemini`, account `antigravity`.
  - On Windows this credential appears as legacy generic target `gemini:antigravity`; LimitLens tries the Rust keyring lookup first, then reads that known Credential Manager target directly.
- Discovery:
  - Scan Windows processes for `language_server` with Antigravity markers, then `agy`.
  - Extract `--csrf_token` and `--extension_server_port` when present.
  - Read listening TCP ports for the process and try the local language-server RPC endpoints.
- RPC methods:
  - Prefer `GetUserStatus` because it includes plan plus model quota data.
  - Fall back to `GetCommandModelConfigs` for quota data only.
- Cloud Code fallback:
  - Try `fetchAvailableModels` first for the full model quota set.
  - Try `loadCodeAssist` plus `retrieveUserQuota` if model fetch is unavailable.
  - Refresh the Google OAuth access token only after auth failure or when no access token is usable.
  - Do not persist refreshed access tokens; saved credentials remain in Windows Credential Manager.
  - Uses the same installed-app Google OAuth client values as Antigravity/OpenUsage. These are split in source only to avoid automated secret scanners; this is obfuscation, not a security boundary.
- Local language-server HTTPS:
  - The local Antigravity server can use a self-signed loopback certificate, so LimitLens accepts invalid certificates only for `https://127.0.0.1` language-server probes.
  - Plain `http://127.0.0.1` probes and all Cloud Code requests use a normal client.
- Display:
  - Plan, when available.
  - Remaining quota pools, not consumed usage.
  - OpenUsage-compatible pool order: Gemini Pro, Gemini Flash, Claude.
  - The primary `fetchAvailableModels` response drives the display when available; `retrieveUserQuota` remains fallback-only.

## Deferred work

- Validate behavior against more Antigravity versions and CLI process shapes.
- Validate whether Windows Antigravity also stores useful VS Code-style SQLite state. Current upstream OpenUsage has moved to the keychain/token path instead of the older SQLite envelope.
- Decide how the glance bar should label multi-pool providers once more than one non-session provider is available.
- Add token and price telemetry only after the provider limit path is stable.

## Reference

This implementation follows the relevant OpenUsage Antigravity ideas: running language-server discovery, `GetUserStatus`/`GetCommandModelConfigs`, and collapsing fine-grained model quota data into Gemini Pro, Gemini Flash, and Claude pools.
