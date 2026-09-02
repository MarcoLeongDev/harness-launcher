## ADDED Requirements
### Requirement: Download Without Stopping the Engine
Installing (downloading) a harness version MUST NOT require stopping a running
engine, MUST NOT change the active (default) version, and MUST NOT start, stop
or restart the engine. A version that is already installed SHALL be reported as
such without redundant work.

#### Scenario: Download a new version while the engine runs
- **WHEN** the user installs a version that is not present locally while the
  engine is running
- **THEN** the download completes, the running engine keeps running the
  previously active version, and the version appears as installed

#### Scenario: Download an already-installed version
- **WHEN** the user installs a version that is already installed
- **THEN** the launcher reports it is already installed without reinstalling

### Requirement: Live Version Switching
Switching the default version SHALL work while the engine is running: the
launcher MUST stop the running engine, install the requested version when
missing, record it as the default, and start the new version again, surfacing
each phase as progress. When the engine is stopped, switching only records the
default (it never auto-starts), matching the stopped-state semantics.

#### Scenario: Switch default version while the engine runs
- **WHEN** the user picks a different default version while the engine is
  running
- **THEN** the engine is stopped, the chosen version becomes the default, and
  the engine starts again running that version

#### Scenario: Switch default version while stopped
- **WHEN** the user picks a different default version while the engine is
  stopped
- **THEN** the chosen version becomes the default and the engine remains
  stopped
