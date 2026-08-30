## MODIFIED Requirements
### Requirement: Harness Process Lifecycle
The launcher SHALL start, stop, restart and supervise the harness `dsh --profile web` process, passing `--no-open` so no browser handoff happens, and the configured `--port`; it SHALL restart the harness after a version switch or port change and SHALL surface process status and recent log output to the user. The launcher SHALL expose explicit engine controls to start, stop and restart the harness, classify engine state as one of `stopped` / `starting` / `running` / `stopping`, and SHALL support a force-restart that escalates to killing any process still bound to the harness port when the child does not release it.

#### Scenario: Start on launch
- **WHEN** the app starts and a harness version is active and `start_on_launch` is enabled
- **THEN** the harness process is started on the configured port and the WebView loads http://127.0.0.1:<port>

#### Scenario: Restart after version switch
- **WHEN** the active version changes
- **THEN** the old harness process is stopped and the new version is started on the same port

#### Scenario: Process crash
- **WHEN** the harness process exits unexpectedly
- **THEN** the launcher records the failure, reports non-running status, and offers a restart

#### Scenario: Engine stop keeps the app alive
- **WHEN** the user triggers engine stop
- **THEN** the harness process is terminated, the launcher keeps running, and the engine reports `stopped`

#### Scenario: Engine start after stop
- **WHEN** the user triggers engine start while the engine is stopped
- **THEN** the harness is started on the effective port and the status returns to `running`

#### Scenario: Engine force-restart
- **WHEN** the user triggers force-restart
- **THEN** the harness child is killed, any process still listening on the port is terminated, and the harness is started again

#### Scenario: Stop persists across relaunch
- **WHEN** the user stops the engine and later quits and relaunches the app
- **THEN** the engine remains stopped on the next launch unless the user starts it
