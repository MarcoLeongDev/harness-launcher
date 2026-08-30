## Context
The launcher is a thin Tauri/macOS shell around the DeepSeek Harness npm package. The harness runs as a child of a bundled Node sidecar. Current UX: an overlay panel injected into the harness page + tray menu. Long npm installs run silently; there is no engine start/stop; port changes navigate without server-side verification.

## Goals / Non-Goals
- Goals: dedicated Settings window with live operation progress; explicit engine lifecycle (start/stop/restart/force-restart); seamless, verified port change; tray + overlay updated similarly.
- Non-Goals: pausing/resuming harness state; process groups beyond node; alternative frontends.

## Decisions
- **Custom protocol `dsh-ui://`** for the settings window instead of `WebviewUrl::App`: the app has no static frontend (frontendDist is a stub, devUrl is the harness itself), so a registered URI scheme gives a stable local origin in dev and prod, with `window.__TAURI__` injected via withGlobalTauri.
- **`launcher://progress` event + `AppState.current_op`**: emitters are command threads; UI subscribes for live updates and can read the in-flight op from get_status when opening late.
- **Child id guard**: every spawn gets an incrementing id stored in ChildState; the supervisor thread only clears the process slot when the Terminated event matches the current id, so a fast force-restart cannot be clobbered by the old child's event.
- **Phase-based status**: HarnessStatus gains `phase` (stopped/starting/running/stopping) from HarnessRuntime; tray item enablement derives from it.
- **Force restart escalation**: kill the child, then poll the port up to ~1.5s; if still bound, kill PIDs from `lsof -ti tcp:<port>` with SIGKILL (macOS-only app, acceptable).
- **Seamless port change**: resolve -> persist desired -> restart with progress -> wait_until_serving -> navigate only on success; revert settings + effective port on failure.
- **npm progress**: run_npm gains an optional line sink; install_version emits throttled phase events (installing/verifying) and streams truncated npm lines.
- **start_on_launch setting**: engine_stop persists false so the engine stays stopped across app relaunch; engine_start/boot re-enable.

## Risks / Trade-offs
- Custom protocol pages have no CSP in config (csp: null); the page is local-only, same trust level as the overlay.
- lsof escalation is macOS-specific; app is macOS-only.
- Streaming every npm line could spam the event bus -> throttle to ~1 msg/s in the sink.

## Migration Plan
- Backward compatible: new commands additive; status payload adds fields; existing overlay still works (plus new sections).

## Open Questions
- Should force-restart also act when the engine is already stopped? (No: it restarts only if a process state exists or the port is still bound.)
