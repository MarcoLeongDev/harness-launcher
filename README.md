# DeepSeek Harness Launcher

<p align="center">
  <img src="logo/DSH Launcher.png" alt="DeepSeek Harness Launcher" width="180" />
</p>

<h3 align="center">
  Run, manage and version the <a href="https://github.com/deepseek-ai/deepseek-harness">DeepSeek Harness</a> engine — no browser, no terminal, no fuss.
</h3>

<p align="center">
  <a href="#features"><img src="https://img.shields.io/badge/platform-macOS-333333?logo=apple&logoColor=white" alt="Platform: macOS" /></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/built%20with-Tauri%202-24c8db?logo=tauri&logoColor=white" alt="Built with Tauri 2" /></a>
  <a href="https://github.com/deepseek-ai/deepseek-harness"><img src="https://img.shields.io/badge/manages-DeepSeek%20Harness-4f46e5" alt="Manages DeepSeek Harness" /></a>
  <img src="https://img.shields.io/badge/productivity-menubar%20app-22c55e" alt="Menubar app" />
  <img src="https://img.shields.io/badge/license-MIT-3b82f6" alt="License: MIT" />
</p>

<p align="center">
  <img src="logo/Screenshot.png" alt="DeepSeek Harness Launcher screenshot" width="820" />
</p>

---

**DeepSeek Harness Launcher** is a polished macOS menubar app that wraps the
[DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) (`dsh`) in a
self-contained Tauri shell. It encapsulates all the complexity of running an AI
harness engine: install, update, switch versions, start, stop — everything a
harness developer needs, without ever touching a terminal or keeping a
browser tab open.

> **The engine runs in the background.** The launcher installs and manages the
> harness inside the app (bundled Node runtime + vendored npm), serves the
> harness WebUI on `http://127.0.0.1:<port>`, and gives you a native menubar
> presence for the full lifecycle. Close the window — the engine keeps serving.

## ✨ Features

| | |
|---|---|
| 🚀 **Engine lifecycle** | Start / Stop / Restart / Force-Restart the background harness right from the menu bar, the floating in-page panel, or the Control Panel window. |
| 📦 **Version management** | Install & switch between any published `dsh` version, **update to latest**, or **roll back** to the previously installed version — each in its own isolated install, offline-safe. |
| 🖥️ **Live download terminal** | While a version is downloading you see the real `npm install` output in a terminal view — with a **Stop** button to cancel the download. |
| 🗑️ **Delete versions** | Remove installed harness versions you no longer need (the active version is protected). |
| 🌐 **Choose your port** | Set any loopback port — the harness restarts on it and the window follows automatically, with transparent fallback if the port is busy. |
| 🔔 **Auto-update** | Periodic harness update checks with a macOS notification (optional app self-update via the Tauri updater). |
| 📋 **Logs** | Follow the live harness log tail straight from the Control Panel. |
| 🧰 **No browser needed** | The harness WebUI opens in its own app window; “Open in Browser” is available when you *do* want a tab. |

## 🖼️ What it looks like

<p align="center">
  <img src="logo/Screenshot.png" alt="Launcher main window" width="820" />
</p>

The **Control Panel** (tray → `Control Panel…`) aggregates engine status, version
management, downloads (with terminal output and Stop), port configuration,
auto-update and logs. The **floating panel** in the bottom-left of the harness
UI gives quick access to every control without leaving your chat.

## 🚀 Quick start

### From a release build

```bash
cd source
npm install                # installs @tauri-apps/cli
npm run assets             # icons + bundled Node sidecar + vendored npm (network once)
npm run build              # tauri build → src-tauri/target/release/bundle/macos/DeepSeek Harness Launcher.app
```

### Manual deploy to /Applications

```bash
npm run deploy             # build release + install into /Applications
npm run deploy:relaunch    # build, install, then restart the running instance
```

### Day-to-day use

1. Launch **DeepSeek Harness Launcher** — it's a menubar app (no Dock icon).
2. On first launch it installs the latest harness version in the background.
3. The engine starts and the harness UI opens in its own window.
4. Use the floating **DSh panel** (bottom-left) or the **Control Panel** to
   start/stop, switch versions, watch downloads and change the port.

## 🔧 How it works

A *thin, replaceable shell*: the harness (`@deepseek-ai/dsh`) is installed,
updated, versioned and rolled back **inside** the app with a bundled Node
runtime and vendored npm. The Tauri binary never needs rebuilding when the
harness releases a new version — updates happen at the click of a button.

```
~/Library/Application Support/ai.dsh.launcher/
├── settings.json          # port, auto-update flags, start_on_launch, active/previous version
├── logs/launcher.log
└── runtime/
    ├── versions/<version>/   # one isolated npm install per harness version
    ├── logs/harness.log      # harness stdout/stderr (rotated @ 2MB)
    └── npm-cache/            # npm cache for the vendored npm
```

## 🧑‍💻 Development

```bash
cd source
npm run assets
npx tauri dev              # or: cargo run --manifest-path src-tauri/Cargo.toml
```

Prerequisites: **macOS** (Apple Silicon), **Rust** stable, Xcode Command Line
Tools, Node.js 20+ (build-time only — the app is fully self-contained at
runtime).

## 🏗️ Architecture

- `src-tauri/src/runtime.rs` — harness process lifecycle (spawn / stop /
  restart / force-restart / supervise, phase tracking, log capture)
- `src-tauri/src/versions.rs` — npm version listing, install, rollback
- `src-tauri/src/progress.rs` — `launcher://progress` / `launcher://console`
  event streams + in-flight operation state
- `src-tauri/src/commands.rs` — IPC surface for the overlay, Control Panel
  and tray (including `cancel_operation` and `delete_version`)
- `src-tauri/src/window.rs` — harness window (+ injected overlay panel) and
  Control Panel window (`dsh-ui://` custom protocol)
- `src-tauri/resources/settings.html` / `stopped.html` / `overlay.js` —
  Control Panel, engine-stopped page and in-page control panel

## 📜 License

MIT — see [LICENSE](LICENSE). The harness itself is a separate MIT-licensed project.