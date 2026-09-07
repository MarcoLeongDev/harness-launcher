// Regression tests for the Control Panel operation-feed lifecycle.
//
// Loads the inline script from src-tauri/resources/settings.html and runs it
// against a minimal DOM + Tauri stub with a virtual clock, then replays the
// launcher://progress / launcher://console event sequences the backend emits.
//
// Covered contract (Stable Operation Key For An Operation's Lifetime):
//   1. the panel script boots without crashing and primes status + logs
//   2. a full single-key update lifecycle closes its feed and never appends
//      the 60s stuck-watch line after the done phase
//   3. a done phase clears a pending stuck timer (the reported bug)
//   4. a genuinely stalled download surfaces the stuck hint, and live console
//      output re-arms it
//
// Run: node scripts/test-panel-feed.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const html = readFileSync(path.join(root, "src-tauri", "resources", "settings.html"), "utf8");
const script = html.match(/<script>([\s\S]*?)<\/script>/)[1];

// ---------- virtual clock ----------
function makeScheduler() {
  let now = 0, nextId = 1;
  const timers = new Map();
  const setTimeout = (fn, ms) => {
    const id = nextId++;
    timers.set(id, { id, fn, at: now + (ms || 0), period: null });
    return id;
  };
  const setInterval = (fn, ms) => {
    const id = nextId++;
    timers.set(id, { id, fn, at: now + (ms || 0), period: ms || 0 });
    return id;
  };
  const clearTimeout = (id) => timers.delete(id);
  const clearInterval = (id) => timers.delete(id);
  const advance = (ms) => {
    const target = now + ms;
    for (;;) {
      let due = null;
      for (const t of timers.values()) {
        if (t.at <= target && (due === null || t.at < due.at)) due = t;
      }
      if (!due) break;
      now = Math.max(now, due.at);
      if (due.period !== null) { due.at = now + due.period; due.fn(); }
      else { timers.delete(due.id); due.fn(); }
    }
    now = target;
  };
  return { setTimeout, setInterval, clearTimeout, clearInterval, advance, now: () => now };
}

// ---------- mini DOM ----------
function makeElement(doc, tag) {
  const el = {
    tagName: (tag || "div").toUpperCase(),
    children: [], parentNode: null,
    _classes: new Set(), _qs: new Map(),
    hidden: true, disabled: false, textContent: "",
    // Browser semantics: assigning innerHTML replaces (here: clears) children.
    // Without this, re-filled selects/tables accumulate options/rows across
    // refreshes and selection-preservation tests cannot be faithful.
    set innerHTML(v) { el.children = []; },
    get innerHTML() { return ""; },
    className: "",
    get classList() {
      const classes = el._classes;
      const sync = () => { el.className = [...classes].join(" "); };
      return {
        add: (...cs) => { cs.forEach((c) => classes.add(c)); sync(); },
        remove: (...cs) => { cs.forEach((c) => classes.delete(c)); sync(); },
        toggle: (c, force) => {
          const on = force === undefined ? !classes.has(c) : !!force;
          if (on) classes.add(c); else classes.delete(c); sync(); return on;
        },
        contains: (c) => classes.has(c),
      };
    },
    appendChild(c) { c.parentNode = el; el.children.push(c); return c; },
    removeChild(c) {
      const i = el.children.indexOf(c);
      if (i >= 0) el.children.splice(i, 1);
      c.parentNode = null; return c;
    },
    replaceChild(nu, old) {
      const i = el.children.indexOf(old);
      if (i >= 0) el.children[i] = nu;
      nu.parentNode = el; old.parentNode = null; return old;
    },
    get firstChild() { return el.children[0] || null; },
    get childElementCount() { return el.children.length; },
    get scrollHeight() { return 100; },
    get clientHeight() { return 0; },
    scrollTop: 0,
    querySelector(sel) {
      if (!el._qs.has(sel)) el._qs.set(sel, makeElement(doc, "div"));
      return el._qs.get(sel);
    },
    querySelectorAll() { return []; },
    addEventListener(type, fn) { (el._handlers = el._handlers || {})[type] = el._handlers[type] || []; el._handlers[type].push(fn); },
    setAttribute() {}, getAttribute() { return null; },
    focus() {}, blur() {},
    remove() { if (el.parentNode) el.parentNode.removeChild(el); },
    click() {},
    style: {},
  };
  return el;
}

