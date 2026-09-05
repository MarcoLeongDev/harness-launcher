# Change: Terminal auto-dismiss, square stop glyph, white install icon

## Why
After stopping a download with the terminal stop button, the terminal feed
stays visible indefinitely: a cancelled feed dismisses only once per
terminal-phase event, and any late console line for the finished operation
re-creates the feed with no dismissal timer — a zombie terminal that never
goes away. Two small icon refinements were also requested: the stop button
is already circular, so the glyph should be a plain square (stop-fill, not
stop-circle), and the versions-table install button reads better with a
white cloud-download glyph like the other action buttons.

## What Changes
- Feed lifecycle: a feed whose operation reaches a terminal phase (done,
  failed, cancelled) schedules its dismissal exactly once and the operation
  key is marked closed; late console lines for closed operations no longer
  re-create feeds.
- Stop glyph: the feed stop button uses the plain square (stop-fill.svg,
  newly copied into resources/bootstrap-icons) instead of stop-circle.
- Install button: the cloud-download glyph renders white (dim backing disc
  in light mode, accent disc on hover — same pattern as the delete button).

## Impact
- Affected specs: webui-integration (feed dismissal, stop glyph, install
  button colour)
- Affected code: src-tauri/resources/settings.html, new resource
  resources/bootstrap-icons/stop-fill.svg, extended
  scripts/test-panel-feed.mjs (cancelled-feed lifecycle + late-line guard).
- User data/content safety: UI/timer-only; no engine, install or settings
  changes.
- Version: patch bump 0.1.57 -> 0.1.58.