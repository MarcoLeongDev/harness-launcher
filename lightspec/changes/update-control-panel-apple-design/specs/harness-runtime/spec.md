## MODIFIED Requirements
### Requirement: Harness Version Management
The launcher SHALL let the user list published versions of `@deepseek-ai/dsh` (from the npm registry), install any listed version, switch the active version, and roll back to any previously installed version. Installs SHALL reuse the shared npm cache and use cache-friendly flags (prefer-offline, no audit/fund) so repeated or neighbor-version installs are as fast as the registry allows, and SHALL report honest progress to the user (command, elapsed time, terminal lines, cancel).

#### Scenario: Default is latest
- **WHEN** the user opens the version selector with no explicit choice
- **THEN** the `latest` dist-tag version is preselected as the default

#### Scenario: Switch to a listed version
- **WHEN** the user selects a published version
- **THEN** the app installs it if absent (reusing the npm cache) and makes it the active version, while showing honest progress with elapsed time
