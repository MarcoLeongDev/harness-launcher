# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| latest release on the `main` branch | ✅ |
| older tags / releases | ❌ (please update) |

The launcher is a fast-moving menubar app; only the most recent release is
supported with security fixes.

## Reporting a vulnerability

**Please do not open a public GitHub issue for security problems.**

1. Use GitHub's **private vulnerability reporting** for this repository
   (*Security → Report a vulnerability*), or
2. contact the maintainer directly if private reporting is unavailable.

Include as much of the following as you can:

- The affected version (commit hash or release tag, and the app version shown
  in the Control Panel).
- Step-by-step reproduction and, if possible, a proof of concept.
- The impact you believe it has (e.g. local privilege, data exposure, sandbox
  escape of harness-served content).

You can expect an initial response within **7 days**. We will credit reporters
in the release notes unless you prefer to stay anonymous.

## Scope

In scope:

- The Tauri shell itself: IPC surface (`src-tauri/src/commands.rs`), the
  `dsh-ui://` custom protocol, capability allow-lists, window/overlay
  injection (`overlay.js`), updater plugin configuration.
- Path handling and validation around version names, install directories and
  log/tail operations.
- Anything that lets harness-served (untrusted) content escape into the
  launcher's Rust core or the OS.

Out of scope:

- Vulnerabilities in the harness engine itself (`@deepseek-ai/dsh`) — report
  those to the harness project.
- A compromised local machine attacking its own loopback ports.

## Security model notes

- The launcher and the engine bind to **loopback only** (`127.0.0.1`). No
  network-facing listener is created by the launcher.
- No telemetry, no analytics, no remote configuration. All state lives under
  `~/Library/Application Support/ai.dsh.launcher/`.
- WebView → Rust access is gated by a Tauri capability file with an explicit
  command allow-list; harness-served content is treated as untrusted.
- The engine prints a per-process token at boot for its own API; the launcher
  never persists engine secrets.

## Hardening history

Security-relevant changes are tracked as regular releases; search the commit
log for `security`, `harden`, or see CHANGELOG.md for

