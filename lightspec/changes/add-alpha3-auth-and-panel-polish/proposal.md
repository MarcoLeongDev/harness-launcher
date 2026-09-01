# Change: Alpha-3 authenticated web URL + Control Panel polish

## Why
Harness 0.1.2-alpha.2/3 generate a per-process browser launch token and reject
any request without it (401 "dsh web authentication required"), so the launcher
opening the static `http://127.0.0.1:<port>` URL shows a white empty window.
Additionally the Control Panel has accumulated dead/unused chrome: a perpetual
"Installing DeepSeek Harness vX / Verifying installation…" status row, a
bottom "STARTING" progress cell that reports nothing, a cramped auto-update
header, a misaligned delete button, and the engine Start/Stop icon disappears
after the first status render.

## What Changes
- Capture the authenticated `dsh web: http://127.0.0.1:<port>/?token=…` URL the
  harness prints at boot and open/navigate the harness window (and Open-in-
  Browser) with it, instead of a static URL. Old versions that print no token
  keep working through the plain-URL fallback.
- Windows keep the native macOS titlebar that follows the system appearance;
  launcher-owned pages (Control Panel, stopped page) adapt light/dark via
  `prefers-color-scheme` so the whole window follows the system theme.
- Versions tab: remove the perpetual download status row and the bottom
  progress ("STARTING") cell; downloading shows only the terminal output with
  a circular icon-only cancel button overlaid on the terminal's bottom-right.
- Versions tab structure: section 1 = Auto-update cell (interval dropdown +
  enable toggle in one cell) + Check-now cell; section 2 = the version table;
  section 3 = Install-version cell (dropdown + icon-only cloud-download button).
- Versions table: delete (trash) button becomes a circular red, borderless,
  white-icon button centred under the ACTIONS column header.
- Engine tab: Start/Stop (and status) icons re-render on every status update.

## Impact
- Affected specs: menubar-app-shell (window URL/authentication),
  webui-integration (Control Panel layout, theme, icons), update-service
  (install feedback via terminal).
- Affected code: src-tauri/src/runtime.rs, commands.rs, lib.rs, tray.rs,
  src-tauri/resources/settings.html, overlay.js, stopped.html.
