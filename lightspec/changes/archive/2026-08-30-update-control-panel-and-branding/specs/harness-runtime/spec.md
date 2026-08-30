## MODIFIED Requirements
### Requirement: Harness Process Lifecycle
The launcher SHALL start, stop, restart and supervise the harness `dsh --profile web` process, passing `--no-open` so no browser handoff happens, and the configured `--port`; it SHALL restart the harness after a version switch or port change and SHALL surface process status and recent log output to the user. The launcher SHALL expose explicit engine controls to start, stop and restart the harness, classify engine state as one of `stopped` / `starting` / `running` / `stopping`, and SHALL support a force-restart that escalates to killing any process still bound to the harness port when the child does not release it. The launcher SHALL keep the engine phase consistent with the actual child process: an idle harness that produces no output for an extended period SHALL NOT be reported as stopped and SHALL NOT lose its tracked process slot.

#### Scenario: Process crash
- **WHEN** the harness process exits unexpectedly
- **THEN** the launcher records the failure, reports non-running status, and offers a restart

#### Scenario: Idle harness stays running
- **WHEN** the harness runs without producing output for more than 30 seconds
- **THEN** the launcher keeps reporting `running` and retains the process slot so stop/restart still control the live child
