## Context
DeepSeek Harness's web profile is a Node process: `dsh --profile web` serves a static SPA on host:port with flags --host/--port/--no-open/--trusted-host. The harness rejects `--host 0.0.0.0` by design, so loopback is the only supported exposure. Published versions of `@deepseek-ai/dsh` are pre-release (e.g. 0.1.1-rc.2). The wrapper must be "just a wrap": harness updates happen inside the app via npm into version-isolated dirs, so the Tauri binary never needs rebuilding for a harness release.

## Goals / Non-Goals
- Goals: replaceable shell; zero system-node dependency; version management with rollback; selectable port; menubar-only UX with overlay controls; spec-driven requirements in Lightspec.
- Non-Goals: harness code changes; Windows/Linux v1; non-loopback exposure.

## Decisions
- Bundled Node sidecar + vendored npm as resources; run `node <npm-cli.js>` (npx = `npm exec`). Build-time materialization uses the build machine's npm (arborist), matching how a real npm install deploys npm.
- Per-version install dirs under app-data runtime/versions/<v> + `current` pointer; rollback = pointer switch + restart (instant, offline-safe).
- Overlay panel injected via initialization_script (WKUserScript at documentStart) + withGlobalTauri IPC; harness index.html has no CSP, and WKUserScript is not blocked by CSP.
- No Dock icon via runtime ActivationPolicy::Accessory; window hides on CloseRequested; quit only from the tray.
- Harness auto-update check = npm view dist-tags.latest vs active; app self-update = tauri-plugin-updater gated on a configured endpoint (off by default, no server required to run).
- Settings in JSON with atomic rename writes.

## Risks / Trade-offs
- ~120 MB bundled Node sidecar -> acceptable for a desktop app; fetch script documents version, extraction is checksum-free but from nodejs.org HTTPS.
- Registry access needed for version listing/updates -> offline mode degrades to installed versions; rollback stays offline.
- WKWebView differences vs system browsers -> harness SPA already runs in browsers; acceptable.

## Migration Plan
- v1 ships as an unsigned/ad-hoc build; signing, notarization, and the app updater endpoint are documented follow-ups.

## Open Questions
- Notification permissions on first launch (macOS).
