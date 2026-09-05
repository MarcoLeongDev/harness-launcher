# Delta for Harness Runtime

## MODIFIED Requirements
### Requirement: Isolated Harness Installations
The launcher SHALL install `@deepseek-ai/dsh` versions into isolated directories under its application-support data directory, each version occupying its own `runtime/versions/<version>/` folder, so that switching or rolling back never re-downloads or clobbers another version. Every IPC entry that turns a caller-supplied version string into a filesystem path or npm spec SHALL reject the request unless the version name passes `is_valid_version_name` (no path separators, no traversal segments, bounded length), so version directories can never escape `runtime/versions/`.

#### Scenario: Traversal version rejected
- **WHEN** a caller requests install, switch, delete or open-directory with a version containing `/`, `\`, `..` or an absolute path
- **THEN** the command fails with an invalid-version error and no filesystem change occurs

### Requirement: Bundled Node and NPM
The launcher SHALL bundle a Node.js runtime as a Tauri sidecar binary and a vendored copy of the npm CLI, and SHALL run install, update, rollback and execution of the harness exclusively with those bundled tools, never requiring system node, npm or npx. The build scripts SHALL pin exact Node and npm versions and SHALL verify the SHA256 checksum of every downloaded toolchain artifact before extracting or vendoring it, so releases are reproducible and tamper-evident.

#### Scenario: Fresh machine without system Node
- **WHEN** the app runs on a machine that has no system Node
- **THEN** harness install and launch still succeed using the bundled Node and vendored npm

#### Scenario: Tampered toolchain download
- **WHEN** a downloaded Node or npm artifact fails checksum verification
- **THEN** the prepare step aborts before the artifact is used

### Requirement: Harness Process Lifecycle
The launcher SHALL start, stop, restart and supervise the harness `dsh --profile web` process, passing `--no-open` so no browser handoff happens, and the configured `--port`; it SHALL restart the harness after a version switch or port change and SHALL surface process status and recent log output to the user. The launcher SHALL expose explicit engine controls to start, stop and restart the harness, classify engine state as one of `stopped` / `starting` / `running` / `stopping`, and SHALL keep the engine phase consistent with the actual child process: an idle harness that produces no output for an extended period SHALL NOT be reported as stopped and SHALL NOT lose its tracked process slot. The launcher SHALL NEVER signal a process it did not spawn: stop/restart act only on the tracked engine child, and a port conflict with another process SHALL be reported with the holder's identity (read-only diagnosis) instead of killing it. The cross-process kill escalation is removed.

#### Scenario: Process crash
- **WHEN** the harness process exits unexpectedly
- **THEN** the launcher records the failure, reports non-running status, and offers a restart

#### Scenario: Port held by another process
- **WHEN** the engine cannot serve because another process holds the port
- **THEN** the error names the holder and no signal is sent to it
