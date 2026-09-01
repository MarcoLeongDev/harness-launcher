# Change: Reliable version downloads + version-table polish

## Why
Installing harness versions from the Control Panel silently hangs or fails. Two
distinct root causes were identified:
1. The harness package (and its dependency tree) carry peer-dependency
   conflicts that npm 12's strict peer resolution chokes on, causing arborist to
   build the ideal tree but silently skip reify and exit 0 with 0 packages,
   leaving the UI stuck on "Install finished - verifying".
2. The launcher passed --prefer-offline, so npm reuses a stale packument cache.
   When a freshly published version depends on a package range the cached
   metadata does not know about, install fails with ETARGET and the user cannot
   download new releases.

Additionally the Versions table still shows an "Active" text tag (the green
checkmark is enough), the engine card toggle button icon is missing, and the
boot-time "installing default version" progress is not visibly surfaced.

## What Changes
- Install command: add --legacy-peer-deps (fixes peer-dep reify skip) and remove
  --prefer-offline so metadata is always fresh from the registry while tarballs
  still use the cache (fixes ETARGET for new releases).
- Versions table: drop the redundant Active/Previous text tags; keep the green
  checkmark for installed versions.
- Engine card: render the Start/Stop toggle icon (play/pause SVG) correctly.
- Boot flow: surface the default-version install progress to the Control Panel so
  "installing on start" is actually visible.
- Pre-release versions (rc / alpha) selectable from the GUI and switchable.

## Impact
- Affected specs: update-service (version install/resolution), webui-integration
  (Control Panel versions + engine card).
- Affected code: src-tauri/src/versions.rs, src-tauri/src/commands.rs,
  src-tauri/src/lib.rs, src-tauri/resources/settings.html.
