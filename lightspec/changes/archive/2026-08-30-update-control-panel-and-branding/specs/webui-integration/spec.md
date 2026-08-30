## MODIFIED Requirements
### Requirement: Settings Window
The launcher SHALL provide a dedicated native Settings window (opened from the tray or the overlay) that aggregates harness status, engine control, version & update management with live progress, port configuration, auto-update settings, and recent logs, served from a launcher-owned custom protocol origin (`dsh-ui://`) so it works in development and production without the harness server. The window SHALL organize its content into tabs — Engine (engine status, controls and port), Versions (version & update management, auto-update and progress) and Logs (harness log tail) — using a visual, non-text-heavy design, and SHALL display the provided brand logo in its header.

#### Scenario: Open settings window
- **WHEN** the user selects Settings from the tray menu or the overlay panel
- **THEN** a settings window opens (or focuses if already open) showing the tabbed control panel

#### Scenario: Live operation progress
- **WHEN** an install / update / rollback / engine / port operation is in progress
- **THEN** the settings window and overlay show a live progress bar with phase and message, and reopening the UI mid-operation still shows the in-flight operation

#### Scenario: Tab navigation
- **WHEN** the user activates the Engine, Versions or Logs tab
- **THEN** the corresponding panel is shown, the active tab is indicated, and the window header keeps the brand logo visible
