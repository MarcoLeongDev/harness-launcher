# Change: Distribute via Homebrew tap (brew-first install)

## Why
Building from source should be for contributors only. Like SubBar, users
SHOULD install via `brew install --cask`, from a DMG published to GitHub +
Gitee releases and served through the shared `MarcoLeongDev/homebrew-tap`
tap (jsDelivr for GitHub, direct Gitee URL for China users).

## What Changes
- `tauri.conf.json`: bundle targets gain `dmg` (universal DMG alongside .app).
- README: brew install becomes the documented install path; source builds
  move under Development/Contributing.
- Release tooling (outside this repo, mirroring usagebar `scripts/`):
  `push.sh` (origin+gitee) and `release.sh` (universal DMG + SHA256SUMS,
  tag push, GitHub release via `gh`, Gitee release via API).
- Tap cask `deepseek-harness-launcher.rb` + dist DMG in the shared tap repo
  (prepared locally; pushed when the app repos go public — jsDelivr only
  serves public repos).
- Version: 0.1.74 -> 0.1.75.

## Impact
- Affected specs: menubar-app-shell (new distribution requirement)
- Affected code: `src-tauri/tauri.conf.json`, `README.md`
- No behavior change; user data untouched. Unsigned build (like SubBar's
  free-cert flow): cask strips quarantine post-install until notarized.
