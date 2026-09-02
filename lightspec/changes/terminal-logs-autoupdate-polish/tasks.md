# Tasks: terminal-logs-autoupdate-polish

## 1. Animate feed show/hide
- [x] 1.1 CSS: feed reveal (fade+slide) and collapse animations; prefers-reduced-motion
- [x] 1.2 JS: trigger animations on append/show and on finishSoon; v0.1.41

## 2. Logs tab: panel-filling log + terminal colour parity
- [ ] 2.1 Log fills panel height, scrolls internally; window never scrolls
- [ ] 2.2 Log lines coloured with the same palette as terminal feeds
- [ ] 2.3 Verify no window-level scrollbar on the Logs tab; v0.1.42

## 3. Feed title + terminal icon
- [ ] 3.1 No ellipsis truncation; full "npm install @deepseek-ai/dsh@<version>" title
- [ ] 3.2 terminal-fill icon in the feed header; v0.1.43

## 4. Auto-update section redesign
- [ ] 4.1 Remove empty cell; layout rethink (toggle + interval + check row)
- [ ] 4.2 "Check now" busy state + inline status text (no dead click); v0.1.44

## 5. Build, deploy & verify
- [ ] 5.1 cargo test green; lightspec validate pass
- [ ] 5.2 Release build + deploy + relaunch; version live; app left running