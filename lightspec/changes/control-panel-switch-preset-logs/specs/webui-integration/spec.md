## ADDED Requirements
### Requirement: One Reusable Terminal Feed Per Operation
The Versions tab SHALL render operation output through a reusable feed
component — header (command + elapsed time), terminal body, and a circular
stop-button overlay — of which one instance exists per in-flight operation.
Parallel version downloads MUST each own their own feed (no shared output
element), console lines MUST be tagged with their operation so a feed only
shows its own lines, and cancellation MUST target a single operation. Feeds
persist across panel refreshes (re-seeded from status + console snapshots)
and the area collapses when no operation is active.

#### Scenario: Two downloads run at the same time
- **WHEN** the user starts downloading two different versions one after the
  other while the first is still running
- **THEN** two independent feeds appear, each streaming only its own npm
  output with its own elapsed timer

#### Scenario: Stop one of two downloads
- **WHEN** the user taps the stop button on the first download's feed
- **THEN** only that download is cancelled and the second continues unaffected

#### Scenario: Panel opened mid-download
- **WHEN** the Control Panel is opened while a download is in flight
- **THEN** the feed for that download is rebuilt from status and console
  snapshots with its existing output
