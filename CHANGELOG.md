# Changelog

## v0.1.97 — Split page assets, CSP without inline execution
- Control Panel and engine-stopped pages are now markup + same-origin assets (`settings.css/js`, `stopped.css/js`, served from the `dsh-ui` protocol with correct content types); visual style and behavior unchanged.
- Content-Security-Policy tightened: `script-src`/`style-src 'self'` (no more `unsafe-inline`), mirrored in `tauri.conf.json`. New `scripts/test-csp.mjs` contract pins it; panel/stopped/i18n tests updated to span the split files.

## v0.1.96 — Biome lint/format gate for JavaScript
- Added `@biomejs/biome` (pinned devDependency) with `biome.json` (space indent, `noVar`/`noAssignInExpressions` as errors) covering `scripts/**/*.mjs` plus the shipped `overlay.js`/`findzoom.js`; new `npm run lint` / `npm run format` scripts, and `npm test` now runs the lint gate first. Fixed all findings (`var` → `let`/`const`, template literals, brace-bodied `forEach`, assignment-free search loops, expanded test shims). All suites stay green.

## v0.1.95 — Rust edition 2024, clippy/fmt gates green
- Toolchain modernized: Rust edition 2021 → 2024, MSRV 1.77 → 1.85; `cargo fmt` applied across the tree and all `cargo clippy` warnings in our crate fixed (unit bindings, `next_back`, `&Path` params, derived `Default`, type alias, doc formatting, test-module placement). Vendored `muda` fork warnings left untouched. All 63 Rust tests + Node contract suites stay green.

## v0.1.94 — Engine-stopped splash speaks five languages
- The no-engine splash now renders in the saved UI language (En | 繁 | 簡 | 日 | Es, default En): same locale table as the Control Panel, painted from get_status.language with English fallback, polling so a Control Panel switch repaints the open splash. Visual style unchanged; backend log lines stay English.

## v0.1.93 — Durable engine updates via MCP env bridging
- Root cause of the 0.1.5-alpha.1 boot loop: user patch layers use process.env secrets for MCP headers; a GUI launch leaves them unset so the value resolves to undefined, which logs as empty headers but fails schemastery validation and bricks the whole plugin tree.
- The launcher now bridges missing process.env names from the host env, credentials refs, and the Agentqueue server config into the engine child (env-only, values redacted in logs, user files never written), with empty-string fallback so validation passes and boot continues.
- Boot failures are classified: MCP invalid-config gets an actionable missing-env message, native-binding failures keep the rebuild path; the native classifier no longer swallows MCP errors. Sessions, settings, and sibling versions are untouched throughout.

## v0.1.92 — Ad-hoc sign deployed app for Gatekeeper
- `scripts/deploy.mjs` re-signs the installed bundle (`codesign --force --deep --sign -`) and verifies it, so a locally built app launches from Finder without a Gatekeeper block. Repeatable and never touches user data (`~/.dsh`).

## v0.1.91 — Find field keeps focus while typing
- Fixed: the find text field lost focus on every keystroke because page
  keydown listeners (e.g. the harness composer refocus) received the field's
  key events. Keystrokes in the find field no longer propagate to the page;
  focus is also restored after search, and closing the bar never strands
  focus on the hidden field. Zoom shortcuts work from inside the field.

## v0.1.89 — Webview find (Cmd/Ctrl+F) and zoom (Cmd/Ctrl +/−/0)
- Both webviews (harness window + Control Panel) now have browser-style
  in-page find (floating bar, match count, Enter/Shift+Enter navigation,
  Esc to close) and page zoom (10% steps, 50–200%, Cmd/Ctrl+0 resets).
- Zoom level persists per origin across restarts and redeploys; user data
  (`~/.dsh`, installed versions, settings) is untouched.

## v0.1.88 — v0.1.3-alpha.2 boot failure fixed (fs-ext native binding)
- Root cause: vendored npm 12 blocks install scripts by default, so
  `fs-ext` (new via `dsh-session-persistence-jsonl`) never compiled and
  the engine crashed with `Cannot find module './build/Release/fs_ext.node'`
  (v0.1.2-rc.1 has no fs-ext dep, so it kept working).
- Harness installs/rebuilds now allow install scripts, build against the
  bundled node (PATH shim, correct NODE_MODULE_VERSION), and fail fast
  with an actionable error when a tree is unbootable — no more silent
  success followed by a 30 s "did not answer" mystery.
- Boot/start/switch detect native-binding failures, attempt one in-place
  rebuild, and a failed running switch rolls back to the previous working
  version instead of stranding the engine. `~/.dsh` sessions and settings
  are never touched by install/repair/rollback.
- Build fix: `bundle-npm.mjs` preserves executable bits when vendoring npm
  (the shipped node-gyp helper lost +x and failed builds with
  `Permission denied`), with a fail-closed exec check.

## v0.1.87 — Tray browser glyph is the SF link symbol
- "Open in Browser" carries SF `link` instead of `arrow.up.right.square`,
  matching the Control Panel engine-cell link button.

## v0.1.86 — Engine-cell open button uses the link glyph
- The Control Panel engine-cell open-in-browser button renders the
  chain-link glyph instead of the box-arrow (tooltip, label and tint
  unchanged; retired the unused box-arrow asset).

## v0.1.85 — Tray fullscreen items removed
- Dropped "Harness Full Screen" / "Control Panel Full Screen" from the
  tray menu (glyphs, handlers, labels): the green traffic-light button
  owns fullscreen since v0.1.84. Tray keeps Open, Control Panel, engine
  controls, browser and quit.

