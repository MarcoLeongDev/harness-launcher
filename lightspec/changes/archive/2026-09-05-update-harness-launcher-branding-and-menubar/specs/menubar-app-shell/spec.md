## MODIFIED Requirements

### Requirement: Tray Menu Controls
The launcher SHALL provide a tray menu offering at least: show/hide the window, open the settings window, start/stop/restart the engine, open the harness in the default browser, and quit the app. Start/Stop/Restart items SHALL be enabled or disabled according to the current engine state. The tray menu SHALL NOT offer a force-restart or an update-harness item. Every tray menu item SHALL carry a native macOS menu glyph (template icon) representing its action.

#### Scenario: Quit from tray
- **WHEN** the user selects Quit from the tray menu
- **THEN** the harness process is stopped and the app exits

#### Scenario: Reopen window
- **WHEN** the user selects Open while the window is hidden
- **THEN** the window is shown and refocused

#### Scenario: Engine controls reflect state
- **WHEN** the engine is running, Stop/Restart are enabled and Start is disabled; when stopped, Start is enabled and Stop/Restart are disabled
- **THEN** the tray items reflect the current engine phase

#### Scenario: Menu items are glyph-labelled
- **WHEN** the user opens the tray menu
- **THEN** every item shows a native macOS icon glyph alongside its text