## MODIFIED Requirements
### Requirement: Settings Window
The launcher SHALL provide a dedicated native Settings window (opened from the tray or the overlay) that aggregates harness status, engine control, version & update management with live progress, port configuration, auto-update settings, and recent logs, served from a launcher-owned custom protocol origin (`dsh-ui://`) so it works in development and production without the harness server. The window SHALL organize its content into tabs — Engine (engine status, controls and port), Versions (version & update management, auto-update and progress) and Logs (harness log tail) — using a visual, minimal, icon-driven design (macOS-style grouped rows) and SHALL display the provided brand logo in its header. The Engine tab SHALL surface the effective port exactly once ("Runs at http://127.0.0.1:<port>"), provide a single port field, and use icon-first engine controls (Start/Stop primary, Restart secondary, Force subdued). The Logs tab SHALL NOT repeat a "Logs" label and SHALL present the log output with a minimal icon toolbar.

On the Versions tab the installed-version table SHALL present a Default radio
column (selecting a row's radio switches the default engine version when the
engine is stopped and is refused with a clear message while it runs), a Version
column, a Folder column whose link opens that version's installation directory,
and an Actions column with a delete button whose trash glyph is white and gains
a red background with white glyph on hover; the table SHALL NOT render a Status
column. All Versions-tab operation notifications (progress messages, live npm
console output and result/error lines) SHALL be shown in a terminal-style cell
appended as the last cell of the installed-versions table group, visible only
while there is something to display, with a circular stop-button overlay while
a version download is in flight.

#### Scenario: Open settings window
- **WHEN** the user selects Settings from the tray menu or the overlay panel
- **THEN** a settings window opens (or focuses if already open) showing the tabbed control panel

#### Scenario: Engine tab port authority
- **WHEN** the engine is running
- **THEN** the Engine tab shows exactly one running address ("Runs at http://127.0.0.1:3080") and one port field, with no duplicated port text elsewhere

#### Scenario: Logs tab
- **WHEN** the user opens the Logs tab
- **THEN** the tab shows the recent harness output with icon-only toolbar actions (refresh, follow), without a redundant title

#### Scenario: Radio switches the default version
- **WHEN** the engine is stopped and the user selects the radio of another installed version
- **THEN** that version becomes the default (active) version and the feedback reads that it is now the default version, not "select it from the Control Panel"

#### Scenario: Radio while running
- **WHEN** the engine is running and the user selects the radio of another installed version
- **THEN** the switch is refused with a clear "stop the engine before switching versions" message and the active version is unchanged

#### Scenario: Folder link opens the install directory
- **WHEN** the user clicks a row's folder link
- **THEN** the file manager reveals that version's installation directory; invalid or traversal version names are rejected

#### Scenario: Delete button hover state
- **WHEN** the user hovers a row's delete button
- **THEN** the button shows a red circular background with a white trash glyph; unhovered it shows the white glyph

#### Scenario: Download terminal feedback
- **WHEN** a version download/install is in flight
- **THEN** the versions-table group shows a terminal-style cell with the npm command, elapsed time, live output lines and a stop-button overlay that cancels the operation

#### Scenario: Idle feed hidden
- **WHEN** no operation is in flight and no recent result message is pending
- **THEN** the terminal cell is hidden so the table ends cleanly
