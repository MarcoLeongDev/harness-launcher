## MODIFIED Requirements
### Requirement: Harness Auto-Update Checks
The launcher SHALL check the npm registry for a newer `@deepseek-ai/dsh` version than the active one, SHALL be toggleable by the user, SHALL run at startup and on a user-configurable interval, and SHALL notify via tray/panel when an update is available. Manual update, install and rollback operations SHALL surface live progress (registry lookup, install, verify, switch, restart) instead of running silently.

#### Scenario: New version available
- **WHEN** an auto-update check finds a newer version
- **THEN** the user is notified and can update from the panel in one action

#### Scenario: Auto-update disabled
- **WHEN** the user disables auto-update checks
- **THEN** no background version checks occur until re-enabled or manually triggered

#### Scenario: Manual update shows progress
- **WHEN** the user triggers a manual update or install
- **THEN** the UI shows live progress for registry lookup, npm install, verification, version switch and engine restart
