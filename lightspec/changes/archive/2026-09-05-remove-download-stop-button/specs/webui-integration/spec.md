## REMOVED Requirements

### Requirement: Stop Button Only While Npm Runs
**Reason**: The stop button no longer exists on any terminal, so the
`stoppable` payload flag and its UI contract are removed.
**Migration**: Feeds show header + output for every operation phase; no
cancel control is offered. The dormant backend cancellation helpers are
deleted as well.

## MODIFIED Requirements

### Requirement: One Reusable Terminal Feed Per Operation
The Versions tab SHALL render operation output through a reusable feed
component — header (command + elapsed time) and a terminal body — of which
one instance exists per in-flight operation. Parallel version downloads MUST
each own their own feed (no shared output element), console lines MUST be
tagged with their operation so a feed only shows its own lines. Feeds persist
across panel refreshes (re-seeded from status + console snapshots) and the
area collapses when no operation is active. The feeds SHALL live in their own
section BELOW the versions table — never between the table's rows — and that
section MUST be visible only while it has content to display, disappearing
completely once the terminal output goes away. Terminals SHALL NOT render a
stop/cancel control.

#### Scenario: Two downloads run at the same time
- **WHEN** the user starts downloading two different versions one after the
  other while the first is still running
- **THEN** two independent feeds appear in the section below the versions
  table, each streaming only its own npm output with its own elapsed timer

#### Scenario: Panel opened mid-download
- **WHEN** the Control Panel is opened while a download is in flight
- **THEN** the feed for that download is rebuilt from status and console
  snapshots with its existing output

#### Scenario: No operation output
- **WHEN** no operation feed has content to display
- **THEN** the feeds section below the versions table is absent from the
  panel (it takes no space)

### Requirement: Animated Operation Feed
The operation feed terminal in the Versions tab SHALL animate its showing and
hiding (fade and slide) rather than appearing abruptly, and MUST respect the
user's reduced-motion preference by skipping the animation. A feed whose
operation reaches a terminal phase (done, failed, or cancelled) SHALL dismiss
itself after a short delay, and its dismissal MUST be scheduled exactly once
(repeat terminal events MUST NOT extend the wait). Once an operation is
closed, late console lines for its key MUST NOT re-create its feed.

#### Scenario: A download starts and finishes
- **WHEN** a version download begins
- **THEN** its feed animates in; when the operation settles the feed animates
  out and the area collapses

#### Scenario: Late console line after dismissal
- **WHEN** a console line tagged with an already-closed operation arrives
  after the feed was removed
- **THEN** no feed is re-created for it