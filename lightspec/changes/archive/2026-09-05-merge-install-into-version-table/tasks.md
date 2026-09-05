## 1. Merge install UX into versions table (v0.1.56)
- [x] 1.1 Rename table header column "Folder" -> "Directory" (title/labels updated)
- [x] 1.2 Add persistent install row as the LAST row of the versions table:
      empty | install dropdown | empty | cloud-down download button
- [x] 1.3 Remove the standalone "Install version" group; keep IDs/IPC logic intact
- [x] 1.4 Update empty-state hint + comments to match the new layout
- [x] 1.5 Extend panel tests (structure checks) and run npm test
- [x] 1.6 Verify; bump patch to 0.1.56; commit

## 2. Log toggle to bottom-right (v0.1.56)
- [x] 2.1 Reposition .log-toggle to bottom-right; update comments
- [x] 2.2 Verify keyboard/hover/a11y behaviour unchanged

## 3. Build, deploy & verify
- [x] 3.1 Tests + cargo check pass
- [x] 3.2 Release build + deploy to /Applications (relaunch; keep app running)
- [x] 3.3 Confirm v0.1.56 live and new table UX renders