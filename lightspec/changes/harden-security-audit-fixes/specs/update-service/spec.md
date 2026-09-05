# Delta for Update Service

## MODIFIED Requirements
### Requirement: Harness Auto-Update Checks
The launcher SHALL NOT run automatic background update checks for the harness
engine. Update discovery SHALL be manual only: the user triggers a check from
the panel ("Check now"), which queries the npm registry for a newer
`@deepseek-ai/dsh` version than the active one and notifies via panel when an
update is available. Manual update, install and rollback operations SHALL
surface live progress (registry lookup, install, verify, switch, restart)
instead of running silently. No background thread, schedule, or persisted
auto-check setting SHALL remain.

#### Scenario: Manual check finds a newer version
- **WHEN** the user triggers a manual update check and a newer version exists
- **THEN** the user is notified in the panel and can update in one action

#### Scenario: No background checks
- **WHEN** the launcher runs for any length of time without user action
- **THEN** it never contacts the npm registry for version discovery on its own

#### Scenario: Manual update shows progress
- **WHEN** the user triggers a manual update or install
- **THEN** the UI shows live progress for registry lookup, npm install, verification, version switch and engine restart
