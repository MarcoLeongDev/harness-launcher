## 1. Reliable installs (root cause)
- [x] 1.1 versions.rs: add --legacy-peer-deps and remove --prefer-offline from the npm install args
- [x] 1.2 Verify alpha.2/alpha.3 install cleanly (no ETARGET, no silent 0-package reify)
- [x] 1.3 missing-peer completion so alpha/rc trees boot (dsh-jobs, dsh-fs, dsh-sandbox, etc.)
- [x] 1.4 Keep the clear-error hardening for any future install failure

## 2. Versions table polish
- [x] 2.1 Remove the redundant "Active" text tag; keep green checkmark
- [x] 2.2 Ensure pre-release (rc/alpha) versions are selectable + switchable from the GUI
- [x] 2.3 Confirm trash-with-popover delete still works for any installed version

## 3. Engine card toggle icon
- [x] 3.1 Verify Start/Stop (play/pause) toggle button icon loads (fix landed in v0.1.13 loadIconsIn)

## 4. Boot-time install visibility
- [x] 4.1 Control Panel refresh() hides progress when currentOp is null (no phantom "installing")

## 5. Build, deploy & verify
- [ ] 5.1 cargo check / lightspec validate pass
- [ ] 5.2 Release build + deploy to /Applications (relaunch)
- [ ] 5.3 Confirm new version live; keep app running; bump patch version
