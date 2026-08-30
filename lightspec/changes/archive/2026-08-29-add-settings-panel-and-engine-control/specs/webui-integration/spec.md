## ADDED Requirements
### Requirement: Settings Window
The launcher SHALL provide a dedicated native Settings window (opened from the tray or the overlay) that aggregates harness status, engine control, version & update management with live progress, port configuration, auto-update settings, and recent logs, served from a launcher-owned custom protocol origin (`dsh-ui://`) so it works in development and production without the harness server.

#### Scenario: Open settings window
- **WHEN** the user selects Settings from the tray menu or the overlay panel
- **THEN** a settings window opens (or focuses if already open) showing the full control panel

#### Scenario: Live operation progress
- **WHEN** an install / update / rollback / engine / port operation is in progress
- **THEN** the settings window and overlay show a live progress bar with phase and message, and reopening the UI mid-operation still shows the in-flight operation

#### Scenario: Engine stopped page
- **WHEN** the engine is stopped while the main window was showing the harness UI
- **THEN** the main window redirects to a launcher-provided stopped page offering Start and Open Settings instead of a dead connection error

## MODIFIED Requirements
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
