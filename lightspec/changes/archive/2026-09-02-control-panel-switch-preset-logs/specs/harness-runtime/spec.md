## ADDED Requirements
### Requirement: Cross-Version Agent Preset Compatibility
Before starting an installed harness engine, the launcher SHALL inspect the
agent presets that engine version ships (alpha `dsh-agent-presets/presets`
roster and rc `dsh/config/agent-presets` layouts) and reconcile the shared
harness home (`~/.dsh`) with them: an `agent-presets.default` value in
`~/.dsh/settings.yaml` that the engine does not ship MUST be rewritten to an
available preset (prefer `standard` for the legacy `code` default), and when
the engine lacks a `code` preset while sessions recorded under rc engines
reference it, the launcher MUST provide a user-root `code` preset shim (a copy
of that engine's `standard` composition) so those sessions resume. The launcher
MUST NOT delete or rewrite user session data or unrelated settings.

#### Scenario: Upgrade from rc to alpha with a rc-era default preset
- **WHEN** the launcher starts an alpha engine while `~/.dsh/settings.yaml`
  records `agent-presets.default: code` and the alpha engine ships no `code`
  preset
- **THEN** the recorded default is rewritten to `standard` before the engine
  spawns and new sessions create successfully

#### Scenario: Resume a session created under an rc engine
- **WHEN** a session recorded with `agentPreset: code` is resumed under an
  engine that ships no `code` preset
- **THEN** the launcher has installed a user-root `code` preset shim so the
  session resumes instead of failing with "preset code not found"

#### Scenario: Downgrade back to an rc engine
- **WHEN** an rc engine that ships its own `code` preset starts
- **THEN** the shipped preset wins over the user-root shim and nothing is
  overwritten
