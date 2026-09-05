## 1. Versions table header: Default -> Active (v0.1.61)
- [x] 1.1 Header cell text + title -> "Active"; radio tooltips say "active version"
- [x] 1.2 Update comments referencing the Default column
- [x] 1.3 cargo test + panel feed test; bump to 0.1.61
- [x] 1.4 Commit, release build, deploy, relaunch, verify

## 2. Minimal header: remove engine version tag (v0.1.62)
- [x] 2.1 Remove the #dsh-ver span, .hdr-tag CSS and its render code
- [x] 2.2 Launcher subtle version text next to the title stays
- [x] 2.3 cargo test; bump to 0.1.62
- [ ] 2.4 Commit, release build, deploy, relaunch, verify

## 3. GitHub release docs (v0.1.63)
- [ ] 3.1 README refresh for release (accurate commands/paths, security notes)
- [ ] 3.2 Add SECURITY.md (supported versions, how to report a vulnerability)
- [ ] 3.3 Sanitize .env.example (no local usernames); review REPORT.md wording
- [ ] 3.4 Commit; bump to 0.1.63
- [ ] 3.5 Build, deploy, relaunch, verify; keep the app running
- [ ] 3.6 Archive lightspec change; merge feature branch into main
