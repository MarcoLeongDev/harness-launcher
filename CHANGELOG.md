# Changelog

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
