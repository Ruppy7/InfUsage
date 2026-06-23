# Windows dev setup

Use Windows as the primary dev environment for this project. WSL is fine for reference/editing, but a Windows tray app must run as a Windows process.

## Required tools

Install Node.js if it is not already installed:

```powershell
winget install OpenJS.NodeJS.LTS
```

Install Rust:

```powershell
winget install Rustlang.Rustup
```

Install Microsoft C++ Build Tools if a Rust/Tauri build fails because MSVC, the linker, or the Windows SDK is missing:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

In the Visual Studio Build Tools installer, enable:

- Desktop development with C++
- MSVC build tools
- Windows SDK

Restart PowerShell, then verify:

```powershell
node -v
npm -v
rustc -V
cargo -V
```

## Test the app

Clone/pull the repo on the Windows side:

```powershell
cd C:\Users\rupes\Documents
git clone https://github.com/Ruppy7/InfUsage.git
cd InfUsage
git switch phase-1-shell
```

Run:

```powershell
npm install
npm run tauri dev
```

After future WSL/Codex changes are pushed:

```powershell
git pull
npm run tauri dev
```

## Phase 1 checkpoint

Verify:

- tray icon appears
- left-click tray icon toggles the main window
- closing the window hides it
- tray menu `Show` restores it
- tray menu `Quit` exits

## Why not WSL for tray testing?

Running Tauri from WSL targets Linux. That tests Linux tray/window behavior, not Windows tray/window behavior.

Use WSL only for:

- reference/editing if needed
- TypeScript checks
- frontend-only work

Use Windows for:

- tray behavior
- Windows Credential Manager
- Windows process discovery
- packaging
