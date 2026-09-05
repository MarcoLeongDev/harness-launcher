# Change: Merge Engine+Versions panel, log toggle, SF-symbol menu icons

## Why
The tray menu and control panel need a cleaner, more native look and a
simpler hierarchy:

- The tray menu still uses generic AppKit glyphs where explicit SF Symbol
  equivalents are wanted (a gear shape, and `arrow.up.right.square` for
  "Open in Browser"); the engine card shows a globe icon for the same action.
- The control panel is split across Engine / Versions / Logs tabs, but Engine
  and Versions belong together on one page: the engine table cell first, then
  "Install version", then the installed-versions table.
- The Logs tab should not be a full tab: the log is an auxiliary surface,
  hidden by default and toggled by a bottom-left circular image-only button
  (log icon, no background/border except on hover).

## What Changes
- Tray menu (menubar-app-shell): gear SF Symbol where the switch-style glyph
  is used today; `arrow.up.right.square` glyph on "Open in Browser".
- Engine card (webui-integration): the globe icon on "open in browser" becomes
  `arrow.up.right.square` (box-arrow-up-right bootstrap SVG is already
  bundled).
- Control panel: merge Engine + Versions into a single default section — the
  engine table cell first, then the "Install version" group, then the
  installed-versions table group (with operation feeds). The separate
  Engine/Versions/Logs tab bar is removed; the merged view is the default.
- Log: hidden by default; a bottom-left circular icon-only toggle (log icon,
  transparent background/border, background only on hover, aria-pressed)
  shows/hides the log pane.

## Impact
- Affected specs: menubar-app-shell, webui-integration
- Affected code: src-tauri/src/tray.rs, src-tauri/resources/settings.html,
  (icon assets in src-tauri/resources)
- Version: patch bumps per item (0.1.50 -> 0.1.53)
