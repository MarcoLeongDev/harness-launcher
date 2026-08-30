# DeepSeek Harness Launcher — Iteration Report (v0.1.1 → v0.1.5)

Branch: feat/control-panel-branding (based on main @ 0751bb6)
Change: lightspec/changes/update-control-panel-and-branding (validated, all tasks done)

## Items delivered (in the required order)

| # | Item | Fix | Commit | Version |
|---|------|-----|--------|---------|
| 1 | App icon + menu-bar icon are not the provided logo | Root cause: gen-icons.mjs hand-rolled PNG decoder produced black/transparent garbage from logo/DSH Launcher.png. Rewrote the pipeline to use macOS `sips`; app icon (icon.png → icns set via `tauri icon`), menu-bar template glyph (silhouette of the logo mark) and brand/logo.png all now derive from the provided logo. | e37c9f9 | 0.1.1 |
| 2 | Control Panel shows engine stopped while the WebUI works | Root cause: supervisor thread treated a 30s no-output idle as child death — cleared the process slot and emitted a "stopped" status even though the harness was alive. Now idle timeouts are heartbeats; only real termination (Terminated/Disconnected) changes state. | c466d36 | 0.1.2 |
| 3 | Control Panel should use tabs | Rewrote settings.html with ARIA tabs: Engine (status + controls + port), Versions (version & update + auto-update + progress/terminal), Logs. All commands wired identically to before. | 26c0e81 | 0.1.3 |
| 4 | Better, less text-heavy design | Visual polish: pulsing running-state dot, update-available banner with one-click Update, installed versions as tiles, version chips moved to footer, centered column, focus-visible/keyboard tab arrows. | fb14123 | 0.1.4 |
| 5 | Control Panel should use the provided logo | Header now shows the brand logo (dsh-ui://localhost/brand/logo.png — now a real logo image); stopped page picks it up too. | 4ee99d2 | 0.1.5 |

## Verification
- cargo test: 11/11 pass (port fallback, semver, settings roundtrip)
- lightspec validate --all --strict: 6/6 pass
- node --check on script assets; settings.html ID/wiring consistency check: 51 IDs, 44 referenced, none missing
- Generated assets verified pixel-wise: app icon and brand logo show the logo (white bg + teal/navy mark); tray icon is a monochrome mark silhouette
- Each version built as a release bundle and installed to /Applications (CFBundleShortVersionString bumped 0.1.1→0.1.5)
- LSUIElement=true in the deployed Info.plist (menu-bar app, no Dock icon)

## Runtime note
The currently running instance was left untouched during iteration (it hosts the session's harness on 127.0.0.1:3081). The final build v0.1.5 is installed; a relaunch swaps the old binary for the new one.
