# webui-integration Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Embedded Harness WebUI
The launcher SHALL display the harness WebUI in a native window loading `http://127.0.0.1:<port>` where the port is the active configured port, and SHALL reload the page when the port changes.

#### Scenario: Port change reload
- **WHEN** the active port changes and the harness restarts
- **THEN** the WebView navigates to the new localhost URL

### Requirement: Bottom-Left Overlay Panel
The launcher SHALL render a floating control panel over the bottom-left corner of the harness WebUI inside the window. The panel SHALL show harness status and active version, and SHALL provide controls for: selecting/installing a version (defaulting to latest), updating to the latest version, rolling back to a previously installed version, changing the port, toggling automatic update checks, viewing recent harness logs, reopening in the system browser, and quitting the app. The panel SHALL additionally expose engine start/stop controls and a live progress indicator for in-flight operations, and SHALL not render on launcher-owned pages (`dsh-ui:`).

#### Scenario: Open panel
- **WHEN** the user clicks the launcher button at the bottom-left of the WebUI
- **THEN** the panel expands with status, version selector, update, rollback, port, auto-update, logs, engine controls, progress and quit controls

#### Scenario: Panel survives navigation
- **WHEN** the user navigates within the harness SPA
- **THEN** the overlay remains visible and functional

#### Scenario: Overlay skips launcher pages
- **WHEN** a page served from the `dsh-ui:` protocol is displayed in the main window
- **THEN** the overlay does not inject a second control panel

### Requirement: Secure IPC Bridge
The launcher SHALL expose Rust commands to the panel through the Tauri IPC layer gated by a capability file, with `withGlobalTauri` enabled for the injected script; the panel SHALL never hold secrets and SHALL treat harness-served content as untrusted.

#### Scenario: Invoke from injected script
- **WHEN** the injected overlay calls a Rust command
- **THEN** the command result is returned to the overlay and unauthorized commands are rejected

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

### Requirement: One Reusable Terminal Feed Per Operation
The Versions tab SHALL render operation output through a reusable feed
component — header (command + elapsed time), terminal body, and a circular
stop-button overlay — of which one instance exists per in-flight operation.
Parallel version downloads MUST each own their own feed (no shared output
element), console lines MUST be tagged with their operation so a feed only
shows its own lines, and cancellation MUST target a single operation. Feeds
persist across panel refreshes (re-seeded from status + console snapshots)
and the area collapses when no operation is active.

#### Scenario: Two downloads run at the same time
- **WHEN** the user starts downloading two different versions one after the
  other while the first is still running
- **THEN** two independent feeds appear, each streaming only its own npm
  output with its own elapsed timer

#### Scenario: Stop one of two downloads
- **WHEN** the user taps the stop button on the first download's feed
- **THEN** only that download is cancelled and the second continues unaffected

#### Scenario: Panel opened mid-download
- **WHEN** the Control Panel is opened while a download is in flight
- **THEN** the feed for that download is rebuilt from status and console
  snapshots with its existing output

### Requirement: Animated Operation Feed
The operation feed terminal in the Versions tab SHALL animate its showing and
hiding (fade and slide) rather than appearing abruptly, and MUST respect the
user's reduced-motion preference by skipping the animation.

#### Scenario: A download starts and finishes
- **WHEN** a version download begins
- **THEN** its feed animates in; when the operation settles the feed animates
  out and the area collapses

### Requirement: Logs Tab Fills the Panel
The Logs tab SHALL occupy the full remaining height of the Control Panel and
scroll its log content internally; opening the Logs tab MUST NOT make the
window itself scroll. The log lines SHALL be rendered with the same colour
palette as the operation feed terminals.

#### Scenario: Logs tab with many lines
- **WHEN** the harness log contains more lines than fit on screen
- **THEN** the log scrolls inside the panel while the window stays fixed

### Requirement: Terminal Feed Title and Icon
The operation feed header SHALL show the full install command
("npm install @deepseek-ai/dsh@<version>") without ellipsis truncation when
space allows, and SHALL use a terminal icon (_) rather than a download icon.

#### Scenario: Feed header during install
- **WHEN** a download is in flight
- **THEN** the header shows the full command with the terminal icon

### Requirement: Auto-Update Section Usability
The auto-update section SHALL have no empty cells, and the "Check now" action
MUST give visible in-place feedback (disabled button with progress indicator
and an inline status message) from the moment it is clicked until it resolves.

#### Scenario: User clicks Check now
- **WHEN** the user clicks "Check now"
- **THEN** the button becomes busy with a spinner and an inline status message
  appears, then updates with the result or error

### Requirement: Stable Operation Key For An Operation's Lifetime
Every launcher operation SHALL stream all of its progress phases under ONE
operation key, from its first emitted phase through its terminal phase
(done, failed, or cancelled). The backend MUST NOT switch an operation's
key mid-flight: a feed created by an earlier phase would never observe a
terminal phase under the new key, so it would linger in the downloading
state, fire the 60s stuck-watch against a completed operation, and keep a
stop button that cancels a key nothing is running under.

#### Scenario: Update while the engine is running
- **WHEN** the user runs "Get latest" while the engine is running and the
  resolved version differs from the active one
- **THEN** the registry, stopping, installing, verifying and terminal
  phases all arrive under the same op key
- **AND** the operation's feed closes shortly after the done phase
- **AND** no "no progress for 60s" line is appended after the done phase

#### Scenario: Genuinely stalled download
- **WHEN** a download is in flight and emits no progress message for 60
  seconds
- **THEN** the feed surfaces the stuck-watch hint once so the user can
  cancel it

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

### Requirement: Launcher Branding in Window Titles
Every runtime-visible launcher title SHALL use the brand "Harness Launcher" (never "DeepSeek Harness Launcher"): the harness web-view window title and Control Panel window title, the Control Panel `<title>` and header, the stopped page, the tray tooltip and Quit label, launcher notifications, the overlay panel meta, and the `web-dist` shell page.

#### Scenario: Open the harness window
- **WHEN** the harness web-view window is created
- **THEN** its title reads "Harness Launcher - Harness"

#### Scenario: Open the Control Panel
- **WHEN** the Control Panel window is created
- **THEN** its title reads "Harness Launcher - Control Panel"

#### Scenario: App is stopped
- **WHEN** the engine is stopped and the stopped page is shown
- **THEN** the page heading and window title read "Harness Launcher" (with the "Engine Stopped -" prefix on the title)

#### Scenario: Quit from the tray
- **WHEN** the user opens the tray menu
- **THEN** the tooltip and Quit item read "Harness Launcher" / "Quit Harness Launcher"

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

