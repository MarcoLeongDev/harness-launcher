# Change: Version table + Engine card views for the Control Panel

## Why
The Versions tab is not a clear management surface and its UX is weak: versions
are scattered between a dropdown, generic buttons and tiles; there is no table
with per-version actions; the check interval is an unlabeled number input; the
Engine tab mixes controls with a select that allows anything, and status is a
dim pill with a colored dot. The panel should present a versions TABLE (each
row one version, per-row delete with confirmation popover, choose the default
version) with a header holding Auto-update toggle / Check for update now /
Check-every interval dropdown (6–24 hours then 1–7 days); an Engine CARD
(collection view): status icon (running/stopped/errored), version dropdown
(enabled only when stopped, only downloaded versions), port field (enabled only
when stopped), circular icon-only Start/Stop (play/pause) and Restart (circular
arrow) with text underneath, and a run-time terminal collapsed by default.
Icons must be Bootstrap icon SVG files (no hardcoded icons), and the header
status pill with the dot is removed in favour of the status icon.

## What Changes
- Versions tab: table of installed versions; per-row Set-default + Delete with
  confirmation popover; header: Auto update toggle, Check for update now,
  Check-every dropdown (6h–24h, 1d–7d, default 12h)
- Engine tab: card view with status icon, version select (stopped-only,
  downloaded-only), port field (stopped-only), circular Start/Stop + Restart
  (+ subdued Force) with text underneath, collapsed run-time terminal
- Bootstrap icon SVG files served from the app resources via dsh-ui protocol;
  header status pill removed
- Icons: raw bootstrap SVG files copied into resources, never hardcoded

## Impact
- Affected specs: webui-integration (settings window tabs/controls)
- Affected code: src-tauri/resources/settings.html, src-tauri/src/lib.rs
  (uri handler), src-tauri/tauri.conf.json (bundle resources)
- Version: patch bumps per item (0.1.10 -> 0.1.12)
