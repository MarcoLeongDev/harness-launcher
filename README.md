# Harness Launcher

<p align="center">
  <img src="logo/DSH Launcher.png" alt="Harness Launcher" width="180" />
</p>

<h3 align="center">
  A polished macOS menubar app for managing the harness engine — install, version, start, stop — with zero terminal or browser fuss.
</h3>

<p align="center">
  <a href="#features"><img src="https://img.shields.io/badge/platform-macOS-333333?logo=apple&logoColor=white" alt="Platform: macOS" /></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/built%20with-Tauri%202-24c8db?logo=tauri&logoColor=white" alt="Built with Tauri 2" /></a>
  <img src="https://img.shields.io/badge/productivity-menubar%20app-22c55e" alt="Menubar app" />
  <img src="https://img.shields.io/badge/license-MIT-3b82f6" alt="License: MIT" />
</p>

<p align="center">
  <img src="logo/Screenshot.png" alt="Harness Launcher screenshot" width="820" />
</p>

---

**Harness Launcher** is a self-contained Tauri macOS menubar app that wraps the harness (`@deepseek-ai/dsh`) in an isolated environment. It manages the full lifecycle: install any published version, switch versions, start or stop the background engine, change ports, follow logs — all without ever opening a terminal or leaving the harness UI.

> **The engine runs in the background.** The launcher installs and manages the harness inside the app (bundled Node runtime + vendored npm) and serves the harness WebUI on `http://127.0.0.1:<port>`. Close the window — the engine keeps serving.

---

## Features

| | |
|---|---|
| 🚀 **Engine lifecycle** | Start / Stop / Restart / Force-Restart the background harness from the menu bar, floating panel, or Control Panel window. |
| 📦 **Version management** | Install, switch, or roll back to any published harness version — each isolated, offline-safe. |
| 🖥️ **Live download terminal** | Watch real `npm install` output with a **Stop** button to cancel. |
| 🗑️ **Delete versions** | Remove installed versions (the active one is protected). |
| 🌐 **Port selection** | Set any loopback port; the engine restarts on it with transparent fallback. |
| 📋 **Logs** | Follow the live harness log tail from the Control Panel. |
| 🧰 **No browser needed** | The harness UI opens in its own app window; "Open in Browser" is available when you want a tab. |
| 🔔 **Get Latest** | Check the registry for newer versions and see if a downloaded version is ready to switch to. |

---

## Quick Start

### Release build

```bash
cd source
npm install                # installs @tauri-apps/cli
npm run assets             # icons + bundled Node sidecar + vendored npm (network once)
npm run build              # tauri build → app bundle
```

### Manual deploy

```bash
npm run deploy             # build release + install into /Applications
npm run deploy:relaunch    # build, install, then restart the running instance
```

### Day-to-day

1. Launch **Harness Launcher** — menubar app, no Dock icon.
2. First launch installs the latest harness version in the background.
3. The engine starts and the harness UI opens in its own window.
4. Use the floating **Harness** panel (bottom-left) or the **Control Panel** to manage everything.

---

## How it works

A *thin, replaceable shell*: the harness (`@deepseek-ai/dsh`) is installed, updated, and rolled back **inside** the app with a bundled Node runtime and vendored npm. The Tauri binary never needs rebuilding when the harness releases — updates happen at a click.

```
~/Library/Application Support/ai.dsh.launcher/
├── settings.json          # port, start_on_launch, active/previous version
├── logs/launcher.log
└── runtime/
    ├── versions/<version>/   # isolated npm install per harness version
    ├── logs/harness.log      # harness stdout/stderr (rotated @ 2MB)
    └── npm-cache/            # vendored npm cache
```

---

## Development

```bash
cd source
npm run assets
npx tauri dev              # or: cargo run --manifest-path src-tauri/Cargo.toml
```

**Prerequisites:** macOS (Apple Silicon), Rust stable, Xcode Command Line Tools, Node.js 20+ (build-time only — the app is fully self-contained at runtime).

---

## Architecture

- `src-tauri/src/runtime.rs` — harness process lifecycle (spawn / stop / restart / supervise, phase tracking, log capture)
- `src-tauri/src/versions.rs` — npm version listing, install, rollback
- `src-tauri/src/progress.rs` — `launcher://progress` / `launcher://console` event streams
- `src-tauri/src/commands.rs` — IPC surface for overlay, Control Panel, and tray
- `src-tauri/src/window.rs` — harness window (+ overlay) and Control Panel (`dsh-ui://` protocol)
- `src-tauri/resources/settings.html` / `stopped.html` / `overlay.js` — UI surfaces

---

## License

MIT — see [LICENSE](LICENSE). The harness itself is a separate MIT-licensed project.
