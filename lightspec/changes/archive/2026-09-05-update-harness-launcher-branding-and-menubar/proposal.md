# Change: Harness Launcher branding + version-latest dropdown + menubar cleanup

## Why
1. The app rebranded to "Harness Launcher" (v0.1.46) but several windows and
   labels still carry the old brand "DeepSeek Harness Launcher": the Control
   Panel window title + `<title>`, the harness web-view window title, the
   stopped page, notifications, the tray tooltip/Quit label and the
   `web-dist` shell page. Users see the old brand in the window titlebars.
2. The Versions tab has a separate "Get latest" button even though the install
   dropdown already knows the latest publish. Thus the user is shown the
   current/active version by default (little utility) and must find the
   separate button to discover an update. The latest should simply BE the
   selection in the dropdown: when it is not yet installed it renders without
   the ✓ checkmark, so the user instantly knows to download it.
3. The menu bar carries "Force Restart Harness" and "Update Harness…" items
   that duplicate engine-card/panel actions; the remaining items are plain
   text with no glyphs, unlike native macOS menus.

## Changes
- Rebrand every runtime-visible "DeepSeek Harness Launcher" title/label to
  "Harness Launcher" (window titles, control panel HTML, stopped page, tray
  tooltip/Quit, notifications, overlay meta, web-dist shell page).
- Versions tab: remove the "Get latest" button; the install-version dropdown
  defaults to the newest published version (last entry of the ascending
  version list, or `latestRemote` when known) instead of the active version.
- Menu bar: remove "Force Restart Harness" and "Update Harness…"; give every
  remaining item a native macOS menu glyph (`IconMenuItem` + `NativeIcon`).
- Backend copy: `check_updates` no longer refers to the removed "Get latest"
  button.

## Requirements
See `specs/menubar-app-shell/spec.md` and `specs/webui-integration/spec.md`
for the delta requirements and scenarios.