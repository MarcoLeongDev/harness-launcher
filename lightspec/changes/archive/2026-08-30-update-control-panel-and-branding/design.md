## Context
Control Panel (`settings.html`) and overlay are served from the launcher's `dsh-ui://` custom protocol; all IPC goes through `__TAURI__.core.invoke`. The current panel is a single scroll with many labeled sections; the engine status has a bug where a quiet harness is reported stopped after ~30s of no output.

## Goals / Non-Goals
- Goals: logo-derived app/tray/brand assets; accurate engine status; tabbed, visual panel; header logo.
- Non-Goals: redesigning the in-harness overlay panel; changing the engine commands.

## Decisions
- Decision: use `sips` (macOS built-in) for all PNG decode/resize in `gen-icons.mjs`; parse a sips-produced BMP in Node to build the monochrome tray template glyph (alpha = luminance inversion, rgb=0). Alternatives considered: fixing the hand-rolled decoder (root cause fragile), adding node-canvas / sharp dependency (new heavy deps — rejected).
- Decision: supervisor loop treats `RecvTimeoutError::Timeout` as a heartbeat (keep waiting), and only `Terminated` / `Disconnected` end the child tracking.
- Decision: `settings.html` uses ARIA tabs (role=tablist/tab/tabpanel), CSS-only visuals, and keeps the existing command wiring.

## Risks / Trade-offs
- sips availability: guaranteed on macOS (app is macOS-only) -> no risk.
- Template glyph fidelity: threshold-driven silhouette of the logo mark; low risk, visually matches the mark.

## Migration Plan
Rebuild assets, bump versions, redeploy; running instance keeps working until relaunch.
