## 1. Radio switch is the designed way to switch
- [x] 1.1 commands.rs: switch_version_inner installs (when missing), refuses while the engine is running, records previous_version, sets current_version, refreshes tray
- [x] 1.2 Feedback messages: "installed vX and set it as the default version" / "vX is now the default version — press Start to run it" / "vX is already the default version"; no more "select it from the Control Panel" (switch_feedback + 2 unit tests)
- [x] 1.3 cargo tests / manual check: radio click switches default when stopped; blocked with clear error while running (cargo test 19 pass; live check after deploy)

## 2. Delete button styling
- [x] 2.1 settings.html: .btn.del white glyph default; hover = red background + white glyph (theme-aware backing in light mode)
- [x] 2.2 Manual check in dark and light appearance (live check after deploy)

## 3. Status column → folder link
- [x] 3.1 settings.html: drop the Status checkmark column; add per-row folder link column (Folder) with folder2-open icon
- [x] 3.2 versions.rs: is_valid_version_name guard + unit tests (path traversal)
- [x] 3.3 commands.rs: open_version_dir command (opens the version directory in Finder); register in lib.rs
- [x] 3.4 Manual check: link opens the correct directory; refuses unknown/traversal names (cargo test 21 pass; live check after deploy)

## 4. Notifications as terminal cell of the table section
- [x] 4.1 settings.html: remove op-result cells from the install section; append terminal-style feed as the last cell of the versions-table group (v0.1.23 styling: header with npm command + elapsed, mono output lines)
- [x] 4.2 Wire progress + console + result notifications into the feed; auto-hide when idle; show only when there is something to display (progress/result strings unified so no duplicate lines)
- [x] 4.3 Stop-button overlay (circular ✕) visible only while a version download is in flight; calls cancel_operation
- [x] 4.4 Manual check: install shows live terminal + stop overlay; result messages appear in the feed; feed hides when idle (live check after deploy)

## 5. Build, deploy & verify
- [x] 5.1 cargo check + cargo test + lightspec validate pass (21 tests, validate clean)
- [x] 5.2 Patch bumps per item (0.1.30 → 0.1.33) committed one per item
- [x] 5.3 Release build + deploy to /Applications; relaunch; confirm version live (0.1.33 in Info.plist, process running); keep the app running
