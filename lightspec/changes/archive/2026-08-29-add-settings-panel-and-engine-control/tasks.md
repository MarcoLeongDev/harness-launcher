## 1. Progress Infrastructure
- [x] 1.1 Add progress.rs: ProgressPayload + emit helper writing AppState.current_op + event
- [x] 1.2 Extend get_status with engine_phase, current_op, start_on_launch

## 2. Runtime Lifecycle
- [x] 2.1 Add phase tracking + child id guard (Terminated must not clobber a newer child)
- [x] 2.2 Implement force_stop (kill + escalate via lsof for anything still bound)
- [x] 2.3 Add start_on_launch setting; boot honors it

## 3. Commands
- [x] 3.1 engine_start / engine_stop / engine_restart / engine_force_restart
- [x] 3.2 Progress wiring: install_version (npm line sink + phases), install_and_switch, update_to_latest, rollback
- [x] 3.3 Seamless set_port (validate/restart/serve-wait/revert)
- [x] 3.4 open_settings command

## 4. Settings Window + Protocol
- [x] 4.1 Register dsh-ui custom protocol; serve settings.html and stopped page
- [x] 4.2 Build settings.html control panel (status, engine, version+progress, port, auto-update, logs, actions)
- [x] 4.3 Capability file lists settings window

## 5. Tray + Overlay
- [x] 5.1 Tray: Settings item, Start/Stop/Restart/Force items with dynamic enable/disable
- [x] 5.2 Overlay: progress bar + engine buttons + dsh-ui skip guard

## 6. Test & Verify
- [x] 6.1 cargo check + unit tests (port fallback, semver, settings roundtrip)
- [x] 6.2 App smoke run: boot, status, engine start/stop/restart/force via IPC-safe paths
- [x] 6.3 node --check on overlay.js and settings.html inline scripts
- [x] 6.4 Validate lightspec change, update README