# Change delta: Authenticated harness window URL (menubar-app-shell)

## MODIFIED Requirements
### Requirement: Embedded Harness WebUI
The launcher SHALL display the harness WebUI in a native window whose URL is
the authenticated URL the active harness engine prints at boot
(`http://127.0.0.1:<port>/?token=…`) when the engine publishes one, and the
plain `http://127.0.0.1:<port>` otherwise. The launcher SHALL re-capture the
printed URL on every engine start and SHALL navigate the window, the tray
"Open Harness" action and "Open in Browser" to the captured URL. When the port
changes the window SHALL follow the new URL. Launcher windows SHALL keep the
native macOS titlebar that follows the system appearance; no window theme may
be forced.

#### Scenario: Token engine window opens
- **WHEN** a harness version that prints `dsh web: http://127.0.0.1:<port>/?token=…`
  finishes booting and the launcher opens the harness window
- **THEN** the window navigates to the captured authenticated URL and the WebUI
  renders instead of the 401 "authentication required" page

#### Scenario: Legacy engine window opens
- **WHEN** a harness version prints no token URL finishes booting
- **THEN** the window opens the plain `http://127.0.0.1:<port>` URL

#### Scenario: Restart re-authenticates
- **WHEN** the engine restarts (new process, new launch token)
- **THEN** the launcher captures the freshly printed URL and navigates the
  window to it

#### Scenario: Port change reload
- **WHEN** the active port changes and the harness restarts
- **THEN** the WebView navigates to the new port's authenticated URL

#### Scenario: Titlebar follows system theme
- **WHEN** the macOS system appearance changes between light and dark
- **THEN** the launcher window titlebar follows the system appearance and
  launcher-owned pages adapt their palette to match
