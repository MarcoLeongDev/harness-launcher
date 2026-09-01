## 1. Reliable installs (root cause)
- [ ] 1.1 versions.rs: add --legacy-peer-deps and remove --prefer-offline from the npm install args
- [ ] 1.2 Verify alpha.2 and rc.1/rc.2 install cleanly (no ETARGET, no silent 0-package reify)
- [ ] 1.3 Keep the clear-error hardening for any future install failure

## 2. Versions table polish
- [ ] 2.1 Remove the redundant "Active"/"Previous" text tags; keep green checkmark
- [ ] 2.2 Ensure pre-release (rc/alpha) versions are selectable + switchable from the GUI
- [ ] 2.3 Confirm trash-with-popover delete still works for any installed version

## 3. Engine card toggle icon
- [ ] 3.1 Fix the missing Start/Stop (play/pause) toggle button icon

## 4. Boot-time install visibility
- [ ] 4.1 Surface the default-version install progress to the Control Panel on launch

## 5. Build, deploy & verify
- [ ] 5.1 cargo check / lightspec validate pass
- [ ] 5.2 Release build + deploy to /Applications (relaunch)
- [ ] 5.3 Confirm new version live; keep app running; bump patch version
