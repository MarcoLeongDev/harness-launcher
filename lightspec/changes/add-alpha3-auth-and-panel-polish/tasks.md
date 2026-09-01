## 1. Authenticated harness URL (alpha 2/3 token)
- [x] 1.1 runtime.rs: parse the `dsh web: <url>` stdout line, store the latest authenticated URL on the runtime, clear it on (re)start, emit status on capture (+ unit tests)
- [x] 1.2 commands.rs: `harness_web_url` helper (wait briefly for the token URL, fall back to the plain URL); use it for window navigation, boot, tray open and open-in-browser
- [x] 1.3 overlay.js: link/anchor uses the captured URL from status
- [x] 1.4 Verify: engine boots on alpha.3 and the harness window renders the WebUI (no 401) — plain URL 401s, token URL 303→cookie→200; v0.1.21 deployed & running

## 2. System-theme following windows
- [x] 2.1 settings.html + stopped.html: full light/dark `prefers-color-scheme` palettes (native titlebar already follows the system; no theme is forced)
- [x] 2.2 Verify: no window theme is forced (code audit, tao follows the system); light/dark palettes embedded in v0.1.25 binary — final visual confirmation by human

## 3. Remove dead download/progress chrome
- [x] 3.1 settings.html: drop the "Installing DeepSeek Harness vX / Verifying installation…" row (spinner + status + elapsed + Cancel text button)
- [x] 3.2 settings.html: drop the bottom progress cell ("STARTING") and its JS/CSS

## 4. Versions tab restructure
- [x] 4.1 Section A: auto-update cell (interval dropdown + toggle together) + Check-now cell
- [x] 4.2 Section B: the version table
- [x] 4.3 Section C: Install-version cell with icon-only cloud-download button (no text)
- [x] 4.4 Downloading shows only the terminal output, with a circular icon-only cancel button overlaid bottom-right of the terminal

## 5. Versions table delete button
- [x] 5.1 Circular red borderless white-icon trash button, centred under ACTIONS

## 6. Engine tab icons
- [x] 6.1 Re-run the icon loader over re-rendered status/toggle icons

## 7. Build, deploy & verify
- [x] 7.1 cargo test + lightspec validate pass (17 unit tests green)
- [x] 7.2 Release build + deploy to /Applications (relaunch) — v0.1.25 live; token flow re-verified (plain 401 / token 303 / cookie 200); new UI markers embedded, removed chrome gone
- [x] 7.3 App left running for human verification; patch bumps per item (0.1.21→0.1.25) in package.json / tauri.conf.json / Cargo.toml
