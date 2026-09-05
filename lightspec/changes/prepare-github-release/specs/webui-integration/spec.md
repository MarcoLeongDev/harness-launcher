## MODIFIED Requirements
### Requirement: Settings Window
The Control Panel settings window SHALL present the launcher's configuration
through ONE merged view (no tab bar): an engine status card with start/stop/
restart and port controls, an update banner, and a versions table. The
versions table SHALL use the columns Active | Version | Directory | Actions
(radio select, version, open-directory button, delete button), and its LAST
row SHALL always be the install row — the published-version dropdown in the
Version column and a cloud-download install button in the Actions column with
the Active and Directory cells empty. A live log SHALL be available in a
bottom drawer toggled by a circular image-only button pinned to the
bottom-right of the window. All state and actions MUST survive launcher
relaunches; user data is never deleted by redeployment.

#### Scenario: Open the settings window
- **WHEN** the user opens the settings window
- **THEN** the merged view is shown with the live engine status and the
  versions table (Active column header, install row last, log toggle at the
  bottom-right)

## ADDED Requirements
### Requirement: Minimal Control Panel Header
The Control Panel header SHALL show the brand logo, the product title, and
the launcher version as subtle text beside the title, and SHALL NOT render an
engine version tag beside the power (quit) button.

#### Scenario: Header renders minimally
- **WHEN** the Control Panel window is opened
- **THEN** the header shows logo, title and subtle launcher version, and no
  "dsh vX.Y.Z" tag appears next to the power button
