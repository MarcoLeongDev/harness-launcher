# DeepSeek Harness Launcher — Iteration Report (v0.1.1 → v0.1.6)

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
| 6 | (hardening) orphaned harness on quit | App now stops the harness child on ANY exit path (tray Quit, panel Quit, AppleScript/Cmd+Q) so no orphaned harness holds the port after the launcher exits. | 14055a0 | 0.1.6 |

## Verification
- cargo test: 11/11 pass (port fallback, semver, settings roundtrip)
- lightspec validate --all --strict: 6/6 pass
- node --check on script assets; settings.html ID/wiring consistency check: 51 IDs, 44 referenced, none missing
- Generated assets verified pixel-wise: app icon and brand logo show the logo (white bg + teal/navy mark); tray icon is a monochrome mark silhouette
- Each version built as a release bundle and installed to /Applications (CFBundleShortVersionString bumped 0.1.1→0.1.5)
- LSUIElement=true in the deployed Info.plist (menu-bar app, no Dock icon)

## Runtime note
The currently running instance was left untouched during iteration (it hosts the session's harness on 127.0.0.1:3081). The final build v0.1.5 is installed; a relaunch swaps the old binary for the new one.
---

# Iteration 2 — Apple-style Control Panel (v0.1.7 → v0.1.9)

Branch: feat/control-panel-apple-design (merged main after iteration 1; lightspec change archived)

| # | Item | Fix | Commit | Version |
|---|------|-----|--------|---------|
| 1 | Engine/Port tab redesign — Apple-like | Full macOS Settings-style redesign: grouped translucent cells, SF-symbol-style inline icon set, icon-first engine controls (Start primary / Stop / Restart / Force subdued), status hero with animated running state, single port authority — the address appears exactly once (status line); port field shows supplementary text only when fallback applies. | 6c13e77 | 0.1.7 |
| 2 | Version download felt fake / impossibly long | Root causes: installs ran warn-loglevel with progress disabled (silent for minutes) and no cache reuse, with 5-minute fetch timeouts (stalls/retries). Now prefer-offline (shared cache), fetch-retries=1, fetch-timeout=60000, http loglevel (real activity streams), live fetch counter as compact subtitle with ticking elapsed time, Apple-style download row (spinner, version, elapsed, Cancel) + collapsible Terminal. | c16d0a5 | 0.1.8 |
| 3 | Logs tab — redundant log label | No Logs/Harness Logs title anywhere; tab name suffices. Minimal icon toolbar (refresh + follow) over output; removed text-y line-count status. | 35254e1 | 0.1.9 |

Verification: cargo test 11/11, lightspec validate --all --strict 6/6, inline JS syntax + ID-wiring checks, npm flag smoke test on the registry, binary markers confirmed in the installed bundle.


---

# Iteration 3 — Versions table + Engine card (v0.1.10 → v0.1.12)

Branch: feat/control-panel-version-engine-overhaul (merged main; previous change archived)

| # | Item | What was done | Commit | Version |
|---|------|---------------|--------|---------|
| 1 | Version tab → table view | Installed versions as a table: per-row status tag, Set default (picks the active version), trash button with an anchored confirmation popover before deletion. Table header: Auto update toggle, Check for update now, and a Check-every dropdown (6–24 hours then 1–7 days, default 12h). Update banner + Install-version row (with pre-release toggle) retained. | 727e1fd | 0.1.10 |
| 2 | Engine tab → collection/card view | First line: status icon (running/stopped/errored), version dropdown (enabled only while stopped; downloaded versions only), port field (enabled only while stopped). Second line: circular icon-only buttons with text underneath — Start/Stop (play/pause) and Restart (circular arrow), plus subdued Force. Run-time output terminal collapsed by default. | f09bec3 | 0.1.11 |
| 3 | Bootstrap icons + remove status pill | 22 Bootstrap icon SVG files copied as real files into resources/bootstrap-icons, bundled and served via dsh-ui:// handler at runtime (raw files, never hardcoded); panel icons load from them; the header status pill (colored dot) removed — status is shown by the Engine card's bootstrap status icon. | fc516bc | 0.1.12 |

Verification: cargo test 11/11, lightspec validate --all --strict 6/6, JS syntax + ID-wiring checks per item, icon presence at runtime bundle path (22 files), binary markers per version, bundle version 0.1.10 → 0.1.12 confirmed after each deploy.

