# Change: Fix stuck-download message after completed updates + credentials schema fix

## Why
1. The panel shows "Checking npm registry for latest version… no progress for 60s — still working?
   tap the stop button to cancel" even when the download completed and the engine is running.
   Root cause: `update_to_latest` emits the registry phase under the generic op key `update`,
   then switches mid-flight to the versioned key `update:<version>` and `clear_op`s the old key.
   `clear_op` emits nothing to the UI, so the `update` feed never receives a terminal phase:
   it stays `downloading` forever, the 60s stuck-watch fires against it, and its stop button
   cancels an operation key nothing is running under.
2. The engine (0.1.2-rc.1) fails to boot with
   `credentials-local: ~/.dsh/.credentials.yaml declares version "1"; this build reads version 1`.
   The document schema version must be the NUMBER 1; the file had the string "1".

## Changes
- Backend: keep the whole update flow under the single `update` op key (no mid-flight switch).
- Ops: migrate ~/.dsh/.credentials.yaml top-level `version` to the number 1 (values untouched).
- Frontend: regression test simulating the full progress lifecycle with virtual timers
  (stuck message must appear only while genuinely idle-and-downloading, never after done).
