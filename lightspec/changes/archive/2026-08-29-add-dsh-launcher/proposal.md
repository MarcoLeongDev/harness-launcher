# Change: DSH Launcher — Tauri macOS wrapper around DeepSeek Harness

## Why
DeepSeek Harness ships as an npm package (`@deepseek-ai/dsh`) whose web profile serves a browser UI on http://127.0.0.1:<port> (default 3080). This change builds a macOS menubar app that wraps the harness: bundled Node + npm, version management (update/rollback/select any published version, default latest), selectable localhost port, embedded WebUI with a bottom-left control panel, and tray-only lifecycle. The Tauri layer stays a thin replaceable shell so harness upgrades never require rebuilding the app.

## What Changes
- New Tauri v2 macOS app "DSh Launcher" (all source under source/).
- Menubar-only lifecycle: tray icon, no Dock item, keeps running when the window closes; quit only from the tray.
- Bundled Node sidecar + vendored npm CLI: harness install/update/rollback run with npm/npx inside the app.
- Per-version isolated installs under the app data dir, active-version pointer, start/stop/restart of `dsh --profile web --no-open --port <port>`.
- Version discovery from the npm registry; default = latest dist-tag; prerelease filter.
- Selectable loopback port (default 3080), persisted, free-port fallback, restart on change.
- WebUI in a Tauri window with an injected bottom-left overlay panel (status, version select/update/rollback, port, auto-update, logs, open-in-browser, quit).
- Optional app self-update via tauri-plugin-updater only when an endpoint is configured; harness auto-update check with tray notification.
- Settings persisted (port, auto-update flags, active version).

## Impact
- Affected specs: `harness-runtime`, `menubar-app-shell`, `webui-integration`, `port-service`, `update-service` (all new).
- Affected code: everything under source/ (src-tauri Rust shell, scripts, resources, lightspec requirements).
- Non-goals: harness internals changes; non-macOS targets in v1; non-loopback exposure.