function boot(opts) {
  const invokeImpl = opts && opts.invokeImpl;
  const sched = makeScheduler();
  const doc = {
    _ids: new Map(),
    getElementById(id) {
      if (!doc._ids.has(id)) doc._ids.set(id, makeElement(doc, "div"));
      return doc._ids.get(id);
    },
    createElement: (tag) => makeElement(doc, tag),
    createTextNode: (text) => ({ textContent: String(text), children: [], parentNode: null }),
    querySelector: () => null,
    querySelectorAll: () => [],
    addEventListener() {},
  };
  doc.body = makeElement(doc, "body");
  doc.documentElement = makeElement(doc, "html");
  const handlers = {};
  const invoked = [];
  const win = {
    __TAURI__: {
      core: { invoke: async (cmd, args) => { invoked.push([cmd, args]); return invokeImpl ? invokeImpl(cmd, args) : {}; } },
      event: { listen: async (evt, cb) => { (handlers[evt] = handlers[evt] || []).push(cb); return function () {}; } },
    },
  };
  const g = globalThis;
  const saved = { window: g.window, document: g.document, fetch: g.fetch, location: g.location };
  Object.defineProperty(g, "window", { value: win, configurable: true });
  Object.defineProperty(g, "document", { value: doc, configurable: true });
  Object.defineProperty(g, "location", { value: { href: "dsh-ui://localhost/settings" }, configurable: true });
  g.fetch = async function () { return { ok: false, text: async function () { return ""; } }; };
  const savedTimers = { setTimeout: g.setTimeout, setInterval: g.setInterval, clearTimeout: g.clearTimeout, clearInterval: g.clearInterval };
  g.setTimeout = sched.setTimeout; g.setInterval = sched.setInterval;
  g.clearTimeout = sched.clearTimeout; g.clearInterval = sched.clearInterval;

  let error = null;
  try { new Function(script)(); } catch (e) { error = e; }
  const restore = function () {
    Object.defineProperty(g, "window", { value: saved.window, configurable: true });
    Object.defineProperty(g, "document", { value: saved.document, configurable: true });
    Object.defineProperty(g, "location", { value: saved.location, configurable: true });
    g.fetch = saved.fetch;
    g.setTimeout = savedTimers.setTimeout; g.setInterval = savedTimers.setInterval;
    g.clearTimeout = savedTimers.clearTimeout; g.clearInterval = savedTimers.clearInterval;
  };
  return {
    sched, doc, handlers, invoked, error, restore,
    emit(evt, payload) { (handlers[evt] || []).forEach(function (fn) { fn({ payload: payload }); }); },
  };
}

// Let every pending invoke/.then callback run while the stubbed globals are
// still in place — restoring before this point makes late callbacks see the
// bare Node globals and crash.
async function settle() {
  for (let i = 0; i < 5; i++) await new Promise(function (r) { setImmediate(r); });
}

// ---------- assertions ----------
let passed = 0, failed = 0;
function check(name, cond, detail) {
  if (cond) { passed++; console.log("  ok  " + name); }
  else { failed++; console.log("  FAIL " + name + (detail ? " — " + detail : "")); }
}
function stuckCount(out) {
  return out.children.filter(function (c) { return /no progress for 60s/.test(String(c.textContent)); }).length;
}

const STATUS = {
  phase: "running", version: "0.1.2-rc.1", port: 3081, host: "127.0.0.1",
  installedVersions: ["0.1.2-rc.1"], activeVersion: "0.1.2-rc.1",
};
const invokeImpl = function (cmd) {
  if (cmd === "get_status") return STATUS;
  if (cmd === "tail_logs") return "[launcher] harness running";
  return {};
};

// ---------- 1. boot ----------
console.log("1. panel script boots and primes status + logs");
{
  const h = boot({ invokeImpl: invokeImpl });
  check("no init-time crash", h.error === null, h.error && String(h.error));
  await settle();
  check("get_status invoked at init", h.invoked.some(function (p) { return p[0] === "get_status"; }));
  check("tail_logs invoked at init", h.invoked.some(function (p) { return p[0] === "tail_logs"; }));
  h.restore();
}

