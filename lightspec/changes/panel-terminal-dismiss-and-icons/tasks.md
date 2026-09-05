## 1. Feed dismiss + icon polish (v0.1.58)
- [x] 1.1 Mark closed operations (done/failed/cancelled); schedule feed
      dismissal once; late console lines for closed ops are ignored
- [x] 1.2 Stop glyph: stop-circle -> plain square (stop-fill.svg copied)
- [x] 1.3 Install button glyph white (light-mode backing disc, accent hover)
- [x] 1.4 Tests: cancelled feed dismisses after ~10s and is not resurrected
      by late console lines; structural checks for glyph + white icon
- [x] 1.5 npm test green; bump to 0.1.58; commit

## 2. Build, deploy & verify
- [ ] 2.1 Release build + deploy:relaunch to /Applications
- [ ] 2.2 Verify live: v0.1.58 running, panel renders, feed dismiss behaviour