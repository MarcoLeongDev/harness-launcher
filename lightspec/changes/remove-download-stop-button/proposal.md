# Change: Remove the download stop button (minimal terminal design)

## Why
The stop button on the operation terminals adds control surface and code
(_stoppable_ payload flag, cancellation plumbing, per-surface buttons) for
little value: stuck downloads are already bounded by the npm timeout, and a
minimal terminal reads better without a cancel control.

## What Changes
- Control Panel: the circular stop button on operation feeds is removed
  (markup, CSS, handler); feeds keep header (command + elapsed) and output.
- Overlay floating panel: the terminal "Stop" button (lc-stopdown) is
  removed; the engine Start/Stop controls are untouched.
- Backend: the now-consumer-less cancellation machinery is removed — the
  `stoppable` progress flag, `cancel_operation` command and its registration,
  the per-op cancel set in AppState, and the cancel polling in `run_npm`.
  Stuck npm operations remain bounded by the existing run timeout.
- The stuck-watch hint no longer references a stop button.

## Impact
- Affected specs: webui-integration (feed component, animated feed, stop
  requirements removed)
- Affected code: src-tauri/resources/settings.html,
  src-tauri/resources/overlay.js, src-tauri/src/{progress,state,versions,
  commands,lib}.rs, resources/bootstrap-icons/stop-fill.svg (deleted),
  scripts/test-panel-feed.mjs.
- User data/content safety: UI/backend plumbing only; installs, settings and
  engine behaviour untouched.
- Version: patch bump 0.1.59 -> 0.1.60.