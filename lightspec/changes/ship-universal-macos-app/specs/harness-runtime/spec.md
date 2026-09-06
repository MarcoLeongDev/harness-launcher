# Delta for Harness Runtime

## MODIFIED Requirements
### Requirement: Bundled Node and NPM
The launcher SHALL bundle a Node.js runtime as a Tauri sidecar binary and a vendored copy of the npm CLI, and SHALL run install, update, rollback and execution of the harness exclusively with those bundled tools, never requiring system node, npm or npx. On macOS the app SHALL ship as a universal binary (Apple Silicon + Intel): the build SHALL fetch, checksum-verify and bundle the Node sidecar for both architectures, and the default release build SHALL target `universal-apple-darwin`.

#### Scenario: Fresh machine without system Node
- **WHEN** the app runs on a machine that has no system Node
- **THEN** harness install and launch still succeed using the bundled Node and vendored npm

#### Scenario: Intel Mac
- **WHEN** the app runs on an Intel Mac
- **THEN** it launches and manages the engine natively via the x86_64 slices

#### Scenario: npx-style invocations
- **WHEN** the runtime needs npx semantics (running package binaries)
- **THEN** the app invokes the vendored npm CLI via `node <npm-cli.js> exec ...`
