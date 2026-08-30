## MODIFIED Requirements
### Requirement: Menubar-Only Presence
The launcher SHALL run as a macOS menu-bar (tray) application: it SHALL show a tray icon, SHALL NOT show a Dock icon, and SHALL keep running when all windows are closed, so closing the window never quits the app. The tray icon SHALL be a monochrome template glyph derived from the provided brand logo, and the application icon SHALL be derived from the provided brand logo at `logo/DSH Launcher.png`.

#### Scenario: Launch
- **WHEN** the app launches
- **THEN** a tray icon appears and no Dock icon is shown

#### Scenario: Branded icons
- **WHEN** the app is installed with the provided logo present in the repository
- **THEN** the Finder/Dock application icon and the menu-bar icon visually match the provided logo (or its mark)
