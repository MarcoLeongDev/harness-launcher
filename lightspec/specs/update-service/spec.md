# update-service Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
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

### Requirement: App Self-Update (Optional Endpoint)
The launcher SHALL support updating its own binary via the official Tauri updater when an update endpoint is configured in settings; SHALL skip self-updates entirely when no endpoint is configured, so the app runs without any update server.

#### Scenario: Endpoint configured
- **WHEN** a valid updater endpoint is configured
- **THEN** the app checks that endpoint for a new build and offers to install it

#### Scenario: No endpoint
- **WHEN** no endpoint is configured
- **THEN** the app never contacts an update server and self-update is disabled silently

### Requirement: Download Without Stopping the Engine
Installing (downloading) a harness version MUST NOT require stopping a running
engine, MUST NOT change the active (default) version, and MUST NOT start, stop
or restart the engine. A version that is already installed SHALL be reported as
such without redundant work.

#### Scenario: Download a new version while the engine runs
- **WHEN** the user installs a version that is not present locally while the
  engine is running
- **THEN** the download completes, the running engine keeps running the
  previously active version, and the version appears as installed

#### Scenario: Download an already-installed version
- **WHEN** the user installs a version that is already installed
- **THEN** the launcher reports it is already installed without reinstalling

### Requirement: Live Version Switching
Switching the default version SHALL work while the engine is running: the
launcher MUST stop the running engine, install the requested version when
missing, record it as the default, and start the new version again, surfacing
each phase as progress. When the engine is stopped, switching only records the
default (it never auto-starts), matching the stopped-state semantics.

#### Scenario: Switch default version while the engine runs
- **WHEN** the user picks a different default version while the engine is
  running
- **THEN** the engine is stopped, the chosen version becomes the default, and
  the engine starts again running that version

#### Scenario: Switch default version while stopped
- **WHEN** the user picks a different default version while the engine is
  stopped
- **THEN** the chosen version becomes the default and the engine remains
  stopped

