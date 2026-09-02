## MODIFIED Requirements
### Requirement: Settings Window
The Control Panel settings window SHALL present the launcher's configuration
through tabbed sections (Engine, Versions, Logs) using the ARIA tab pattern
with keyboard arrow navigation, and SHALL expose version-management controls
(install/switch/delete), an engine status card with start/stop/restart and
port controls, and an update section. All state and actions MUST survive
launcher relaunches; user data is never deleted by redeployment.

#### Scenario: Open the settings window
- **WHEN** the user opens the settings window
- **THEN** the Engine tab is shown with the live engine status

## ADDED Requirements
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
