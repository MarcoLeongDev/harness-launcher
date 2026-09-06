## 1. Brew distribution plumbing (v0.1.75)
- [x] 1.1 Bundle `dmg` target in tauri.conf.json
- [x] 1.2 README: brew-first install, source builds for contributors
- [x] 1.3 cargo test; bump to 0.1.75
- [x] 1.4 Commit, universal build (app + DMG), deploy, relaunch, verify

## 1b. Rebrand app identity to Harness Launcher (v0.1.76)
- [x] 1b.1 productName + descriptions + deploy script + README app mentions
- [x] 1b.2 Keep: bundle id (data continuity), binary name, engine strings
- [x] 1b.3 cargo test; bump to 0.1.76
- [x] 1b.4 Commit, universal build, deploy (migrate old app path), relaunch, verify

## 2. Private forge setup + first release
- [x] 2.1 Create PRIVATE GitHub + Gitee repos, remotes, push main
- [x] 2.2 Outside-repo scripts/push.sh + scripts/release.sh
- [x] 2.3 Tag v0.1.76, private GitHub + Gitee releases with DMG
- [x] 2.4 Prepare tap cask locally, validate (no public push)

## 3. Close-out
- [x] 3.1 Archive lightspec change; merge feature branch into main
- [x] 3.2 Final report (completed items, commits, versions)
