# Windows dev setup

Use Windows as the dev environment for this project. InfUsage is a Windows-native tray app.

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

## Run the app

From the project folder:

```powershell
cd C:\Users\rupes\Documents\InfUsage
git switch codex/tray-design-refresh
npm install
npm run tauri dev
```

## Tray checkpoint

Verify:

- tray icon appears
- left-click tray icon toggles the main window
- closing the window hides it
- tray menu `Show` restores it
- tray menu `Quit` exits
- popup positions near the bottom-right without going into the taskbar
