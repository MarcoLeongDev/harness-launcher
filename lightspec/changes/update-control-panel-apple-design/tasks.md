## 1. Apple-style Engine tab (0.1.7)
- [x] 1.1 Single port authority — one "Runs at http://127.0.0.1:port" line, port field shown once
- [x] 1.2 Icon-driven engine controls (Start/Stop primary, Restart secondary, Force muted)
- [x] 1.3 Grouped macOS-like rows, minimal text
- [x] 1.4 Node syntax/ID checks; bump to 0.1.7

## 2. Version download UX (0.1.8)
- [x] 2.1 npm install flags: prefer-offline cache reuse, no audit/fund
- [x] 2.2 Honest progress: elapsed time, command shown once, collapsible terminal
- [x] 2.3 Apple-style design for download state + cancel
- [x] 2.4 cargo test + syntax checks; bump to 0.1.8

## 3. Logs tab minimalism (0.1.9)
- [ ] 3.1 Remove redundant "Harness Logs" label
- [ ] 3.2 Icon toolbar (refresh/follow) over minimal log output
- [ ] 3.3 checks; bump to 0.1.9

## 4. Build, deploy & verify
- [ ] 4.1 cargo test/build clean; lightspec validate passes
- [ ] 4.2 Release build + deploy to /Applications
- [ ] 4.3 Relaunch app, confirm new version live; keep running