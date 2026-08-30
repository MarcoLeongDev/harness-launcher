# Change: Control Panel redesign, branding and engine-status fix

## Why
User feedback on the deployed launcher:
- the app icon and menu-bar icon do not show the provided logo (the icon pipeline
  decodes the logo into garbage — black/transparent assets);
- the Control Panel reports the engine as stopped while the harness WebUI is
  actually running;
- the Control Panel is text-heavy and not organized; it should use tabs
  (Engine / Versions / Logs) and a cleaner design;
- the Control Panel does not display the provided logo.

## What Changes
- Rework the icon pipeline (`scripts/gen-icons.mjs`) to use macOS `sips`
  instead of the broken hand-rolled PNG decoder; derive the app icon,
  menu-bar template glyph and `brand/logo.png` from `logo/DSH Launcher.png`.
- Fix the engine supervisor so a quiet (idle) harness is never misreported as
  stopped; status changes only on real termination.
- Rebuild the Control Panel (`settings.html`) with tabs: Engine (status +
  engine controls + port), Versions (version & updates + auto-update),
  Logs (harness log tail) — with a more visual, less text-heavy design and
  the provided logo in the header.

## Impact
- Affected specs: `harness-runtime` (status accuracy), `menubar-app-shell`
  (icons), `webui-integration` (panel structure, design, logo)
- Affected code: `scripts/gen-icons.mjs`, `src-tauri/src/runtime.rs`,
  `src-tauri/resources/settings.html`, `src-tauri/resources/stopped.html`
- Version: patch bumps per item (0.1.1 -> 0.1.5).