## v0.1.84 — Green button enters native fullscreen
- Harness + Control Panel windows are fullscreen-primary: the green
  traffic-light button enters native macOS fullscreen instead of zooming
  (Option-click / long-press still zoom; tray toggles unchanged).

## v0.1.83 — Control Panel + tray language switcher (En|繁|簡|日|Es)
- Segmented language control next to the header quit button; choice
  persists in settings.json and translates the Control Panel chrome and
  the tray menu immediately (default En, English fallback). New
  panel-only `set_language` command; backend progress/notice lines stay
  English (operational output).

## v0.1.82 — Neutral operation terminal header
- Operation terminal headers read "Harness Launcher #" for every action
  (install, switch, update, rollback, engine ops) instead of claiming an
  install is running. Applies to Control Panel feeds and overlay terminal.

## v0.1.81 — Manual version-list refresh button
- The install-row version dropdown gains an icon-only refresh button
  (refresh glyph, no text) that re-fetches the published list from the
  registry immediately instead of waiting for the 60 s status-poll cache
  to expire. New panel-only `refresh_versions` command; selection is
  preserved and the button busy-disables while fetching.

## v0.1.80 — Tray fullscreen toggles
- New "Harness Full Screen" and "Control Panel Full Screen" tray menu
  items toggle native macOS fullscreen on their window (green-button zoom
  unchanged). Fullscreen SF Symbol glyphs; items disable while their
  window does not exist.

## v0.1.79 — Brand header opens the GitHub project page
- The logo, app name and launcher version now open the project GitHub page in the default browser (Control Panel header, engine-stopped page, harness-window overlay brand line). Visual style unchanged: no underline, color shift or extra decoration. New benign read-only `open_repo_page` command (allowlisted URL, callable from any window).

## v0.1.78 — README quickstart + new screenshot
- README reorganized around a quickstart flow (badges, screenshots,
  install, features, source build); new Control Panel + harness screenshot.

## v0.1.77 — Brew trust + unsigned-build notes
- Install docs now include `brew trust MarcoLeongDev/tap` (required for
  third-party taps) and explain the ad-hoc-signed (free Apple account)
  first-launch flow.

## v0.1.74 — Release-doc polish
- README: unofficial-project trademark disclaimer, first-launch network
  note, full prerequisites + commands, universal (Intel) support statement.
- New slim CONTRIBUTING.md holding the vendored-muda patch sync burden.

## v0.1.73 — Universal macOS binary
- Default release build targets `universal-apple-darwin` (arm64 + x86_64,
  verified with `lipo`); Node sidecars fetched + verified for both arches
  and combined into one universal sidecar; `build:host` kept for fast
  iteration; deploy script prefers the universal bundle.
- Bundle category set to DeveloperTool.

## v0.1.72 — Release hygiene (SN13)
- Added this CHANGELOG; README/SECURITY updated for the security-hardened
  behavior (read-only harness window, manual update checks, no force-restart).

## v0.1.71 — Serialized version mutations (SN12)
- Install/switch/download/rollback/set/delete now hold a global
  version-mutation lock, so concurrent operations queue instead of
  interleaving directory, engine or settings changes.

## v0.1.70 — Engine auto-check removed (SN11)
- Verified: silent auto-update never existed, but a background auto-check
  thread (registry poll + notification) did run. Removed the checker thread,
  the `auto_update_harness` / `auto_update_interval_hours` settings and the
  overlay auto-check UI. Update discovery is manual-only ("Check now").
  Stale keys in existing `settings.json` files are ignored.

## v0.1.69 — Bounded logs (SN10)
- `tail_logs` line count clamped to 10–2000; `launcher.log` rotates at 1 MB.

## v0.1.68 — Launch-token redaction (SN9)
- Engine `?token=` / bearer credentials are redacted at log ingestion (log
  file, in-memory tail, status payloads) and in `tail_logs` (covers log files
  written before this change). Raw URLs are still captured for navigation.

## v0.1.67 — Pinned, verified toolchain (SN3)
- Node v24.20.0 (SHA256-pinned) and npm 12.0.2 (integrity-pinned); both fetch
  scripts verify checksums and fail closed. Bumping either requires updating
  the pin in `scripts/`.

## v0.1.66 — IPC least privilege + CSP + no cross-process kills (SN1, SN6, SN7)
- Per-window capabilities (`main` vs `settings`) plus Rust-side source-window
  enforcement: the harness window keeps status/logs/start/browser/settings;
  everything mutating requires the Control Panel. Overlay reflects this.
- Content-Security-Policy on launcher-owned `dsh-ui://` pages.
- Removed `engine_force_restart` and the `lsof` + `kill -9` port escalation;
  only the tracked engine child is ever signalled, and port conflicts report
  the holder (read-only) instead of killing it.

## v0.1.65 — Delete-popover XSS fix (SN5)
- Delete-confirm popover built with DOM/`textContent`; icon names allow-listed.
- Panel feed regression test covers hostile version names.

## v0.1.64 — Version-name validation (SN2, SN4)
- Every IPC entry that turns a version string into a path or npm spec rejects
  traversal/shell-shaped names; `install_version` is the choke point.

## v0.1.63 — Release docs
- Release-ready README, SECURITY.md, sanitized `.env.example`; dropped stale
  REPORT.md and the private-session screenshot.

## v0.1.62 — Minimal header
- Removed the engine version tag beside the header power button.

## v0.1.61 — Active column
- Versions table header "Default" → "Active", matching the radio semantics.
