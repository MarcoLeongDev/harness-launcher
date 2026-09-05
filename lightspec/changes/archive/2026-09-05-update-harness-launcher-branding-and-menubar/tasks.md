# Tasks

## 1. Menu bar (v0.1.48) — commit `b967afb`
- [x] Menu bar: remove "Force Restart Harness" + "Update Harness…" (item, state, refresh, handlers)
- [x] Menu bar: native glyph for each remaining item via `IconMenuItem` + `NativeIcon`
- [x] Tests: `cargo test` green (30 passed); `npm test` green (12 passed)
- [x] Version bump 0.1.47 → 0.1.48 (package.json, tauri.conf.json, Cargo.toml, Cargo.lock)
- [x] Commit `b967afb` (v0.1.48), redeploy to /Applications, verified menubar (7 items, no force/update, quit label, enabled states)

## 2. Rebrand titles (v0.1.49) — commit `f013717`
- [x] Rebrand window titles: `window.rs` Harness + Control Panel → "Harness Launcher - …"
- [x] Rebrand control panel HTML: `<title>`, logo alt in `settings.html`
- [x] Rebrand stopped page title/heading (`stopped.html`)
- [x] Rebrand tray tooltip + Quit label, overlay meta/toggle, notifications (`lib.rs`, `update.rs`), `prepare.mjs` shell
- [x] Version bump 0.1.48 → 0.1.49
- [x] Commit `f013717` (v0.1.49), redeployed, verified titles: "Harness Launcher - Harness" + "Harness Launcher - Control Panel", panel shows 0.1.49

## 3. Versions tab (v0.1.50) — commit `eea1a68`
- [x] Versions tab: remove "Get latest" button (HTML, `sel` map, listener)
- [x] Versions tab: install dropdown defaults to newest published version (no ✓ → knows to download)
- [x] `check_updates` copy no longer references the removed "Get latest" button
- [x] Regression test: dropdown defaults to latest, installed ✓ / uninstalled no ✓ (16 assertions green)
- [x] Version bump 0.1.49 → 0.1.50
- [x] Commit `eea1a68` (v0.1.50), redeployed, verified: dropdown preselected to latest (0.1.2-rc.1 = registry latest), no Get latest button, app left running

## 4. Archive
- [x] Archive lightspec change; merge feature branch to main; final report table