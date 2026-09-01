## Context
Harness `dsh web` (0.1.2-alpha.2+) mints a per-process launch token and serves
the WebUI only after a `GET /?token=<token>` exchange that sets a signed,
persistent cookie (dsh-client-connection BrowserAuth). The token is printed on
stdout as `dsh web: <url>` (optionally followed by ` (LAN: …)`). Older
releases print the plain URL. The launcher already streams harness stdout into
its log tail.

## Goals / Non-Goals
- Goals: open the WebUI window with a working URL on every harness release
  shape; remove Control Panel chrome the user flagged as dead; keep old-engine
  compatibility.
- Non-Goals: no change to the harness itself; no cookie/secret management in
  the launcher (the WebView handles the cookie natively); no custom titlebar.

## Decisions
- Decision: parse the token URL from harness stdout at the source (runtime log
  pump) instead of querying an internal API — the printed line is the
  documented, stable contract ("reopen the URL printed by dsh web").
  Alternative (screen-scraping the WebUI or disabling auth via flags) rejected:
  no supported disable flag exists and the printed URL is authoritative.
- Decision: `harness_web_url(port, wait)` waits up to ~10 s for the captured
  URL on start/restart paths (called from blocking contexts only) and falls
  back to the plain URL so pre-token engines keep working.
- Decision: keep the native macOS titlebar (it follows the system when no
  theme is forced — verified in tao 0.35.3 `set_ns_theme(None)` semantics) and
  make launcher pages follow the system via `prefers-color-scheme` so the
  window adapts as a unit.
- Decision: the download terminal becomes the single install feedback surface
  (per user request); a stuck download is visible as silent terminal output,
  and the circular overlay button cancels it.

## Risks / Trade-offs
- If a future harness renames the `dsh web:` line the launcher falls back to
  the plain URL (degraded but functional, matching pre-alpha behaviour).
- The wait window (10 s) adds latency only on the start/restart code paths
  that already wait for the port to serve.
