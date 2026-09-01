# Change: Alpha 5 — switch feedback, delete button styling, folder links, terminal notification cell

## Why
The Control Panel Versions tab gives confusing feedback and misses practical
affordances:
1. Clicking the "Default" radio to switch the engine version replies
   *"installed 0.1.2-alpha.3 — select it from the Control Panel"* — but the radio
   **is** the designed way to switch, so the message (and the fact that the
   switch never actually happens) is wrong.
2. The delete (trash) button icon is red and turns grey on hover; it should be a
   white glyph that gains a red background on hover.
3. The table's "Status" column is redundant (a green check per installed row)
   and there is no way to open a version's install directory.
4. Operation notifications ("installed 0.1.0-rc.7 — select it from the Control
   Panel", npm output) render in a stray cell under the *Install version* row,
   which makes no sense there. The git history (v0.1.23) had a terminal-style
   download surface with a stop-button overlay; it should return as the last
   cell of the versions-table section.

## What Changes
- `install_and_switch` (radio + install button) actually switches the default
  version (installs when missing, refuses while the engine is running, records
  previous version) and returns helpful feedback instead of
  "select it from the Control Panel".
- Delete button: white trash glyph by default (theme-aware backing for light
  mode); hovered state = red background + white glyph.
- Remove the "Status" column; add a per-row folder link that opens the exact
  installation directory of that version (new `open_version_dir` command with
  path-traversal validation and unit tests).
- Versions-tab notifications (progress messages, npm console output, result
  lines) move to a terminal-style cell appended as the **last cell of the
  installed-versions table section**, restored from v0.1.23 (v0.1.23 had it as
  its own group; it now closes the table group), including the circular stop
  button overlay while a version downloads; the stray cells under the install
  row are removed.

## Impact
- Affected specs: webui-integration (Settings Window Versions tab behaviour)
- Affected code: `src-tauri/src/commands.rs` (switch_version_inner, new
  open_version_dir), `src-tauri/src/versions.rs` (version-name validation),
  `src-tauri/src/lib.rs` (command registration),
  `src-tauri/resources/settings.html` (table columns, delete button, feed cell)
- User data: runtime versions/logs/settings under the app data dir are untouched
  by all changes and by redeployment (ditto bundle install only).
