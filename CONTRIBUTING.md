# Contributing

Thanks for helping improve LimitLens.

## Development

LimitLens is a Windows-first Tauri app.

```powershell
npm install
npm run tauri dev
```

Before opening a pull request:

```powershell
npm run build
cd src-tauri
cargo test
```

## Pull Requests

- Keep changes focused.
- Do not include credentials, cookies, tokens, logs, or local app data.
- Add or update tests for parser, security, storage, or provider behavior changes.
- Explain any provider endpoint assumptions, especially for undocumented APIs.

## Provider Integrations

Provider secrets must stay in the Rust host. Do not expose provider tokens, API keys, cookies, or raw auth files to React or plugin code.

Provider output should be sanitized into small usage summaries before it reaches the UI.