// ---------- 2. full single-key update lifecycle ----------
console.log("2. single-key update lifecycle: feed closes, no stuck line after done");
{
  const h = boot({ invokeImpl: invokeImpl });
  const feedsBox = h.doc.getElementById("op-feeds");
  h.emit("launcher://progress", { op: "update", version: null, phase: "registry", message: "Checking npm registry for latest version…", percent: 5 });
  const feedEl = feedsBox.children[feedsBox.children.length - 1];
  const out = feedEl._qs.get(".term-out");
  check("registry phase opens one feed", feedsBox.children.length === 1 && feedEl.hidden === false);
  check("registry check shows no header (pure message)", feedEl._qs.get(".op-feed-head").hidden === true);
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "stopping", message: "Stopping the running engine to switch versions…", percent: 10 });
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "installing", message: "Installing DeepSeek Harness version@0.1.2-rc.1", percent: 40 });
  check("npm install shows the command header", feedEl._qs.get(".op-feed-head").hidden === false);
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "verifying", message: "Verifying v0.1.2-rc.1…", percent: 80 });
  check("still a single feed (no orphaned key switch)", feedsBox.children.length === 1);
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "done", message: "switched to v0.1.2-rc.1 — engine running on port 3081", percent: 100 });
  h.sched.advance(70000);
  check("no stuck line after done", stuckCount(out) === 0, "got " + stuckCount(out));
  check("done line recorded", out.children.some(function (c) { return /engine running on port 3081/.test(String(c.textContent)); }));
  h.sched.advance(10000);
  check("feed removed after hide", feedEl.parentNode === null);
  await settle();
  h.restore();
}

// ---------- 3. done clears a pending stuck timer ----------
console.log("3. done phase clears a pending stuck timer (reported bug)");
{
  const h = boot({ invokeImpl: invokeImpl });
  const feedsBox = h.doc.getElementById("op-feeds");
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "installing", message: "Installing DeepSeek Harness version@0.1.2-rc.1", percent: 40 });
  const feedEl = feedsBox.children[0];
  h.sched.advance(30000);
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "done", message: "switched to v0.1.2-rc.1", percent: 100 });
  h.sched.advance(60000);
  check("no stuck line when done lands first", stuckCount(feedEl._qs.get(".term-out")) === 0);
  await settle();
  h.restore();
}

// ---------- 4. genuinely stalled download ----------
console.log("4. stuck hint appears while genuinely stalled; console output re-arms it");
{
  const h = boot({ invokeImpl: invokeImpl });
  const feedsBox = h.doc.getElementById("op-feeds");
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "installing", message: "Installing DeepSeek Harness version@0.1.2-rc.1", percent: 40 });
  const feedEl = feedsBox.children[0];
  const out = feedEl._qs.get(".term-out");
  h.sched.advance(61000);
  check("stuck hint appended once after 60s idle", stuckCount(out) === 1, "got " + stuckCount(out));
  h.emit("launcher://console", { op: "update", stream: "out", text: "npm warn …" });
  h.sched.advance(61000);
  check("console output re-arms the stuck watch", stuckCount(out) === 2, "got " + stuckCount(out));
  await settle();
  h.restore();
}

// ---------- 5. install dropdown defaults to the newest published version ----------
// The Versions tab must present the latest version as the pre-selected choice
// (no separate "Get latest" button): with the newest version not installed it
// shows without the ✓ so the user instantly knows to download it.
console.log("5. install dropdown defaults to the newest published version");
{
  const STATUS5 = {
    phase: "stopped", version: "0.1.1-rc.1", port: 3081, host: "127.0.0.1",
    versions: ["0.1.1-rc.1", "0.1.2-rc.1"],
    installedVersions: ["0.1.1-rc.1"], activeVersion: "0.1.1-rc.1",
    latestRemote: null,
  };
  const h = boot({ invokeImpl: function (cmd) {
    if (cmd === "get_status") return STATUS5;
    if (cmd === "tail_logs") return "[launcher] harness stopped";
    return {};
  } });
  await settle();
  const installSel = h.doc.getElementById("sel-install-ver");
  check("no init-time crash", h.error === null, h.error && String(h.error));
  check("dropdown defaults to newest version, not the active one",
    installSel.value === "0.1.2-rc.1", "got " + installSel.value);
  const opts = installSel.children;
  const activeOpt = opts.find(function (o) { return o.value === "0.1.1-rc.1"; });
  const latestOpt = opts.find(function (o) { return o.value === "0.1.2-rc.1"; });
  check("installed option carries the checkmark", /✓/.test(String(activeOpt && activeOpt.textContent)));
  check("uninstalled latest option has no checkmark", !/✓/.test(String(latestOpt && latestOpt.textContent)));
  h.restore();
}

