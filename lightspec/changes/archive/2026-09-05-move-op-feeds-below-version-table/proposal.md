# Change: Move operation feeds below the versions table

## Why
After merging the install row into the versions table, in-flight operation
terminals render between the installed-version rows and the install row —
inside the table group. The terminals break the table's row rhythm and look
odd. Operation output deserves its own surface below the table.

## What Changes
- The operation feeds move to their own group/section BELOW the versions
  table group (which ends with the persistent install row).
- The section is visible only while it has content; when the terminal output
  goes away (feeds finish and animate out), the section disappears entirely
  (takes no space).
- Comments, CSS (first-feed border inside its own group) and structural
  tests updated; feed behaviour (per-operation feeds, seeding, cancellation)
  unchanged.

## Impact
- Affected specs: webui-integration (feed placement, install-row guarantee)
- Affected code: src-tauri/resources/settings.html,
  scripts/test-panel-feed.mjs structural checks.
- User data/content safety: UI-only change; engine, installs and settings
  untouched.
- Version: patch bump 0.1.56 -> 0.1.57.