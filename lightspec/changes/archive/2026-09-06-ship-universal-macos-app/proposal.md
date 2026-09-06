# Change: Ship a universal macOS app + release-doc polish

## Why
The app is Apple-Silicon-only: the Node sidecar fetch is hardcoded to
`darwin-arm64` and the Rust bundle targets the host arch, so Intel Macs
cannot run it. For a public GitHub release the app SHOULD be universal
(arm64 + x86_64) while staying macOS-only, and the release docs need the
remaining polish: bundle category, trademark disclaimer, first-launch
network note, full prerequisites/commands, and the vendored-muda sync note.

Out of scope (explicitly skipped): CI workflows (`.github/`) and a full
`CONTRIBUTING.md` build guide — prerequisites live in the README instead;
a slim CONTRIBUTING holds only the muda-patch sync note.

## What Changes
- `fetch-node.mjs`: fetch + verify BOTH darwin-arm64 and darwin-x64 sidecars
  (pinned SHA256 each); triple-to-asset mapping fixed for overrides.
- `bundle-npm.mjs`: smoke-check every sidecar present.
- `package.json`: default `build` becomes universal
  (`tauri build --target universal-apple-darwin`); `build:host` kept for fast
  iteration; deploy path unchanged.
- `tauri.conf.json`: `bundle.category = DeveloperTool`.
- README: unofficial-project trademark disclaimer, first-launch network note,
  expanded prerequisites + commands, Intel support statement.
- CONTRIBUTING.md (slim): vendored-muda sync burden + pointer to README.
- Version: 0.1.72 -> 0.1.73 (universal), -> 0.1.74 (docs).

## Impact
- Affected specs: harness-runtime (Bundled Node and NPM)
- Affected code: `scripts/fetch-node.mjs`, `scripts/bundle-npm.mjs`,
  `package.json`, `src-tauri/tauri.conf.json`, `README.md`,
  `CONTRIBUTING.md` (new)
- User data untouched. Intel Macs gain support; Apple Silicon unaffected.
- Requires both `aarch64-apple-darwin` and `x86_64-apple-darwin` Rust targets
  for release builds.
