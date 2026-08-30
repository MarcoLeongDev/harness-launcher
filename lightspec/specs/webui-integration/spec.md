# webui-integration Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Embedded Harness WebUI
The launcher SHALL display the harness WebUI in a native window loading `http://127.0.0.1:<port>` where the port is the active configured port, and SHALL reload the page when the port changes.

#### Scenario: Port change reload
- **WHEN** the active port changes and the harness restarts
- **THEN** the WebView navigates to the new localhost URL

### Requirement: Bottom-Left Overlay Panel
The launcher SHALL render a floating control panel over the bottom-left corner of the harness WebUI inside the window. The panel SHALL show harness status and active version, and SHALL provide controls for: selecting/installing a version (defaulting to latest), updating to the latest version, rolling back to a previously installed version, changing the port, toggling automatic update checks, viewing recent harness logs, reopening in the system browser, and quitting the app. The panel SHALL additionally expose engine start/stop controls and a live progress indicator for in-flight operations, and SHALL not render on launcher-owned pages (`dsh-ui:`).

#### Scenario: Open panel
- **WHEN** the user clicks the launcher button at the bottom-left of the WebUI
- **THEN** the panel expands with status, version selector, update, rollback, port, auto-update, logs, engine controls, progress and quit controls

#### Scenario: Panel survives navigation
- **WHEN** the user navigates within the harness SPA
- **THEN** the overlay remains visible and functional

#### Scenario: Overlay skips launcher pages
- **WHEN** a page served from the `dsh-ui:` protocol is displayed in the main window
- **THEN** the overlay does not inject a second control panel

### Requirement: Secure IPC Bridge
The launcher SHALL expose Rust commands to the panel through the Tauri IPC layer gated by a capability file, with `withGlobalTauri` enabled for the injected script; the panel SHALL never hold secrets and SHALL treat harness-served content as untrusted.

#### Scenario: Invoke from injected script
- **WHEN** the injected overlay calls a Rust command
- **THEN** the command result is returned to the overlay and unauthorized commands are rejected

### Requirement: Settings Window
The launcher SHALL provide a dedicated native Settings window (opened from the tray or the overlay) that aggregates harness status, engine control, version & update management with live progress, port configuration, auto-update settings, and recent logs, served from a launcher-owned custom protocol origin (`dsh-ui://`) so it works in development and production without the harness server. The window SHALL organize its content into tabs — Engine (engine status, controls and port), Versions (version & update management, auto-update and progress) and Logs (harness log tail) — using a visual, minimal, icon-driven design (macOS-style grouped rows) and SHALL display the provided brand logo in its header. The Engine tab SHALL surface the effective port exactly once ("Runs at http://127.0.0.1:<port>"), provide a single port field, and use icon-first engine controls (Start/Stop primary, Restart secondary, Force subdued). The Logs tab SHALL NOT repeat a "Logs" label and SHALL present the log output with a minimal icon toolbar.

#### Scenario: Open settings window
- **WHEN** the user selects Settings from the tray menu or the overlay panel
- **THEN** a settings window opens (or focuses if already open) showing the tabbed control panel

#### Scenario: Engine tab port authority
- **WHEN** the engine is running
- **THEN** the Engine tab shows exactly one running address ("Runs at http://127.0.0.1:3080") and one port field, with no duplicated port text elsewhere

#### Scenario: Logs tab
- **WHEN** the user opens the Logs tab
- **THEN** the tab shows the recent harness output with icon-only toolbar actions (refresh, follow), without a redundant title

