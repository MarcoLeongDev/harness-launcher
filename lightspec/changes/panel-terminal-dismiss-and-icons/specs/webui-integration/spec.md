## MODIFIED Requirements

### Requirement: Animated Operation Feed
The operation feed terminal in the Versions tab SHALL animate its showing and
hiding (fade and slide) rather than appearing abruptly, and MUST respect the
user's reduced-motion preference by skipping the animation. A feed whose
operation reaches a terminal phase (done, failed, or cancelled — including a
download stopped by the user) SHALL dismiss itself after a short delay, and
its dismissal MUST be scheduled exactly once (repeat terminal events MUST
NOT extend the wait). Once an operation is closed, late console lines for
its key MUST NOT re-create its feed.

#### Scenario: A download starts and finishes
- **WHEN** a version download begins
- **THEN** its feed animates in; when the operation settles the feed animates
  out and the area collapses

#### Scenario: User stops a download
- **WHEN** the user taps the stop button and the operation reports
  cancelled
- **THEN** the terminal stays briefly (to show the cancellation) and then
  animates away on its own, without lingering indefinitely

#### Scenario: Late console line after dismissal
- **WHEN** a console line tagged with an already-closed operation arrives
  after the feed was removed
- **THEN** no feed is re-created for it

### Requirement: Merged Install Row in the Versions Table
The Control Panel SHALL fold installation into the versions table: the
standalone "Install version" group SHALL be removed, the header column SHALL
be labelled "Directory" (not "Folder"), and the table SHALL always end with
the install row — an empty Default cell, the published-version dropdown in
the Version column, an empty Directory cell, and a cloud-download install
button in the Actions column rendered with a white glyph (dim backing disc
in light mode, accent disc on hover). The install row MUST remain the
table's last row — operation output renders in its own section below the
table group, never between the table's rows — and the install row stays
present when no versions are installed. Existing element IDs and IPC
behaviour MUST be preserved so downloads, switching and deletion keep
working.

#### Scenario: No versions installed
- **WHEN** no harness versions are installed yet
- **THEN** the table still shows the install row with a usable dropdown and
  install button

#### Scenario: Download in flight
- **WHEN** a version is downloading (an operation feed is visible in the
  section below the table)
- **THEN** the install row remains the last row of the table itself

## ADDED Requirements

### Requirement: Square Stop Glyph on the Feed Stop Button
The operation feed stop button SHALL render a plain filled square glyph
(stop-fill) rather than a stop-circle glyph, because the button itself is
already circular.

#### Scenario: Feed stop button renders
- **WHEN** a download feed is in the downloading state
- **THEN** the circular stop button shows the filled-square glyph