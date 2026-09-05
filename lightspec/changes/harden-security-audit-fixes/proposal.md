# Change: Harden launcher for public release (security audit SN1-7, 9-10, 12-13 + engine auto-update cleanup)

## Why
The pre-release security audit (reported 2026-09-06, 13 findings) found the
launcher releasable but with hardening gaps that SHOULD be closed before a
public GitHub release: untrusted harness content holds full IPC rights, two
IPC entries allow path traversal, the build toolchain is fetched without
integrity checks, and several smaller issues (XSS sink, missing CSP,
unbounded log reads, token logging). The user approved fixing SN 1-7, 9, 10,
12, 13, plus SN11 (verify whether engine auto-update still exists and remove
unused code). SN8 (signed app self-updates) is explicitly deferred.

## What Changes
- SN2+SN4: `is_valid_version_name` enforced on every IPC entry that turns a
  version string into a path or npm spec (`delete_version`, `set_version`,
  `install_and_switch`, `rollback`); tests added.
- SN5: Control Panel delete-confirm popover built with DOM/`textContent`
  instead of string-concatenated `innerHTML`.
- SN1+SN6+SN7: per-window Tauri capabilities (harness window: status +
  events only; Control Panel: full set), `withGlobalTauri` off for
  harness-served content, CSP on `dsh-ui://` pages, force-stop escalation
  restricted to the tracked child (+ UI confirmation for force restart).
- SN3: pinned Node/npm versions with SHA256 verification in
  `fetch-node.mjs` / `bundle-npm.mjs`.
- SN9: launch token never logged; redaction centralized in `tail_logs`.
- SN10: `tail_logs` line-count clamp + rotation for `launcher.log`.
- SN11: engine auto-check machinery removed (manual "check now" retained);
  dead code deleted.
- SN12: version mutations serialized so delete/install/switch cannot race.
- SN13: CHANGELOG added; README/SECURITY release notes updated.
- Version: patch bumps per item (0.1.63 -> 0.1.72+).

## Impact
- Affected specs: harness-runtime, update-service, webui-integration
- Affected code: `src-tauri/src/commands.rs`, `runtime.rs`, `versions.rs`,
  `window.rs`, `lib.rs`, `settings.rs`, `state.rs`, `update.rs`,
  `capabilities/`, `tauri.conf.json`, `resources/settings.html`,
  `resources/overlay.js`, `resources/stopped.html`, `scripts/`,
  `CHANGELOG.md` (new)
- User data (installed versions, settings, logs) is preserved; only the
  auto-check schedule keys become inert.
