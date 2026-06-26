# Security Policy

## Supported Versions

Security fixes target the latest `main` branch and the latest GitHub Release.

## Reporting a Vulnerability

Please report security issues privately by opening a GitHub security advisory if available, or by contacting the maintainer directly from the GitHub profile.

Do not include live provider credentials, cookies, tokens, or personal account data in public issues.

## Security Expectations

LimitLens handles local auth files, API keys, and provider session data. The app is designed so secrets stay in the Rust host and Windows Credential Manager where possible.

Expected behavior:

- Provider credentials are not sent to React.
- Provider credentials are not written to snapshots.
- OpenCode cookies are stored in Windows Credential Manager.
- UI and plugin code receive sanitized usage summaries only.
- No telemetry is sent by LimitLens.

Unsigned Windows builds may show SmartScreen or unknown-publisher warnings until code signing is available.
