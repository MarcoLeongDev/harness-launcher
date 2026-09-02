# Change: Terminal animation, log-tab parity, feed title, auto-update redesign

## Why
Four UI issues remain after the previous control-panel iteration:

1. The operation feed (terminal) in the Versions tab appears and disappears
   abruptly — showing/hiding should be animated.
2. The Logs tab is bad design: the log box has a fixed cap so the WINDOW
   scrolls instead of the log scrolling inside the panel; and the log text is
   rendered with a different colour scheme than the terminal output in the
   Versions tab (the terminal colours individual lines).
3. The terminal feed title truncates "npm install @deepseek-ai/dsh@…" even
   though there is room, and the feed header shows a cloud-download icon
   instead of a terminal icon.
4. The auto-update section is confusing: it has an empty cell and "Check now"
   gives no visible feedback about what is happening.

## What Changes
- Animate the feed's showing/hiding (CSS transition + reveal/collapse), with
  reduced-motion respect.
- Logs tab: the log fills the panel height and scrolls internally (the window
  never scrolls), and the log view renders lines with the same colour palette
  as the terminal feeds (shared .terminal + .ln colour rules).
- Feed title: full-width command text (no ellipsis truncation), terminal
  (_) icon instead of the cloud-download icon.
- Auto-update section: remove the empty cell, give "Check now" an in-place
  busy state (disabled + spinner + inline status) so checking is visible.

## Impact
- Affected specs: webui-integration (control panel).
- Affected code: src-tauri/resources/settings.html, resources/bootstrap-icons
  (new terminal-fill.svg), scripts checks.
