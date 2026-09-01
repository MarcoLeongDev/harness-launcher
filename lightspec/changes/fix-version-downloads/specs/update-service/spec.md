# Change delta: Reliable Version Installation (update-service)

## ADDED Requirements
### Requirement: Reliable Version Installation
The launcher SHALL install any published harness version (stable, rc, or alpha)
from the npm registry without silent failure. Installation MUST use the
legacy-peer-deps flag so peer-dependency conflicts in the package tree do not
cause arborist to skip reify, and MUST NOT force prefer-offline so package
metadata is always resolved fresh from the registry (preventing ETARGET errors
when a new release depends on a version the cached metadata does not list). A
failed or zero-package install MUST surface a clear, actionable error instead of
leaving the UI in a perpetual installing state.

#### Scenario: Install a freshly published pre-release
- **WHEN** the user installs a just-published version (for example an alpha or rc)
  whose dependencies reference versions not present in a stale cache
- **THEN** the launcher resolves fresh metadata and installs the full tree
  successfully

#### Scenario: Install a tree with peer-dependency conflicts
- **WHEN** the selected version dependency tree has peer-dependency conflicts
- **THEN** the launcher installs with legacy-peer-deps and the harness ends up
  usable (bin present), rather than producing zero packages

#### Scenario: Install produces no usable harness
- **WHEN** an install completes but the harness package directory is missing
- **THEN** the launcher reports a clear error naming the likely cause instead of
  hanging on verifying
