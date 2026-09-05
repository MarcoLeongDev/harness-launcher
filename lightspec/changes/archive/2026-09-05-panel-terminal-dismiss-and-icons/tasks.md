## 1. Feed dismiss + icon polish (v0.1.58)
- [x] 1.1 Mark closed operations (done/failed/cancelled); schedule feed
      dismissal once; late console lines for closed ops are ignored
- [x] 1.2 Stop glyph: stop-circle -> plain square (stop-fill.svg copied)
- [x] 1.3 Install button glyph white (light-mode backing disc, accent hover)
- [x] 1.4 Tests: cancelled feed dismisses after ~10s and is not resurrected
      by late console lines; structural checks for glyph + white icon
- [x] 1.5 npm test green; bump to 0.1.58; commit

## 3. Stop button only while npm runs (v0.1.59)
- [x] 3.1 Backend: stoppable flag on ProgressPayload; true only at real npm
      emits (install, throttled npm previews, peer completion)
- [x] 3.2 Frontend: stop visibility driven by the flag; hide on non-download
      phases while keeping output visible
- [x] 3.3 Tests: registry-only feed has no stop; installing shows stop;
      verifying hides stop; npm test green
- [x] 3.4 Bump to 0.1.59; commit

## 2. Build, deploy & verify
- [x] 2.1 Release build + deploy:relaunch to /Applications
- [x] 2.2 Verify live: v0.1.58 running, panel renders, feed dismiss behaviour