## 1. Icons & branding from the provided logo (0.1.1)
- [x] 1.1 Replace broken PNG decode in gen-icons.mjs with sips-based pipeline
- [x] 1.2 App icon (icon.png + tauri icon set) derives from logo/DSH Launcher.png
- [x] 1.3 Menu-bar icon = monochrome glyph of the logo mark (template image)
- [x] 1.4 brand/logo.png for the Control Panel derives from the logo
- [x] 1.5 Regenerate assets; verify non-black pixels; bump to 0.1.1

## 2. Engine status accuracy fix (0.1.2)
- [x] 2.1 Supervisor loop: idle timeout must not clear the child slot or emit stopped
- [x] 2.2 cargo test + review; bump to 0.1.2

## 3. Tabbed Control Panel (0.1.3)
- [x] 3.1 Tabs: Engine (status+controls+port), Versions (version/update/auto-update+progress), Logs
- [x] 3.2 Wire all existing commands into the tabbed layout
- [x] 3.3 JS syntax check; bump to 0.1.3

## 4. Design polish (0.1.4)
- [x] 4.1 Visual rework: status hero, cards, less text-heavy
- [x] 4.2 Accessibility (tab roles, focus), responsive
- [x] 4.3 bump to 0.1.4

## 5. Logo in the Control Panel (0.1.5)
- [x] 5.1 Header logo from brand/logo.png, rounded presentation
- [x] 5.2 bump to 0.1.5

## 6. Build, deploy & verify
- [x] 6.1 cargo build/test clean, javascript syntax checks pass
- [x] 6.2 lightspec validate --strict passes
- [x] 6.3 Release build + deploy to /Applications
- [x] 6.4 Relaunch app and confirm new version live; keep running