# Delta for WebUI Integration

## MODIFIED Requirements
### Requirement: Open in Browser icon
The engine card "open in browser" button SHALL use the `arrow.up.right.square`
glyph instead of the globe icon.

#### Scenario: engine card open button
- WHEN the control panel renders the engine card
- THEN the open-in-browser button shows the square arrow-out glyph

### Requirement: Merged Engine + Versions Panel
The control panel SHALL present Engine and Versions as one default section:
the engine table cell first, followed by the "Install version" group and then
the installed-versions table group (including operation feeds). The separate
Engine / Versions / Logs tab bar SHALL be removed.

#### Scenario: default panel
- WHEN the control panel opens
- THEN the merged engine+versions content is shown by default

### Requirement: Log Toggle
The control panel SHALL hide the log by default and SHALL provide a
bottom-left circular image-only toggle button (log icon; transparent
background and border, background only on hover; `aria-pressed`) that shows
and hides the log pane.

#### Scenario: toggle log visibility
- WHEN the user clicks the bottom-left log button
- THEN the log pane is shown; clicking again hides it
