## ADDED Requirements

### Requirement: Embedded Harness WebUI
The launcher SHALL display the harness WebUI in a native window loading `http://127.0.0.1:<port>` where the port is the active configured port, and SHALL reload the page when the port changes.

#### Scenario: Port change reload
- **WHEN** the active port changes and the harness restarts
- **THEN** the WebView navigates to the new localhost URL

### Requirement: Bottom-Left Overlay Panel
The launcher SHALL render a floating control panel over the bottom-left corner of the harness WebUI inside the window. The panel SHALL show harness status and active version, and SHALL provide controls for: selecting/installing a version (defaulting to latest), updating to the latest version, rolling back to a previously installed version, changing the port, toggling automatic update checks, viewing recent harness logs, reopening in the system browser, and quitting the app.

#### Scenario: Open panel
- **WHEN** the user clicks the launcher button at the bottom-left of the WebUI
- **THEN** the panel expands with status, version selector, update, rollback, port, auto-update, logs, open-in-browser and quit controls

#### Scenario: Panel survives navigation
- **WHEN** the user navigates within the harness SPA
- **THEN** the overlay remains visible and functional

### Requirement: Secure IPC Bridge
The launcher SHALL expose Rust commands to the panel through the Tauri IPC layer gated by a capability file, with `withGlobalTauri` enabled for the injected script; the panel SHALL never hold secrets and SHALL treat harness-served content as untrusted.

#### Scenario: Invoke from injected script
- **WHEN** the injected overlay calls a Rust command
- **THEN** the command result is returned to the overlay and unauthorized commands are rejected
