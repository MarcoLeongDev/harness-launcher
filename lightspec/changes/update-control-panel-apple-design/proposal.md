# Change: Apple-style Control Panel redesign (Engine, Downloads, Logs)

## Why
User feedback after the tabbed panel landed: the Engine/Port tab still repeats
port information in many places, engine controls are plain text buttons, version
downloads feel stuck ("faking it", impossibly long) with an unpolished layout,
and the Logs tab repeats the word "Logs" ("Harness Logs" label). The panel
should be icon-driven and minimal, designed as Apple would.

## What Changes
- Engine tab: single authoritative "Runs at http://127.0.0.1:<port>" line; port
  field shown once; SF-symbol-style iconography; prominent primary Start/Stop
  with subtle secondary Restart and muted Force controls; grouped, macOS-like
  rows (System Settings look).
- Version download: honest progress with elapsed time, npm command shown once,
  collapsible terminal details, faster installs via npm cache-friendly flags.
- Logs tab: no redundant title; minimal icon toolbar over the log output.
- Global: icon + minimal text, remove repeated labels.

## Impact
- Affected specs: `webui-integration` (panel design, tabs), `harness-runtime`
  (npm install flags for faster, cache-friendly downloads)
- Affected code: `src-tauri/resources/settings.html`, `src-tauri/src/versions.rs`
- Version: patch bumps per item (0.1.7 -> 0.1.9).
