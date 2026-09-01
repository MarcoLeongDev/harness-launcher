# Change delta: Control Panel polish (webui-integration)

## MODIFIED Requirements
### Requirement: Settings Window
The launcher SHALL provide a dedicated native Settings window (opened from the tray or the overlay) that aggregates harness status, engine control, version & update management with live progress, port configuration, auto-update settings, and recent logs, served from a launcher-owned custom protocol origin (`dsh-ui://`) so it works in development and production without the harness server. The window SHALL organize its content into tabs — Engine (engine status, controls and port), Versions (version & update management, auto-update and install) and Logs (harness log tail) — using a visual, minimal, icon-driven design (macOS-style grouped rows) and SHALL display the provided brand logo in its header. The Engine tab SHALL surface the effective port exactly once, provide a single port field, and use icon-first engine controls (Start/Stop primary, Restart secondary) whose icons are re-rendered on every status update. The window SHALL follow the system appearance: launcher-owned pages SHALL provide light and dark palettes selected by `prefers-color-scheme`. The Versions tab SHALL be organised as: an auto-update section with the interval dropdown and enable toggle in one cell and the check-now action in another; the version table as its own section; an install section with the published-version dropdown and an icon-only cloud-download install button; and, while a version downloads, the terminal output as the only feedback surface with a circular icon-only cancel button overlaid on the terminal's bottom-right corner. The version table SHALL present the delete action as a circular red borderless button with a white icon, horizontally centred under the ACTIONS column header.

#### Scenario: Open settings window
- **WHEN** the user selects Settings from the tray menu or the overlay panel
- **THEN** a settings window opens (or focuses if already open) showing the tabbed control panel

#### Scenario: Engine tab icons render
- **WHEN** the engine phase changes while the Engine tab is visible
- **THEN** the status and Start/Stop icons render (no blank icon buttons)

#### Scenario: Auto-update section layout
- **WHEN** the user opens the Versions tab
- **THEN** the first section holds the auto-update interval dropdown and toggle in one cell with the Check-now action in another cell, followed by the version table section, followed by the install section

#### Scenario: Download feedback via terminal only
- **WHEN** a version download is in flight
- **THEN** the panel shows the live npm terminal output with a circular icon-only cancel button overlaid at the terminal's bottom-right, and no separate "Installing…/Verifying…" status row or bottom progress cell is shown

#### Scenario: Delete button styling
- **WHEN** the version table renders rows
- **THEN** each row's delete control is a circular red borderless button with a white trash icon centred under the ACTIONS header

#### Scenario: System appearance switch
- **WHEN** macOS switches between light and dark appearance while the window is visible
- **THEN** the Control Panel palette follows the system appearance
