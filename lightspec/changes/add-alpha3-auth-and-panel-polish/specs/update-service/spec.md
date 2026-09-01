# Change delta: Install feedback surface (update-service)

## ADDED Requirements
### Requirement: Terminal-Only Install Feedback
While a harness version downloads, the Control Panel SHALL present the live
npm console output as the only progress surface, with a circular icon-only
cancel control overlaid on the terminal's bottom-right corner; the panel SHALL
NOT render a separate textual "Installing…/Verifying installation…" status row
or a bottom progress/phase cell for these operations.

#### Scenario: Download in flight
- **WHEN** an install/update/rollback download is running
- **THEN** only the terminal output (with the circular cancel overlay) is
  shown for the operation

#### Scenario: Cancel a download
- **WHEN** the user clicks the circular overlay button during a download
- **THEN** the running npm child is asked to stop and the operation aborts