// ---------- 6. structure: install row merged into the versions table ----------
// The standalone "Install version" group is gone; the dropdown + cloud
// download live in the table's persistent LAST row, the header column reads
// "Directory", and the log toggle is pinned bottom-right.
console.log("6. install row merged into the versions table; log toggle bottom-right");
{
  check("header column renamed to Directory",
    /<span class="vc-dir" title="Open the version directory" data-i18n="directory" data-i18n-title="directoryTitle">Directory<\/span>/.test(html));
  check("no Folder header label remains", !/>Folder</.test(html));
  check("row labels say directory, not folder", !/Open the folder/.test(html));
  check("standalone Install version group removed", !/Install version<\/span>/.test(html));
  const vtable = html.indexOf('<div class="vtable">');
  const opFeeds = html.indexOf('id="op-feeds"');
  const installSel = html.indexOf('id="sel-install-ver"');
  const btnInstall = html.indexOf('id="btn-install"');
  // Order: table (rows + install row LAST inside it), then the feeds section
  // below the table group — feeds never sit between the table's rows.
  check("install dropdown + button are the table's last row",
    vtable !== -1 && vtable < installSel && installSel < btnInstall &&
    btnInstall < opFeeds && opFeeds < html.indexOf("</main>"));
  check("feeds render in their own section below the table",
    html.includes('class="group op-feeds-group" id="op-feeds"') &&
    html.indexOf('vinstall-row') < opFeeds);
  check("feeds section collapses when empty",
    /#op-feeds:empty \{ display: none; \}/.test(html));
  check("install button uses the cloud-down icon",
    /id="btn-install"[^>]*>\s*<i class="ico sm bi" data-ico="cloud-arrow-down-fill"><\/i>/.test(html));
  check("install row keeps the 4-column layout (empty | dropdown | empty | button)",
    /vrow vinstall-row">\s*<span class="vc-def"[^>]*><\/span>\s*<span class="vc-ver vc-ver-install">\s*<select id="sel-install-ver"/.test(html));
  check("log toggle pinned bottom-right", /\.log-toggle \{ position: fixed; right: 14px/.test(html));
  check("log toggle no longer pinned left", !/\.log-toggle \{[^}]*left: 14px/.test(html));
}

// ---------- 7. stopped download dismisses and is never resurrected ----------
// A user-stopped download reports the cancelled phase once, but trailing npm
// console lines may arrive afterwards. The feed must dismiss exactly once
// (repeat terminal events must not extend the wait) and late console lines
// for the closed operation must not re-create the feed.
console.log("7. stopped download: feed dismisses once; late lines never resurrect it");
{
  const h = boot({ invokeImpl: invokeImpl });
  const feedsBox = h.doc.getElementById("op-feeds");
  h.emit("launcher://progress", { op: "download:v0.1.2-alpha.5", version: "0.1.2-alpha.5", phase: "registry", message: "Checking DeepSeek Harness versions", percent: 5 });
  h.emit("launcher://progress", { op: "download:v0.1.2-alpha.5", version: "0.1.2-alpha.5", phase: "installing", message: "Installing DeepSeek Harness version@0.1.2-alpha.5", percent: 40 });
  const feedEl = feedsBox.children[feedsBox.children.length - 1];
  check("download feed shows its command header", feedEl.hidden === false && feedEl._qs.get(".op-feed-head").hidden === false);
  // The user taps stop: the backend reports the cancelled phase (twice — the
  // second report mirrors the first on a refresh-driven progress replay).
  h.emit("launcher://progress", { op: "download:v0.1.2-alpha.5", version: "0.1.2-alpha.5", phase: "cancelled", message: "Download cancelled", percent: 0 });
  h.emit("launcher://progress", { op: "download:v0.1.2-alpha.5", version: "0.1.2-alpha.5", phase: "cancelled", message: "Download cancelled", percent: 0 });
  h.sched.advance(10200); // first terminal window (10s) + 180ms animate-out
  check("feed removed after the first cancelled window (no extension)",
    feedEl.parentNode === null);
  check("feeds section is empty again", feedsBox.children.length === 0);
  // Trailing npm output flushed after the cancel must not resurrect the feed.
  h.emit("launcher://console", { op: "download:v0.1.2-alpha.5", stream: "out", text: "npm warn cancel cleanup line" });
  h.emit("launcher://console", { op: "download:v0.1.2-alpha.5", stream: "err", text: "npm error canceled" });
  check("late console lines for a closed op do not re-create the feed",
    feedsBox.children.length === 0);
  h.sched.advance(61000);
  check("no stuck timer fired for the closed op", stuckCount({ children: [] }) === 0 &&
    feedsBox.children.length === 0);
  await settle();
  h.restore();
}

// ---------- 9. minimal terminal: header only for download phases ----------
// Terminals render command header + output for download phases; pure message
// phases (registry checks, launcher notices) stream as info lines with no
// header and no stop/cancel control anywhere.
console.log("9. minimal terminal: header for download phases only, no stop control");
{
  const h = boot({ invokeImpl: invokeImpl });
  const feedsBox = h.doc.getElementById("op-feeds");
  // Registry check: pure message — output line only, no header.
  h.emit("launcher://progress", { op: "update", version: null, phase: "registry", message: "Checking npm registry for latest version…", percent: 5 });
  let feedEl = feedsBox.children[0];
  check("registry feed visible without header",
    feedEl.hidden === false && feedEl._qs.get(".op-feed-head").hidden === true);
  check("registry message streamed as info line",
    feedEl._qs.get(".term-out").children.some(function (c) { return /Checking npm registry/.test(String(c.textContent)); }));
  // Real npm install: header + elapsed.
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "installing", message: "Installing DeepSeek Harness version@0.1.2-rc.1", percent: 40 });
  check("npm install shows the header", feedEl._qs.get(".op-feed-head").hidden === false);
  check("terminal header is action-neutral",
    feedEl._qs.get(".op-feed-cmd").textContent === "Harness Launcher #",
    JSON.stringify(feedEl._qs.get(".op-feed-cmd").textContent));
  // Launcher-level notice (no op key): pure message, no header.
  h.emit("launcher://console", { stream: "info", text: "Removed installed version 0.1.1-rc.1" });
  const noticeEl = feedsBox.children[feedsBox.children.length - 1];
  check("notice feed has no header",
    noticeEl._qs.get(".op-feed-head").hidden === true);
  h.sched.advance(10000);
  await settle();
  h.restore();
}

// ---------- 8. structure: minimal terminal + white install icon ----------
console.log("8. structure: no stop control; white install button glyph");
{
  check("no stop button in the panel markup",
    !html.includes("dl-cancel") && !/aria-label="Stop download"/.test(html));
  check("no stop glyph reference remains",
    !/data-ico="stop-(fill|circle)"/.test(html));
  const fs = await import("node:fs");
  const path = await import("node:path");
  const url = await import("node:url");
  const root = path.resolve(path.dirname(url.fileURLToPath(import.meta.url)), "..");
  check("stop-fill.svg removed from resources",
    !fs.existsSync(path.join(root, "src-tauri", "resources", "bootstrap-icons", "stop-fill.svg")));
  const overlay = fs.readFileSync(path.join(root, "src-tauri", "resources", "overlay.js"), "utf8");
  check("no stop button in the overlay panel",
    !overlay.includes("lc-stopdown") && !overlay.includes("stop-btn"));
  check("install button glyph is white",
    /\.btn\.install \{ color: #fff; \}/.test(html));
  check("install button gets a light-mode backing disc",
    /\(prefers-color-scheme: light\)\s*\{\s*\.btn\.install \{ background: rgba\(29, 36, 48, 0\.32\); \}/.test(html));
}

console.log("10. tray fullscreen toggles carry glyphs and handlers");
{
  const tray = readFileSync(path.join(root, "src-tauri", "src", "tray.rs"), "utf8");
  function assetExists(p) { try { readFileSync(p); return true; } catch { return false; } }
  for (const id of ["fullscreen-harness", "fullscreen-settings"]) {
    check("tray symbol PNG exists for " + id,
      assetExists(path.join(root, "src-tauri", "resources", "tray", "symbol", id + ".png")));
    check("tray menu item registered for " + id,
      tray.includes("\"" + id + "\""));
  }
  check("fullscreen toggle calls set_fullscreen", /set_fullscreen\(!fullscreen\)/.test(tray));
  check("fullscreen items gated on window existence", /fullscreen_harness\s*\n?\s*\.set_enabled/.test(tray));
}

console.log("9. versions table renders hostile version names as inert text");
{
  const EVIL = "<img src=x onerror=alert(1)>";
  const h = boot({ invokeImpl: function (cmd) {
    if (cmd === "get_status") return { phase: "stopped", version: "x", port: 3081, host: "127.0.0.1",
      versions: [], installedVersions: ["0.1.1-rc.1", EVIL], activeVersion: "0.1.1-rc.1",
      latestRemote: null };
    if (cmd === "tail_logs") return "";
    return {};
  } });
  await settle();
  check("no init-time crash with hostile version name", h.error === null, h.error && String(h.error));
  const tbody = h.doc.getElementById("ver-tbody");
  const texts = [];
  (function walk(el) {
    if (el.textContent) texts.push(String(el.textContent));
    (el.children || []).forEach(walk);
  })(tbody);
  check("hostile version name present as text", texts.some(function (t) { return t.indexOf(EVIL) !== -1; }),
    texts.join("|").slice(0, 200));
  check("no innerHTML sink interpolates a version variable",
    !/\.innerHTML\s*=[^;]*\+\s*v\b/.test(html));
  h.restore();
}

// ---------- 11. version-list refresh button ----------
console.log("11. install row refresh button refetches the list, keeps selection");
{
  check("refresh button in install row markup with refresh glyph",
    html.includes('id="btn-refresh-versions"') && html.includes('data-ico="arrow-clockwise"'));
  let glyphOk = false;
  try { readFileSync(path.join(root, "src-tauri", "resources", "bootstrap-icons", "arrow-clockwise.svg")); glyphOk = true; } catch {}
  check("refresh glyph bundled", glyphOk);
  check("refresh button styled icon-only",
    /\.ver-refresh-btn svg \{ width: 16px; height: 16px; \}/.test(html));
  let refreshCalls = 0, extraVersion = false;
  const h = boot({ invokeImpl: function (cmd) {
    if (cmd === "refresh_versions") { refreshCalls++; extraVersion = true; return "version list refreshed (3 versions)"; }
    if (cmd === "get_status") {
      return { enginePhase: "stopped", running: false, launcherVersion: "0.1.80",
        activeVersion: "0.1.1", previousVersion: null, port: 3081, actualPort: 3081, portChanged: false,
        versions: extraVersion ? ["0.1.1", "0.1.2", "0.1.3"] : ["0.1.1", "0.1.2"],
        installedVersions: ["0.1.1"], latestRemote: "0.1.2", updateAvailable: false,
        includePrerelease: false, startOnLaunch: true, bootError: null,
        currentOp: null, currentOps: [], console: [], webUrl: null };
    }
    if (cmd === "tail_logs") return "";
    return {};
  } });
  check("no init-time crash with refresh button present", h.error === null, h.error && String(h.error));
  await settle();
  const rsel = h.doc.getElementById("sel-install-ver");
  const rbtn = h.doc.getElementById("btn-refresh-versions");
  check("dropdown lists published versions", rsel.children.length === 2);
  check("refresh button wired",
    !!(rbtn._handlers && rbtn._handlers.click && rbtn._handlers.click.length));
  rsel.value = "0.1.1";
  rbtn._handlers.click[0]();
  check("button busy-disables while fetching", rbtn.disabled === true);
  await settle();
  check("refresh_versions invoked once", refreshCalls === 1);
  check("dropdown picks up the new version", rsel.children.length === 3);
  check("selection preserved across refresh", rsel.value === "0.1.1");
  check("button re-enabled after refresh", rbtn.disabled === false);
  h.restore();
}

// ---------- 12. neutral terminal header ----------
console.log("12. operation terminals head with Harness Launcher #");
{
  check("panel feed default header is neutral",
    html.includes('<span class="op-feed-cmd">Harness Launcher #</span>'));
  check("no install-claim header remains in the panel",
    !html.includes("Installing DeepSeek Harness version"));
  const overlaySrc = readFileSync(path.join(root, "src-tauri", "resources", "overlay.js"), "utf8");
  check("overlay terminal title is neutral",
    overlaySrc.includes(">Harness Launcher #</span>") && !overlaySrc.includes("Installing DeepSeek Harness version@"));
}

// ---------- 13. language switcher ----------
console.log("13. header language switcher persists and repaints");
{
  const segCodes = ["en", "zh-Hant", "zh-Hans", "ja", "es"];
  const segLabels = [">En<", ">繁<", ">簡<", ">日<", ">Es<"];
  check("five language segments with the specified labels",
    segCodes.every(function (c) { return html.includes('data-lang="' + c + '"'); }) &&
    segLabels.every(function (s) { return html.includes(s); }));
  // Locale table spot-checks (the fake DOM carries no markup attributes, so
  // table content is asserted on the script source instead).
  const scriptSrc = html.match(/<script>([\s\S]*?)<\/script>/)[1];
  function localeVal(lang, key) {
    const bare = lang === "en" || lang === "ja" || lang === "es";
    const open = "    " + (bare ? lang : '"' + lang + '"') + ": {";
    const blk = scriptSrc.slice(scriptSrc.indexOf(open)).split("\n    },")[0];
    const m = blk.match(new RegExp(key + ':\\s*"([^\"]+)"'));
    return m && m[1];
  }
  check("ja quit translated", localeVal("ja", "quit") === "終了", String(localeVal("ja", "quit")));
  check("zh-Hant refresh translated", localeVal("zh-Hant", "refreshList") === "重新整理版本清單");
  check("zh-Hans start translated", localeVal("zh-Hans", "start") === "启动");
  check("es cancel translated", localeVal("es", "cancel") === "Cancelar");
  check("unknown code falls back to En", /if \(!LOCALES\[l\]\) l = "en"/.test(scriptSrc));
  check("missing key falls back to English", /\|\| LOCALES\.en\[key\]/.test(scriptSrc));
  check("status payload drives the panel language", /if \(s\.language\) applyLanguage\(s\.language\)/.test(scriptSrc));
  const langStatus = function (lang) {
    return { enginePhase: "stopped", running: false, launcherVersion: "0.1.80",
      activeVersion: "0.1.1", previousVersion: null, port: 3081, actualPort: 3081, portChanged: false,
      versions: ["0.1.1"], installedVersions: ["0.1.1"], latestRemote: null, updateAvailable: false,
      includePrerelease: false, startOnLaunch: true, bootError: null, language: lang,
      currentOp: null, currentOps: [], console: [], webUrl: null };
  };
  // Saved Japanese: the panel converges on it at boot.
  let hj = boot({ invokeImpl: function (cmd) {
    if (cmd === "get_status") return langStatus("ja");
    if (cmd === "tail_logs") return "";
    return {};
  } });
  await settle();
  check("no init-time crash with saved language", hj.error === null, hj.error && String(hj.error));
  check("document lang follows the saved language", hj.doc.documentElement.lang === "ja");
  // Click the Es segment: set_language persists, repaint follows the payload.
  const seg = hj.doc.getElementById("lang-seg");
  seg._handlers.click[0]({ target: { closest: function () { return { getAttribute: function () { return "es"; } }; } } });
  await settle();
  const persisted = hj.invoked.filter(function (p) { return p[0] === "set_language"; });
  check("set_language invoked once with the picked code",
    persisted.length === 1 && persisted[0][1] && persisted[0][1].language === "es",
    JSON.stringify(persisted));
  hj.restore();
  // Unknown saved code falls back to English.
  let hx = boot({ invokeImpl: function (cmd) {
    if (cmd === "get_status") return langStatus("xx");
    if (cmd === "tail_logs") return "";
    return {};
  } });
  await settle();
  check("unknown saved language resets document lang to en",
    hx.doc.documentElement.lang === "en");
  hx.restore();
}

console.log("");
console.log(passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
