# Delta for WebUI Integration

## MODIFIED Requirements
### Requirement: Secure IPC Bridge
The launcher SHALL expose Rust commands to its windows through the Tauri IPC
layer gated by per-window capability files following least privilege: the
harness-content window SHALL receive only the WebView/event scope, while the
launcher-owned Control Panel window SHALL receive the full command set.
Authorization SHALL additionally be enforced in Rust by the source window of
every invocation: version-mutating, engine-lifecycle (beyond start),
port, settings and app-exit commands SHALL reject callers from the harness
window, and harness-served content SHALL be treated as untrusted. Launcher-
owned `dsh-ui://` pages SHALL carry a restrictive Content Security Policy.
The panel SHALL never hold secrets.

#### Scenario: Invoke from injected script
- **WHEN** the injected overlay calls a status read or another allow-listed command
- **THEN** the command result is returned to the overlay and unauthorized commands are rejected

#### Scenario: Harness content attempts a privileged command
- **WHEN** harness-served (untrusted) content invokes a version-mutating, process-signalling or app-lifecycle command
- **THEN** the invocation is rejected with a Control Panel pointer

## ADDED Requirements
### Requirement: Control Panel Output Sanitization
The Control Panel SHALL render all registry- or engine-supplied strings
(version names, log lines, terminal output) as text, never as HTML, and SHALL
redact launch-token credentials (`token=…`, bearer values) in every log and
terminal surface including the overlay viewer.

#### Scenario: Malicious version name displayed
- **WHEN** a version string contains HTML or script markup
- **THEN** it is displayed as inert text and no markup executes

#### Scenario: Token-bearing log line displayed
- **WHEN** a log or terminal line contains a launch token
- **THEN** every viewer shows the redacted form, never the raw token

## ADDED Requirements
### Requirement: Custom-Protocol Content Security Policy
Launcher-owned pages served over `dsh-ui://` SHALL carry a restrictive
Content Security Policy (scripts from self only, no object embeds), so a
single injection sink cannot escalate to full script execution.

#### Scenario: CSP present on launcher pages
- **WHEN** the Control Panel or stopped page is served
- **THEN** the response includes a restrictive Content-Security-Policy
