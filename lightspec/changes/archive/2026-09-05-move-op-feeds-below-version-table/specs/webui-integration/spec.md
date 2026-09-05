## MODIFIED Requirements

### Requirement: One Reusable Terminal Feed Per Operation
The Versions tab SHALL render operation output through a reusable feed
component — header (command + elapsed time), terminal body, and a circular
stop-button overlay — of which one instance exists per in-flight operation.
Parallel version downloads MUST each own their own feed (no shared output
element), console lines MUST be tagged with their operation so a feed only
shows its own lines, and cancellation MUST target a single operation. Feeds
persist across panel refreshes (re-seeded from status + console snapshots)
and the area collapses when no operation is active. The feeds SHALL live in
their own section BELOW the versions table — never between the table's rows
— and that section MUST be visible only while it has content to display,
disappearing completely once the terminal output goes away.

#### Scenario: Two downloads run at the same time
- **WHEN** the user starts downloading two different versions one after the
  other while the first is still running
- **THEN** two independent feeds appear in the section below the versions
  table, each streaming only its own npm output with its own elapsed timer

#### Scenario: Stop one of two downloads
- **WHEN** the user taps the stop button on the first download's feed
- **THEN** only that download is cancelled and the second continues unaffected

#### Scenario: Panel opened mid-download
- **WHEN** the Control Panel is opened while a download is in flight
- **THEN** the feed for that download is rebuilt from status and console
  snapshots with its existing output

#### Scenario: No operation output
- **WHEN** no operation feed has content to display
- **THEN** the feeds section below the versions table is absent from the
  panel (it takes no space)

### Requirement: Merged Install Row in the Versions Table
The Control Panel SHALL fold installation into the versions table: the
standalone "Install version" group SHALL be removed, the header column SHALL
be labelled "Directory" (not "Folder"), and the table SHALL always end with
the install row — an empty Default cell, the published-version dropdown in
the Version column, an empty Directory cell, and a cloud-download install
button in the Actions column. The install row MUST remain the table's last
row — operation output renders in its own section below the table group,
never between the table's rows — and the install row stays present when no
versions are installed. Existing element IDs and IPC behaviour MUST be
preserved so downloads, switching and deletion keep working.

#### Scenario: No versions installed
- **WHEN** no harness versions are installed yet
- **THEN** the table still shows the install row with a usable dropdown and
  install button

#### Scenario: Download in flight
- **WHEN** a version is downloading (an operation feed is visible in the
  section below the table)
- **THEN** the install row remains the last row of the table itself