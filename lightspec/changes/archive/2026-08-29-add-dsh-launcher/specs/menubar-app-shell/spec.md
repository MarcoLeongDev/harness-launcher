## ADDED Requirements

### Requirement: Menubar-Only Presence
The launcher SHALL run as a macOS menu-bar (tray) application: it SHALL show a tray icon, SHALL NOT show a Dock icon, and SHALL keep running when all windows are closed, so closing the window never quits the app.

#### Scenario: Launch
- **WHEN** the app launches
- **THEN** a tray icon appears and no Dock icon is shown

#### Scenario: Window closed
- **WHEN** the user closes the window
- **THEN** the window hides, the app keeps running with the tray icon, and the harness keeps serving

### Requirement: Tray Menu Controls
The launcher SHALL provide a tray menu offering at least: show/hide the window, restart the harness, open in default browser, update the harness, and quit the app.

#### Scenario: Quit from tray
- **WHEN** the user selects Quit from the tray menu
- **THEN** the harness process is stopped and the app exits

#### Scenario: Reopen window
- **WHEN** the user selects Open while the window is hidden
- **THEN** the window is shown and refocused

### Requirement: Hidden-Window Lifecycle
The launcher SHALL intercept window close requests and hide instead of closing, and SHALL quit only through the explicit tray Quit action (or an in-app quit control), guaranteeing no orphaned harness processes.

#### Scenario: Close button
- **WHEN** the user presses the window close button
- **THEN** the window is hidden and the process continues
