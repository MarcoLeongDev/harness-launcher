# Change: Merge install UX into the versions table

## Why
The Control Panel still carries a standalone "Install version" group above the
installed-versions table. The install affordances live far from the list they
act on, and the duplicated layout costs vertical space. Folding installation
into the versions table gives every version-related control one home, with the
install row as a stable last row of the table.

## What Changes
- Versions table (Control Panel, settings.html):
  - Header column "Folder" is renamed to "Directory".
  - The published-version dropdown and the cloud-download install button move
    into the table as its persistent LAST row:
    empty Default cell | install version dropdown | empty Directory cell | cloud-down install button
  - The standalone "Install version" group is removed.
  - The install row stays last even while operation feeds render above it, and
    when no versions are installed.
- Log toggle button moves from bottom-left to the bottom-right of the window
  for better balance with the drawer content.

## Impact
- Affected specs: webui-integration (Control Panel UI)
- Affected code: src-tauri/resources/settings.html (markup, CSS, comment
  updates only — element IDs and IPC logic are preserved), extended
  scripts/test-panel-feed.mjs structural checks.
- User data/content safety: no engine, install, or settings-data changes;
  installed versions under ~/Library/Application Support survive untouched.
- Version: patch bump 0.1.55 -> 0.1.56.
