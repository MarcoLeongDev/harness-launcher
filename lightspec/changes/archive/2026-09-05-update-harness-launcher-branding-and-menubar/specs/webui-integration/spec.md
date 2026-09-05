## ADDED Requirements

### Requirement: Latest Version Defaults in the Install Dropdown
The Versions tab install-version dropdown SHALL default its selection to the newest published harness version (the last entry of the ascending version list, or the known `latest` dist-tag when available) rather than the active version, and SHALL NOT offer a separate "Get latest" control. Because an uninstalled newest version renders without a ✓ checkmark, the panel SHALL make it immediately clear which version is worth downloading next.

#### Scenario: Versions page opens with an update available
- **WHEN** the user opens the Versions tab and the newest published version is not installed
- **THEN** the install dropdown is pre-selected to that newest version and its option shows no checkmark

#### Scenario: The newest version is already installed
- **WHEN** the newest published version is already installed
- **THEN** the dropdown still pre-selects it and its option shows the ✓ checkmark

### Requirement: Launcher Branding in Window Titles
Every runtime-visible launcher title SHALL use the brand "Harness Launcher" (never "DeepSeek Harness Launcher"): the harness web-view window title and Control Panel window title, the Control Panel `<title>` and header, the stopped page, the tray tooltip and Quit label, launcher notifications, the overlay panel meta, and the `web-dist` shell page.

#### Scenario: Open the harness window
- **WHEN** the harness web-view window is created
- **THEN** its title reads "Harness Launcher - Harness"

#### Scenario: Open the Control Panel
- **WHEN** the Control Panel window is created
- **THEN** its title reads "Harness Launcher - Control Panel"

#### Scenario: App is stopped
- **WHEN** the engine is stopped and the stopped page is shown
- **THEN** the page heading and window title read "Harness Launcher" (with the "Engine Stopped -" prefix on the title)

#### Scenario: Quit from the tray
- **WHEN** the user opens the tray menu
- **THEN** the tooltip and Quit item read "Harness Launcher" / "Quit Harness Launcher"