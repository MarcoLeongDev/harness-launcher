# Tasks: control-panel-switch-preset-logs

## 1. Cross-version agent-preset compatibility (critical)
- [x] 1.1 presets.rs: enumerate an installed version's agent presets (alpha shipped root + rc config root)
- [x] 1.2 Repair invalid `agent-presets.default` in ~/.dsh/settings.yaml before engine start (code→standard)
- [x] 1.3 Legacy `code` preset shim in ~/.dsh/.agent-presets so rc-era sessions resume under alpha
- [x] 1.4 Wire into runtime::start; unit tests; v0.1.34

## 2. Live version switching (auto stop → switch → start)
- [x] 2.1 switch_version_inner: stop running engine, install, set default, start new version
- [x] 2.2 Engine card version dropdown selectable while running (switch = install_and_switch)
- [x] 2.3 Feedback messages + tests; v0.1.35

## 3. Download without stopping the engine
- [ ] 3.1 New install_version IPC command: download only, never switches or restarts
- [ ] 3.2 Install button uses download-only path; v0.1.36

## 4. Parallel downloads with reusable terminal component
- [ ] 4.1 progress.rs: per-op current_ops map, per-op console tags, per-op cancel
- [ ] 4.2 cancel_operation targets one op; run_npm honours per-op cancel
- [ ] 4.3 settings.html: TerminalFeed component, one feed per active download
- [ ] 4.4 Overlay compatibility (current_op preserved); v0.1.37

## 5. Header restyle
- [ ] 5.1 Subtle launcher version (no tag), engine version tag + power button right-aligned; v0.1.38

## 6. Remove footer version label
- [ ] 6.1 Remove footer harness version + wiring; v0.1.39

## 7. Logs tab: overlay controls + shared terminal styling
- [ ] 7.1 Go-to-last overlay bottom-right, refresh overlay top-right (circular, hover background)
- [ ] 7.2 Log fills panel height, resizes with window
- [ ] 7.3 Shared .terminal style for feeds + log; v0.1.40

## 8. Build, deploy & verify
- [ ] 8.1 cargo test green; lightspec validate pass
- [ ] 8.2 Release build + deploy to /Applications + relaunch; confirm new version live; leave app running
