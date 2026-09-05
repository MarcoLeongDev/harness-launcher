## 1. Remove stop button + dead cancel code (v0.1.60)
- [x] 1.1 Panel: drop dl-cancel markup/CSS/handler; header display via the
      kind+phase heuristic; stuck hint reworded (no stop mention)
- [x] 1.2 Overlay: drop lc-stopdown button/CSS/handler (engine Stop stays)
- [x] 1.3 Backend: remove stoppable flag, emit_stoppable, cancel_operation,
      AppState.cancel set, run_npm cancel polling; versions.rs reverts to emit
- [x] 1.4 Delete unused stop-fill.svg resource
- [x] 1.5 Tests rewritten (no stop-button selectors); npm test green;
      cargo check green; bump to 0.1.60; commit

## 2. Build, deploy & verify
- [ ] 2.1 Release build + deploy:relaunch to /Applications
- [ ] 2.2 Verify live: no Stop download control on any feed; terminal
      behaviour (header, elapsed, output, auto-dismiss) intact