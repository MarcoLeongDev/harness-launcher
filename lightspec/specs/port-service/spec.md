# port-service Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Selectable Localhost Port
The launcher SHALL let the user select the port on which the harness is exposed at `http://127.0.0.1:<port>`, SHALL default to 3080, SHALL persist the selection, and SHALL validate the port is in the valid range (1–65535). Changing the port SHALL proceed with visible progress, SHALL wait for the harness to actually serve on the new port before reloading the WebView, and SHALL revert the selection transparently if the restart or serve check fails.

#### Scenario: Default port
- **WHEN** the app is first launched
- **THEN** the harness listens on 127.0.0.1:3080 unless the user has changed it

#### Scenario: Change port
- **WHEN** the user enters a new port and applies it
- **THEN** the setting persists, the harness restarts on the new port with progress shown, and the WebView reloads once the harness answers on the new port

#### Scenario: Port change failure
- **WHEN** the harness fails to serve on the requested port after restart
- **THEN** the launcher reverts to the previous effective port and reports the error

### Requirement: Free-Port Fallback
The launcher SHALL detect that the configured port is already in use on loopback and SHALL pick the next free port, report the actual port to the user, and remember it for the session.

#### Scenario: Port busy
- **WHEN** the configured port is occupied
- **THEN** the harness starts on the next free port and the UI shows the actual port in use

### Requirement: Loopback-Only Exposure
The launcher SHALL run the harness bound to 127.0.0.1 only, SHALL NOT pass `--host 0.0.0.0`, and SHALL never expose the harness to the network.

#### Scenario: Exposure check
- **WHEN** the harness is launched
- **THEN** it binds to 127.0.0.1 and is reachable only via localhost

