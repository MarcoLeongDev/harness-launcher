## MODIFIED Requirements
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
