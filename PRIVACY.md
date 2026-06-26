# Privacy

LimitLens is a local Windows tray app. It does not include telemetry or analytics.

## What Stays Local

- Saved DeepSeek API key slots are stored through Windows Credential Manager.
- OpenCode Go session data is stored through Windows Credential Manager.
- Latest provider snapshots are stored in the app data folder as sanitized summaries.
- UI preferences are stored locally by the app.

## What Is Sent to Providers

LimitLens calls provider services only when you connect or refresh that provider.

- Codex and Claude use local CLI credentials to request usage/limit summaries.
- DeepSeek uses your saved API key to call the official balance endpoint.
- OpenCode Go uses your saved workspace/session details to request the Go limits page.

## What Is Not Collected

LimitLens does not collect analytics, usage telemetry, crash reports, or personal account data for the maintainer.

## Sensitive Data

Do not paste provider cookies, API keys, or auth files into public GitHub issues. If you need help, redact secrets before sharing logs or screenshots.
