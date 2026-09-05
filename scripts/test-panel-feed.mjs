// Regression tests for the Control Panel operation-feed lifecycle.
//
// Loads the inline script from src-tauri/resources/settings.html and runs it
// against a minimal DOM + Tauri stub with a virtual clock, then replays the
// launcher://progress / launcher://console event sequences the backend emits.
//
// Covered contract (lightspec: Stable Operation Key For An Operation's Lifetime):
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
  check("stop button visible while downloading", feedEl._qs.get(".dl-cancel").hidden === false);
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "stopping", message: "Stopping the running engine to switch versions…", percent: 10 });
  h.emit("launcher://progress", { op: "update", version: "0.1.2-rc.1", phase: "installing", message: "Installing DeepSeek Harness version@0.1.2-rc.1", percent: 40 });
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
  h.emit("launcher://progress", { op: "update", version: null, phase: "registry", message: "Checking npm registry for latest version…", percent: 5 });
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
  h.emit("launcher://progress", { op: "update", version: null, phase: "registry", message: "Checking npm registry for latest version…", percent: 5 });
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
    /<span class="vc-dir" title="Open the version directory">Directory<\/span>/.test(html));
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

console.log("");
console.log(passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
