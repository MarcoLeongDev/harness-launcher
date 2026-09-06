# Delta for Menubar App Shell

## ADDED Requirements
### Requirement: Homebrew Cask Distribution
The launcher SHALL be installable via `brew install --cask` from the shared
`MarcoLeongDev/homebrew-tap` tap. Release builds SHALL produce a universal
DMG published to the GitHub and Gitee release pages; the cask SHALL pin the
version and SHA256 and serve the DMG over HTTPS. Source builds SHALL remain
available for contributors only.

#### Scenario: User installs via brew
- **WHEN** a user runs the documented brew tap + install commands
- **THEN** the launcher installs into /Applications and runs on both Apple
  Silicon and Intel Macs
