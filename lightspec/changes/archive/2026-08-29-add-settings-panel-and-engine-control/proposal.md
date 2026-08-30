# Change: Settings Panel Window with Engine Control and Operation Progress

## Why
The launcher today gives the user no way to see what a version install/update is doing (operations run silently until they finish), no explicit start/stop control over the background harness engine, and no dedicated place to manage port/runtime settings outside the small harness-page overlay. Power users need a full control panel and live progress.

## What Changes
- **New Settings window** (native Tauri window, served over a custom `dsh-ui://` protocol) aggregating: harness status, engine control (start / stop / restart / **force-restart**), version & update management with **live progress**, port change, auto-update settings, logs, and global actions.
- **Live operation progress**: a `launcher://progress` event stream with phase, message and optional percent for install / update / rollback / engine / port operations; the in-flight operation is exposed in `get_status` so late-opening UIs can attach.
- **Engine lifecycle commands**: `engine_start`, `engine_stop`, `engine_restart`, `engine_force_restart`; stop is persistent via a new `start_on_launch` setting; a forced restart escalates to killing anything still bound to the port after the child is killed.
- **Seamless port change**: `set_port` now validates, restarts with progress, waits for the harness to actually serve on the new port before navigating the window, and transparently reverts on failure.
- Engine status becomes a phase (`stopped`/\`starting`/\`running`/\`stopping`) surfaced in status payloads, tray menu (dynamic enable/disable of Start/Stop), overlay, and the settings window.
- When the engine is stopped and the main window previously showed the harness UI, the window redirects to a launcher-provided **stopped page** (Start / Open Settings) instead of a dead connection error.
- Overlay panel gains a progress bar, phase line, and Start/Stop/Force buttons; it self-skips on `dsh-ui:` pages.

## Impact
- Affected specs: `harness-runtime`, `webui-integration`, `menubar-app-shell`, `port-service`, `update-service`.
- Affected code: `src-tauri/src/{runtime,commands,state,settings,tray,window,lib}.rs`, new `src-tauri/src/progress.rs`, `src-tauri/resources/settings.html`, `src-tauri/resources/overlay.js`, `src-tauri/capabilities/default.json`.
- Non-goals: autostart at login; multiple simultaneous engine instances; non-macOS targets.
