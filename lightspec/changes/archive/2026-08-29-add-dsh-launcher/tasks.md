## 1. Project Scaffold
- [x] 1.1 Initialize source/ layout: src-tauri, scripts, resources, web-dist
- [x] 1.2 Add icons (app + tray template) and vendored npm/Node sidecar scripts
- [x] 1.3 Requirements authored in Lightspec (proposal, design, tasks, spec deltas)

## 2. Rust Shell
- [x] 2.1 Bootstrap: tray icon, ActivationPolicy Accessory, hidden-window lifecycle
- [x] 2.2 Settings persistence (port, auto-update flags, active/previous version)
- [x] 2.3 Tray menu (show/hide, restart harness, open browser, update, quit)

## 3. Harness Runtime Manager
- [x] 3.1 Runtime dir layout (runtime/versions/<v>, logs, npm-cache)
- [x] 3.2 Bundled node sidecar invocation + vendored npm (node npm-cli.js)
- [x] 3.3 npm view version listing (latest dist-tag, all versions, prerelease filter)
- [x] 3.4 Install/switch/rollback with per-version isolation; update-to-latest action
- [x] 3.5 Harness process spawn (dsh --profile web --no-open --port <port>), stop/restart, log capture

## 4. Port Service
- [x] 4.1 Port config, validation, persistence; free-port fallback; restart+reload on change

## 5. WebUI Window + Overlay Panel
- [x] 5.1 Window loads http://127.0.0.1:<port>, withGlobalTauri, capability grants
- [x] 5.2 Initialization script injecting bottom-left overlay panel
- [x] 5.3 IPC commands for all panel actions; wire panel UI to commands

## 6. Update Service
- [x] 6.1 Harness auto-update check at startup + configurable interval + notification
- [x] 6.2 Optional app self-update via tauri-plugin-updater when endpoint configured

## 7. Build, Verify, Document
- [x] 7.1 npm install CLI, prepare assets, cargo check/build clean
- [x] 7.2 Smoke-check runtime manager (install a version, list, switch, rollback, start/stop)
- [x] 7.3 README with build, run, update/rollback usage, signing/notarization notes
- [ ] 7.4 Archive the lightspec change after completion