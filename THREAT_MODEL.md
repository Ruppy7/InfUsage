# Threat Model

This is a lightweight security model for LimitLens.

## Assets

- Provider access tokens and refresh tokens.
- DeepSeek API keys.
- OpenCode Go session cookies.
- Sanitized usage snapshots.
- Local app settings.

## Trust Boundaries

- Rust host: trusted. Reads credentials, calls providers, stores secrets.
- React UI: untrusted for secrets. Receives sanitized summaries only.
- JavaScript provider plugins: sandboxed guest code. Access only injected `ctx.host` capabilities.
- Provider APIs/pages: external and potentially changing.

## Main Risks

- Secret leakage into UI state, snapshots, logs, errors, or screenshots.
- OpenCode cookie compromise.
- Undocumented provider endpoint changes.
- Malicious or malformed provider responses.
- Plugin sandbox escape or excessive resource use.
- Unsigned installer trust warnings and binary provenance concerns.

## Current Mitigations

- Secrets stay in Rust host code.
- API keys and OpenCode session data use Windows Credential Manager.
- Provider output is normalized into small metric lines.
- Plugin runtime has timeout, memory, stack, and output limits.
- Latest snapshots store sanitized summaries only.
- GitHub Actions release artifacts will include SHA256 checksums.

## Non-Goals For Now

- Defending against malware already running as the same Windows user.
- Guaranteed stability of undocumented provider endpoints.
- Code-signed Windows installer before a certificate is available.
