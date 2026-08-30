## ADDED Requirements

### Requirement: Harness Auto-Update Checks
The launcher SHALL check the npm registry for a newer `@deepseek-ai/dsh` version than the active one, SHALL be toggleable by the user, SHALL run at startup and on a user-configurable interval, and SHALL notify via tray/panel when an update is available.

#### Scenario: New version available
- **WHEN** an auto-update check finds a newer version
- **THEN** the user is notified and can update from the panel in one action

#### Scenario: Auto-update disabled
- **WHEN** the user disables auto-update checks
- **THEN** no background version checks occur until re-enabled or manually triggered

### Requirement: App Self-Update (Optional Endpoint)
The launcher SHALL support updating its own binary via the official Tauri updater when an update endpoint is configured in settings; SHALL skip self-updates entirely when no endpoint is configured, so the app runs without any update server.

#### Scenario: Endpoint configured
- **WHEN** a valid updater endpoint is configured
- **THEN** the app checks that endpoint for a new build and offers to install it

#### Scenario: No endpoint
- **WHEN** no endpoint is configured
- **THEN** the app never contacts an update server and self-update is disabled silently
