## 1. Version-name validation on all IPC entries — SN2+SN4 (v0.1.64)
- [x] 1.1 Gate `delete_version`, `set_version`, `install_and_switch`, `rollback` on `is_valid_version_name`
- [x] 1.2 Unit tests for traversal rejection on each entry path
- [x] 1.3 cargo test + panel feed test; bump to 0.1.64
- [x] 1.4 Commit, release build, deploy, relaunch, verify

## 2. Delete-popover XSS fix — SN5 (v0.1.65)
- [ ] 2.1 Rebuild delete-confirm popover with DOM/`textContent` (no version string in `innerHTML`)
- [ ] 2.2 Audit remaining `innerHTML` sinks in settings.html/overlay.js
- [ ] 2.3 Panel feed test; bump to 0.1.65
- [ ] 2.4 Commit, release build, deploy, relaunch, verify

## 3. Capability split + CSP + force-stop hardening — SN1+SN6+SN7 (v0.1.66)
- [ ] 3.1 Per-window capabilities (main: status/events only; settings: full)
- [ ] 3.2 `withGlobalTauri` off for harness-served content; CSP on `dsh-ui://`
- [ ] 3.3 Force-stop restricted to tracked child; UI confirmation for force restart
- [ ] 3.4 cargo test; bump to 0.1.66
- [ ] 3.5 Commit, release build, deploy, relaunch, verify

## 4. Pinned + verified build toolchain — SN3 (v0.1.67)
- [ ] 4.1 Pin Node + npm versions; SHA256 verify downloads in fetch/bundle scripts
- [ ] 4.2 Reproducible-asset check; bump to 0.1.67
- [ ] 4.3 Commit (no rebuild needed unless toolchain changes)

## 5. Token redaction — SN9 (v0.1.68)
- [ ] 5.1 Never log launch-token URLs; centralize redaction in `tail_logs`
- [ ] 5.2 Overlay log viewer redacts like the Control Panel
- [ ] 5.3 cargo test; bump to 0.1.68
- [ ] 5.4 Commit, release build, deploy, relaunch, verify

## 6. Log bounds — SN10 (v0.1.69)
- [ ] 6.1 Clamp `tail_logs` line count; rotate `launcher.log`
- [ ] 6.2 cargo test; bump to 0.1.69
- [ ] 6.3 Commit, release build, deploy, relaunch, verify

## 7. Engine auto-update cleanup — SN11 (v0.1.70)
- [ ] 7.1 Verify auto-check wiring; remove auto-checker, flags, UI; keep manual check
- [ ] 7.2 Delete dead code flagged by compiler warnings
- [ ] 7.3 cargo test; bump to 0.1.70
- [ ] 7.4 Commit, release build, deploy, relaunch, verify

## 8. Version-mutation serialization — SN12 (v0.1.71)
- [ ] 8.1 Serialize install/switch/delete/rollback mutations
- [ ] 8.2 cargo test; bump to 0.1.71
- [ ] 8.3 Commit, release build, deploy, relaunch, verify

## 9. Release hygiene — SN13 (v0.1.72)
- [ ] 9.1 CHANGELOG.md; README/SECURITY release notes touch-up
- [ ] 9.2 Bump to 0.1.72
- [ ] 9.3 Commit, release build, deploy, relaunch, verify; keep the app running

## 10. Close-out
- [ ] 10.1 Archive lightspec change; merge feature branch into main
- [ ] 10.2 Final report (completed items, commits, versions)
