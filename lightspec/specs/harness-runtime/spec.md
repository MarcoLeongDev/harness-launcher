# harness-runtime Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Isolated Harness Installations
The launcher SHALL install `@deepseek-ai/dsh` versions into isolated directories under its application-support data directory, each version occupying its own `runtime/versions/<version>/` folder, so that switching or rolling back never re-downloads or clobbers another version. Every IPC entry that turns a caller-supplied version string into a filesystem path or npm spec SHALL reject the request unless the version name passes `is_valid_version_name` (no path separators, no traversal segments, bounded length), so version directories can never escape `runtime/versions/`.

#### Scenario: Traversal version rejected
- **WHEN** a caller requests install, switch, delete or open-directory with a version containing `/`, `\`, `..` or an absolute path
- **THEN** the command fails with an invalid-version error and no filesystem change occurs

### Requirement: Bundled Node and NPM
The launcher SHALL bundle a Node.js runtime as a Tauri sidecar binary and a vendored copy of the npm CLI, and SHALL run install, update, rollback and execution of the harness exclusively with those bundled tools, never requiring system node, npm or npx. On macOS the app SHALL ship as a universal binary (Apple Silicon + Intel): the build SHALL fetch, checksum-verify and bundle the Node sidecar for both architectures, and the default release build SHALL target `universal-apple-darwin`.

#### Scenario: Fresh machine without system Node
- **WHEN** the app runs on a machine that has no system Node
- **THEN** harness install and launch still succeed using the bundled Node and vendored npm

#### Scenario: Intel Mac
- **WHEN** the app runs on an Intel Mac
- **THEN** it launches and manages the engine natively via the x86_64 slices

#### Scenario: npx-style invocations
- **WHEN** the runtime needs npx semantics (running package binaries)
- **THEN** the app invokes the vendored npm CLI via `node <npm-cli.js> exec ...`

### Requirement: Harness Version Management
The launcher SHALL let the user list published versions of `@deepseek-ai/dsh` (from the npm registry), install any listed version, switch the active version, and roll back to any previously installed version. Installs SHALL reuse the shared npm cache and use cache-friendly flags (prefer-offline, no audit/fund) so repeated or neighbor-version installs are as fast as the registry allows, and SHALL report honest progress to the user (command, elapsed time, terminal lines, cancel).

#### Scenario: Default is latest
- **WHEN** the user opens the version selector with no explicit choice
- **THEN** the `latest` dist-tag version is preselected as the default

#### Scenario: Switch to a listed version
- **WHEN** the user selects a published version
- **THEN** the app installs it if absent (reusing the npm cache) and makes it the active version, while showing honest progress with elapsed time

### Requirement: Harness Process Lifecycle
The launcher SHALL start, stop, restart and supervise the harness `dsh --profile web` process, passing `--no-open` so no browser handoff happens, and the configured `--port`; it SHALL restart the harness after a version switch or port change and SHALL surface process status and recent log output to the user. The launcher SHALL expose explicit engine controls to start, stop and restart the harness, classify engine state as one of `stopped` / `starting` / `running` / `stopping`, and SHALL keep the engine phase consistent with the actual child process: an idle harness that produces no output for an extended period SHALL NOT be reported as stopped and SHALL NOT lose its tracked process slot. The launcher SHALL NEVER signal a process it did not spawn: stop/restart act only on the tracked engine child, and a port conflict with another process SHALL be reported with the holder's identity (read-only diagnosis) instead of killing it. The cross-process kill escalation is removed.

#### Scenario: Process crash
- **WHEN** the harness process exits unexpectedly
- **THEN** the launcher records the failure, reports non-running status, and offers a restart

#### Scenario: Port held by another process
- **WHEN** the engine cannot serve because another process holds the port
- **THEN** the error names the holder and no signal is sent to it

### Requirement: Update Command via NPM
The launcher SHALL expose a single action that updates the harness to the latest version using the bundled npm (equivalent to `npm install @deepseek-ai/dsh@latest`), keeping all npm/npx machinery inside the app.

#### Scenario: Update to latest
- **WHEN** the user triggers update and a newer latest version exists
- **THEN** npm installs it, it becomes active, the harness restarts, and the panel reports success

### Requirement: Cross-Version Agent Preset Compatibility
Before starting an installed harness engine, the launcher SHALL inspect the
agent presets that engine version ships (alpha `dsh-agent-presets/presets`
roster and rc `dsh/config/agent-presets` layouts) and reconcile the shared
harness home (`~/.dsh`) with them: an `agent-presets.default` value in
`~/.dsh/settings.yaml` that the engine does not ship MUST be rewritten to an
available preset (prefer `standard` for the legacy `code` default), and when
the engine lacks a `code` preset while sessions recorded under rc engines
reference it, the launcher MUST provide a user-root `code` preset shim (a copy
of that engine's `standard` composition) so those sessions resume. The launcher
MUST NOT delete or rewrite user session data or unrelated settings.

#### Scenario: Upgrade from rc to alpha with a rc-era default preset
- **WHEN** the launcher starts an alpha engine while `~/.dsh/settings.yaml`
  records `agent-presets.default: code` and the alpha engine ships no `code`
  preset
- **THEN** the recorded default is rewritten to `standard` before the engine
  spawns and new sessions create successfully

#### Scenario: Resume a session created under an rc engine
- **WHEN** a session recorded with `agentPreset: code` is resumed under an
  engine that ships no `code` preset
- **THEN** the launcher has installed a user-root `code` preset shim so the
  session resumes instead of failing with "preset code not found"

#### Scenario: Downgrade back to an rc engine
- **WHEN** an rc engine that ships its own `code` preset starts
- **THEN** the shipped preset wins over the user-root shim and nothing is
  overwritten

