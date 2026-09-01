## 1. Authenticated harness URL (alpha 2/3 token)
- [ ] 1.1 runtime.rs: parse the `dsh web: <url>` stdout line, store the latest authenticated URL on the runtime, clear it on (re)start, emit status on capture (+ unit tests)
- [ ] 1.2 commands.rs: `harness_web_url` helper (wait briefly for the token URL, fall back to the plain URL); use it for window navigation, boot, tray open and open-in-browser
- [ ] 1.3 overlay.js: link/anchor uses the captured URL from status
- [ ] 1.4 Verify: engine boots on alpha.3 and the harness window renders the WebUI (no 401)

## 2. System-theme following windows
- [ ] 2.1 settings.html + stopped.html: full light/dark `prefers-color-scheme` palettes (native titlebar already follows the system; no theme is forced)
- [ ] 2.2 Verify: switch macOS appearance and confirm the window follows

## 3. Remove dead download/progress chrome
- [ ] 3.1 settings.html: drop the "Installing DeepSeek Harness vX / Verifying installation…" row (spinner + status + elapsed + Cancel text button)
- [ ] 3.2 settings.html: drop the bottom progress cell ("STARTING") and its JS/CSS

## 4. Versions tab restructure
- [ ] 4.1 Section A: auto-update cell (interval dropdown + toggle together) + Check-now cell
- [ ] 4.2 Section B: the version table
- [ ] 4.3 Section C: Install-version cell with icon-only cloud-download button (no text)
- [ ] 4.4 Downloading shows only the terminal output, with a circular icon-only cancel button overlaid bottom-right of the terminal

## 5. Versions table delete button
- [ ] 5.1 Circular red borderless white-icon trash button, centred under ACTIONS

## 6. Engine tab icons
- [ ] 6.1 Re-run the icon loader over re-rendered status/toggle icons

## 7. Build, deploy & verify
- [ ] 7.1 cargo test + lightspec validate pass
- [ ] 7.2 Release build + deploy to /Applications (relaunch), engine + panel verified per item
- [ ] 7.3 Keep the app running for human verification; patch bumps per item in package.json / tauri.conf.json / Cargo.toml
