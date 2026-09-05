## MODIFIED Requirements

### Requirement: Settings Window
The Control Panel settings window SHALL present the launcher's configuration
through ONE merged view (no tab bar): an engine status card with start/stop/
restart and port controls, an update banner, and a versions table. The
versions table SHALL use the columns Default | Version | Directory | Actions
(radio select, version, open-directory button, delete button), and its LAST
row SHALL always be the install row — the published-version dropdown in the
Version column and a cloud-download install button in the Actions column with
the Default and Directory cells empty. A live log SHALL be available in a
bottom drawer toggled by a circular image-only button pinned to the
bottom-right of the window. All state and actions MUST survive launcher
relaunches; user data is never deleted by redeployment.

#### Scenario: Open the settings window
- **WHEN** the user opens the settings window
- **THEN** the merged view is shown with the live engine status and the
  versions table (install row last, log toggle at the bottom-right)

### Requirement: Latest Version Defaults in the Install Dropdown
The versions-table install dropdown (the persistent last row) SHALL default
its selection to the newest published harness version
(the last entry of the ascending version list, or the known `latest`
dist-tag when available) rather than the active version, and SHALL NOT offer
a separate "Get latest" control. Because an uninstalled newest version
renders without a ✓ checkmark, the panel SHALL make it immediately clear
which version is worth downloading next.

#### Scenario: Table opens with an update available
- **WHEN** the user opens the Control Panel and the newest published version
  is not installed
- **THEN** the install dropdown in the table's last row is pre-selected to
  that newest version and its option shows no checkmark

#### Scenario: The newest version is already installed
- **WHEN** the newest published version is already installed
- **THEN** the dropdown still pre-selects it and its option shows the ✓
  checkmark

## ADDED Requirements

### Requirement: Merged Install Row in the Versions Table
The Control Panel SHALL fold installation into the versions table: the
standalone "Install version" group SHALL be removed, the header column SHALL
be labelled "Directory" (not "Folder"), and the table SHALL always end with
the install row — an empty Default cell, the published-version dropdown in
the Version column, an empty Directory cell, and a cloud-download install
button in the Actions column. The install row MUST stay the last row while
operation feeds render above it and when no versions are installed. Existing
element IDs and IPC behaviour MUST be preserved so downloads, switching and
deletion keep working.

#### Scenario: No versions installed
- **WHEN** no harness versions are installed yet
- **THEN** the table still shows the install row with a usable dropdown and
  install button

#### Scenario: Download in flight
- **WHEN** a version is downloading (an operation feed is visible)
- **THEN** the install row remains the last row of the table
