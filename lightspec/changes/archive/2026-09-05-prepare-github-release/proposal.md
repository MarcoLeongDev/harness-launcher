# Change: Prepare GitHub public release (Active column, minimal header, docs)

## Why
The launcher is being prepared for a public GitHub release. Three items are
required before publishing: the versions-table "Default" column header should
say "Active" (it matches the radio's actual semantics — selecting the active
engine version), the engine version tag beside the header power button should
go for a minimal design, and the public-facing docs need a refresh for release
readiness.

## What Changes
- Control panel versions table: first column header "Default" -> "Active"
  (radio tooltips updated to match; no behavior change).
- Control panel header: remove the engine version tag ("dsh vX.Y.Z") left of
  the power (quit) button; the launcher's own subtle version text next to the
  title stays.
- Release docs: README refresh, new SECURITY.md, sanitized .env.example,
  REPORT.md kept as an iteration record (release-friendly wording).
- Version: patch bumps per item (0.1.60 -> 0.1.63).

## Impact
- Affected specs: webui-integration
- Affected code: src-tauri/resources/settings.html, README.md, SECURITY.md
  (new), .env.example, REPORT.md
- Version: 0.1.60 -> 0.1.63
