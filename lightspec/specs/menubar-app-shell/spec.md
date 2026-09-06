# menubar-app-shell Specification

## Purpose
TBD - created by archiving change add-dsh-launcher. Update Purpose after archive.
## Requirements
### Requirement: Menubar-Only Presence
The launcher SHALL run as a macOS menu-bar (tray) application: it SHALL show a tray icon, SHALL NOT show a Dock icon, and SHALL keep running when all windows are closed, so closing the window never quits the app. The tray icon SHALL be a monochrome template glyph derived from the provided brand logo, and the application icon SHALL be derived from the provided brand logo at `logo/DSH Launcher.png`.

#### Scenario: Launch
- **WHEN** the app launches
- **THEN** a tray icon appears and no Dock icon is shown

#### Scenario: Branded icons
- **WHEN** the app is installed with the provided logo present in the repository
- **THEN** the Finder/Dock application icon and the menu-bar icon visually match the provided logo (or its mark)

### Requirement: Tray Menu Controls
The launcher SHALL provide a tray menu offering at least: show/hide the window, open the settings window, start/stop/restart the engine, open the harness in the default browser, and quit the app. Start/Stop/Restart items SHALL be enabled or disabled according to the current engine state. The tray menu SHALL NOT offer a force-restart or an update-harness item. Every tray menu item SHALL carry a native macOS menu glyph (template icon) representing its action.

#### Scenario: Quit from tray
- **WHEN** the user selects Quit from the tray menu
- **THEN** the harness process is stopped and the app exits

#### Scenario: Reopen window
- **WHEN** the user selects Open while the window is hidden
- **THEN** the window is shown and refocused

#### Scenario: Engine controls reflect state
- **WHEN** the engine is running, Stop/Restart are enabled and Start is disabled; when stopped, Start is enabled and Stop/Restart are disabled
- **THEN** the tray items reflect the current engine phase

#### Scenario: Menu items are glyph-labelled
- **WHEN** the user opens the tray menu
- **THEN** every item shows a native macOS icon glyph alongside its text

### Requirement: Hidden-Window Lifecycle
The launcher SHALL intercept window close requests and hide instead of closing, and SHALL quit only through the explicit tray Quit action (or an in-app quit control), guaranteeing no orphaned harness processes.

#### Scenario: Close button
- **WHEN** the user presses the window close button
- **THEN** the window is hidden and the process continues

### Requirement: Homebrew Cask Distribution
The launcher SHALL be installable via `brew install --cask` from the shared
`MarcoLeongDev/homebrew-tap` tap. Release builds SHALL produce a universal
DMG published to the GitHub and Gitee release pages; the cask SHALL pin the
version and SHA256 and serve the DMG over HTTPS. Source builds SHALL remain
available for contributors only.

#### Scenario: User installs via brew
- **WHEN** a user runs the documented brew tap + install commands
- **THEN** the launcher installs into /Applications and runs on both Apple
  Silicon and Intel Macs

