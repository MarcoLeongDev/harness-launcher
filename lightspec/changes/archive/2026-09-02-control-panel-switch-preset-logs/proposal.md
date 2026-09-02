# Change: Cross-version preset compatibility, live switching, and control-panel polish

## Why
Three functional problems and several UX defects were reported after running
alpha engine versions from the launcher:

1. **Sessions break after switching engine versions (critical).** rc engines
   shipped an agent preset named `code`; alpha engines ship `standard, ptc,
   minimal, cordis` instead. The harness home (`~/.dsh`) keeps
   `agent-presets.default: code` in `settings.yaml`, and every session created
   under rc records `agentPreset: code`. Under an alpha engine the default
   preset no longer resolves, so NEW sessions fail, and resuming any session
   created under rc fails with `agent-presets: preset "code" not found`. The
   user is effectively locked out of upgrading.
2. **Switching versions refuses to act while the engine runs**
   ("stop the engine before switching versions"). The correct UX is to stop the
   running engine, switch, and start the new version automatically.
3. **Downloads fight over one terminal.** Concurrent version downloads share a
   single output div, reset each other's console, and share one cancel flag, so
   parallel downloads cannot be tracked or cancelled independently.
4. **Downloads require a stopped engine.** Downloading is just downloading: it
   must work while the engine runs and must not switch or restart anything.
5. UI polish: bottom version label duplicates the header; header version tags
   need restyling (subtle app version, engine tag + power right-aligned); the
   Logs tab needs overlay controls (go-to-last bottom-right, refresh top-right,
   circular, hover-only background), a log that fills the panel and resizes with
   the window, and shared terminal styling.

## What Changes
- **Preset compatibility on engine start** (new `presets.rs`): before spawning
  an engine, inspect the installed version's available agent presets and (a)
  repair an invalid `agent-presets.default` in `~/.dsh/settings.yaml`
  (`code` → `standard` when available), and (b) install a user-root preset
  shim `~/.dsh/.agent-presets/code` (copy of the version's `standard`
  composition) so sessions recorded under rc resume. No user data is deleted.
- **Live version switching**: `install_and_switch` stops a running engine,
  installs/switches, and starts the new version again; the engine card version
  dropdown works while running. Feedback text describes the outcome.
- **Install ≠ switch**: a new `install_version` IPC command downloads a version
  without touching the active version or the running engine.
- **Per-operation progress**: progress payloads are tracked per operation key
  (`op:version`), console lines are tagged with their operation, cancellation
  is per operation, and the Control Panel renders one reusable terminal feed per
  active download.
- **Control Panel polish**: header version restyle + right alignment, footer
  version label removed, Logs tab overlay controls, full-height resizable log,
  and a shared `.terminal` style used by both the operation feeds and the log.

## Impact
- Affected specs: harness-runtime (engine start preset compatibility),
  update-service (install/switch semantics), webui-integration (control panel).
- Affected code: src-tauri/src/{presets.rs,commands.rs,progress.rs,runtime.rs,
  state.rs,versions.rs,lib.rs}, src-tauri/resources/settings.html.
