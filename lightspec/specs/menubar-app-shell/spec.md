# menubar-app-shell Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Menubar-Only Presence
The launcher SHALL run as a macOS menu-bar (tray) application: it SHALL show a tray icon, SHALL NOT show a Dock icon, and SHALL keep running when all windows are closed, so closing the window never quits the app.

#### Scenario: Launch
- **WHEN** the app launches
- **THEN** a tray icon appears and no Dock icon is shown

#### Scenario: Window closed
- **WHEN** the user closes the window
- **THEN** the window hides, the app keeps running with the tray icon, and the harness keeps serving

### Requirement: Tray Menu Controls
The launcher SHALL provide a tray menu offering at least: show/hide the window, open the settings window, start/stop/restart/force-restart the engine, open the harness in the default browser, update the harness, and quit the app. Start/Stop items SHALL be enabled or disabled according to the current engine state.

#### Scenario: Quit from tray
- **WHEN** the user selects Quit from the tray menu
- **THEN** the harness process is stopped and the app exits

#### Scenario: Reopen window
- **WHEN** the user selects Open while the window is hidden
- **THEN** the window is shown and refocused

#### Scenario: Engine controls reflect state
- **WHEN** the engine is running, Stop/Restart/Force are enabled and Start is disabled; when stopped, Start is enabled and Stop/Restart/Force are disabled
- **THEN** the tray items reflect the current engine phase

### Requirement: Hidden-Window Lifecycle
The launcher SHALL intercept window close requests and hide instead of closing, and SHALL quit only through the explicit tray Quit action (or an in-app quit control), guaranteeing no orphaned harness processes.

#### Scenario: Close button
- **WHEN** the user presses the window close button
- **THEN** the window is hidden and the process continues

