## MODIFIED Requirements
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
