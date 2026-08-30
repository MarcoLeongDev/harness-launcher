# harness-runtime Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Isolated Harness Installations
The launcher SHALL install `@deepseek-ai/dsh` versions into isolated directories under its application-support data directory, each version occupying its own `runtime/versions/<version>/` folder, so that switching or rolling back never re-downloads or clobbers another version.

#### Scenario: First install of the default version
- **WHEN** the app starts with no installed harness
- **THEN** it installs the `latest` dist-tag version into `runtime/versions/<version>/` and records it as the active version

#### Scenario: Installing a second version
- **WHEN** the user installs a different version
- **THEN** the new version is installed alongside existing ones in its own directory and the existing directories remain untouched

### Requirement: Bundled Node and NPM
The launcher SHALL bundle a Node.js runtime as a Tauri sidecar binary and a vendored copy of the npm CLI, and SHALL run install, update, rollback and execution of the harness exclusively with those bundled tools, never requiring system node, npm or npx.

#### Scenario: Fresh machine without system Node
- **WHEN** the app runs on a machine that has no system Node
- **THEN** harness install and launch still succeed using the bundled Node and vendored npm

#### Scenario: npx-style invocations
- **WHEN** the runtime needs npx semantics (running package binaries)
- **THEN** the app invokes the vendored npm CLI via `node <npm-cli.js> exec ...`

### Requirement: Harness Version Management
The launcher SHALL let the user list published versions of `@deepseek-ai/dsh` (from the npm registry), install any listed version, switch the active version, and roll back to any previously installed version.

#### Scenario: Default is latest
- **WHEN** the user opens the version selector with no explicit choice
- **THEN** the `latest` dist-tag version is preselected as the default

#### Scenario: Switch to a listed version
- **WHEN** the user selects a published version
- **THEN** the app installs it if absent and makes it the active version

#### Scenario: Rollback to a previously installed version
- **WHEN** the user chooses rollback
- **THEN** the app switches the active pointer to the prior installed version without network access

### Requirement: Harness Process Lifecycle
The launcher SHALL start, stop, restart and supervise the harness `dsh --profile web` process, passing `--no-open` so no browser handoff happens, and the configured `--port`; it SHALL restart the harness after a version switch or port change and SHALL surface process status and recent log output to the user. The launcher SHALL expose explicit engine controls to start, stop and restart the harness, classify engine state as one of `stopped` / `starting` / `running` / `stopping`, and SHALL support a force-restart that escalates to killing any process still bound to the harness port when the child does not release it. The launcher SHALL keep the engine phase consistent with the actual child process: an idle harness that produces no output for an extended period SHALL NOT be reported as stopped and SHALL NOT lose its tracked process slot.

#### Scenario: Process crash
- **WHEN** the harness process exits unexpectedly
- **THEN** the launcher records the failure, reports non-running status, and offers a restart

#### Scenario: Idle harness stays running
- **WHEN** the harness runs without producing output for more than 30 seconds
- **THEN** the launcher keeps reporting `running` and retains the process slot so stop/restart still control the live child

### Requirement: Update Command via NPM
The launcher SHALL expose a single action that updates the harness to the latest version using the bundled npm (equivalent to `npm install @deepseek-ai/dsh@latest`), keeping all npm/npx machinery inside the app.

#### Scenario: Update to latest
- **WHEN** the user triggers update and a newer latest version exists
- **THEN** npm installs it, it becomes active, the harness restarts, and the panel reports success

