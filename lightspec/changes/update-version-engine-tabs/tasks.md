## 1. Versions table view (0.1.10)
- [x] 1.1 Table of versions: row per version, status tag, Set-default, trash button
- [x] 1.2 Delete confirmation popover anchored to the trash button
- [x] 1.3 Table header: Auto update toggle, Check for update now, Check-every dropdown (6-24 h, 1-7 d, default 12)
- [x] 1.4 Keep update banner + install-new row; verify; bump to 0.1.10

## 2. Engine card view (0.1.11)
- [x] 2.1 First line: status icon + version dropdown (stopped-only, downloaded-only) + port (stopped-only)
- [x] 2.2 Second line: circular Start/Stop (play/pause) + Restart (circular arrow) with text underneath (+ subdued Force)
- [x] 2.3 Run-time terminal collapsed by default
- [x] 2.4 Verify; bump to 0.1.11

## 3. Bootstrap icons + remove status pill (0.1.12)
- [ ] 3.1 Copy bootstrap SVG files into resources; bundle them
- [ ] 3.2 Serve icons via dsh-ui protocol handler
- [ ] 3.3 Replace all inline icons with bootstrap img; remove header status pill
- [ ] 3.4 Verify; bump to 0.1.12

## 4. Build, deploy & verify
- [ ] 4.1 cargo test/lightspec pass; syntax + ID checks
- [ ] 4.2 Release build + deploy to /Applications
- [ ] 4.3 Relaunch, confirm new version live; keep running