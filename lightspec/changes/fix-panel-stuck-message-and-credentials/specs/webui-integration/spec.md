## ADDED Requirements

### Requirement: Stable Operation Key For An Operation's Lifetime
Every launcher operation SHALL stream all of its progress phases under ONE
operation key, from its first emitted phase through its terminal phase
(done, failed, or cancelled). The backend MUST NOT switch an operation's
key mid-flight: a feed created by an earlier phase would never observe a
terminal phase under the new key, so it would linger in the downloading
state, fire the 60s stuck-watch against a completed operation, and keep a
stop button that cancels a key nothing is running under.

#### Scenario: Update while the engine is running
- **WHEN** the user runs "Get latest" while the engine is running and the
  resolved version differs from the active one
- **THEN** the registry, stopping, installing, verifying and terminal
  phases all arrive under the same op key
- **AND** the operation's feed closes shortly after the done phase
- **AND** no "no progress for 60s" line is appended after the done phase

#### Scenario: Genuinely stalled download
- **WHEN** a download is in flight and emits no progress message for 60
  seconds
- **THEN** the feed surfaces the stuck-watch hint once so the user can
  cancel it
