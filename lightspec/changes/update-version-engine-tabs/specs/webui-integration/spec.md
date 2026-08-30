## MODIFIED Requirements
### Requirement: Settings Window
The launcher SHALL provide a dedicated native Settings window (opened from the tray or the overlay) that aggregates harness status, engine control, version & update management with live progress, port configuration, auto-update settings, and recent logs, served from a launcher-owned custom protocol origin (`dsh-ui://`) so it works in development and production without the harness server. The window SHALL organize its content into tabs — Engine (engine status, controls and port), Versions (version & update management, auto-update and progress) and Logs (harness log tail) — using a visual, minimal, icon-driven design (macOS-style grouped rows) and SHALL display the provided brand logo in its header.

#### Scenario: Open settings window
- **WHEN** the user selects Settings from the tray menu or the overlay panel
- **THEN** a settings window opens (or focuses if already open) showing the tabbed control panel

### Requirement: Versions Table
The Versions tab SHALL present installed versions in a table (one row per version) with a per-row status tag (Active/Previous/Installed), a Set-default action for choosing the active version, and a trash (Delete) button that opens a confirmation popover anchored next to it before removal. The table header SHALL contain an Auto-update toggle, a "Check for update now" action, and a "Check every" interval dropdown whose values span 6–24 hours and then 1–7 days with 12 hours as the default. An update banner and an install-a-published-version row SHALL remain available.

#### Scenario: delete requires confirmation
- **WHEN** a user clicks the trash button on a version row
- **THEN** a confirmation popover appears next to the button and the version is deleted only after confirming

#### Scenario: choose default version
- **WHEN** a user chooses Set default on an installed version
- **THEN** that version becomes the active version (installed+switched) and is tagged Active

#### Scenario: check every dropdown
- **WHEN** a user opens the Check every dropdown
- **THEN** it offers 6–24 hours plus 1–7 days, defaults to 12 hours, and repersists the chosen interval

### Requirement: Engine Card
The Engine tab SHALL present a card (collection) view whose first line shows a status icon (running/stopped/errored via Bootstrap icon), a version dropdown that is enabled only while the engine is stopped and lists downloaded versions only, and a port field enabled only while stopped. The second line SHALL hold circular icon-only buttons with text underneath for Start/Stop (play/pause icons) and Restart (circular arrow icon). The run-time terminal output SHALL be collapsed by default and expansible.

#### Scenario: controls locked while running
- **WHEN** the engine is running
- **THEN** the version dropdown and port field are disabled and only Stop (pause) is primary

#### Scenario: run-time terminal
- **WHEN** a run has produced output
- **THEN** the terminal under the card is collapsed by default and expands on toggle

### Requirement: Bootstrap Icon Assets
The Settings window SHALL use Bootstrap icon SVG files served from the app resources through the `dsh-ui://` protocol (e.g. `dsh-ui://localhost/bootstrap-icons/<name>.svg`); icons SHALL NOT be hardcoded inline, and the header SHALL NOT show a status pill with a colored dot — status is conveyed by the Engine card status icon.

#### Scenario: icons load from resources
- **WHEN** the Settings window renders an icon
- **THEN** it loads from a bundled Bootstrap SVG file via the dsh-ui protocol

#### Scenario: no status pill
- **WHEN** the Settings window is open
- **THEN** the header shows no running/status pill or status dot
