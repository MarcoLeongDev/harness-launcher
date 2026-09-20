// Regression: control-panel log drawer must append and never go empty.
// Bug: <pre id="log-tail"> carried data-i18n="noOutput", so applyLanguage()
// (run on every 3s status poll) wiped rendered log spans back to
// "No output yet"; refreshLog only repopulates every 5s.
// Contract:
//   1. settings.html #log-tail carries NO data-i18n attribute.
//   2. settings.js applyLanguage skips the log container + relocalises only
//      the empty placeholder.
//   3. booted panel renders tail_logs lines and keeps them across status
//      polls (append-only, never empty while content exists).
//   4. empty log renders the localised noLogs hint, not "No output yet".
// Run: node scripts/test-log-append.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const html = readFileSync(path.join(root, "src-tauri", "resources", "settings.html"), "utf8");
const script = readFileSync(path.join(root, "src-tauri", "resources", "settings.js"), "utf8");

let passed = 0,
  failed = 0;
function check(name, cond, detail) {
  if (cond) {
    passed++;
    console.log(`  ok  ${name}`);
  } else {
    failed++;
    console.log(`  FAIL ${name}${detail ? ` — ${detail}` : ""}`);
  }
}

// ---------- 1. static markup ----------
console.log("1. log-tail carries no i18n repaint hook");
const logTailTag = html.match(/<pre[^>]*id="log-tail"[^>]*>/);
check("log-tail element exists", !!logTailTag, "no #log-tail in settings.html");
check(
  "log-tail has no data-i18n attribute",
  !!logTailTag && !/data-i18n/.test(logTailTag[0]),
  logTailTag ? logTailTag[0] : "",
);
check('no data-i18n="noOutput" remains in panel markup', !/data-i18n="noOutput"/.test(html));

// ---------- 2. script guards ----------
console.log("2. applyLanguage never wipes appended logs");
check("applyLanguage skips #log-tail", /if\s*\(\s*el\.id\s*===\s*"log-tail"\s*\)\s*continue/.test(script));
check("empty placeholder relocalised separately", /relocaliseLogEmpty\(\)/.test(script));
check(
  "language switch re-fetches logs",
  /listen\("launcher:\/\/language"[\s\S]*?refreshLog\(\)/.test(script),
);

// ---------- 3+4. dynamic boot harness (mini DOM, virtual clock) ----------
function makeScheduler() {
  let now = 0,
    nextId = 1;
  const timers = new Map();
  return {
    setTimeout: (fn, ms) => {
      const id = nextId++;
      timers.set(id, { id, fn, at: now + (ms || 0), period: null });
      return id;
    },
    setInterval: (fn, ms) => {
      const id = nextId++;
      timers.set(id, { id, fn, at: now + (ms || 0), period: ms || 0 });
      return id;
    },
    clearTimeout: (id) => timers.delete(id),
    clearInterval: (id) => timers.delete(id),
    advance: (ms) => {
      const target = now + ms;
      for (;;) {
        let due = null;
        for (const t of timers.values()) if (t.at <= target && (!due || t.at < due.at)) due = t;
        if (!due) break;
        now = Math.max(now, due.at);
        if (due.period !== null) {
          due.at = now + due.period;
          due.fn();
        } else {
          timers.delete(due.id);
          due.fn();
        }
      }
      now = target;
    },
  };
}

function makeElement(doc, tag) {
  const el = {
    tagName: (tag || "div").toUpperCase(),
    children: [],
    parentNode: null,
    hidden: true,
    disabled: false,
    textContent: "",
    id: "",
    set innerHTML(_v) {
      el.children = [];
    },
    get innerHTML() {
      return "";
    },
    className: "",
    _classes: new Set(),
    _qs: new Map(),
    get classList() {
      const classes = el._classes;
      return {
        add: (...cs) => {
          for (const c of cs) classes.add(c);
        },
        remove: (...cs) => {
          for (const c of cs) classes.delete(c);
        },
        toggle: (c, force) => {
          const on = force === undefined ? !classes.has(c) : !!force;
          if (on) classes.add(c);
          else classes.delete(c);
          return on;
        },
        contains: (c) => classes.has(c),
      };
    },
    appendChild(c) {
      c.parentNode = el;
      el.children.push(c);
      return c;
    },
    removeChild(c) {
      const i = el.children.indexOf(c);
      if (i >= 0) el.children.splice(i, 1);
      c.parentNode = null;
      return c;
    },
    get firstChild() {
      return el.children[0] || null;
    },
    get childElementCount() {
      return el.children.length;
    },
    get scrollHeight() {
      return 100;
    },
    get clientHeight() {
      return 0;
    },
    scrollTop: 0,
    querySelector(sel) {
      if (!el._qs.has(sel)) el._qs.set(sel, makeElement(doc, "div"));
      return el._qs.get(sel);
    },
    querySelectorAll() {
      return [];
    },
    addEventListener(t, fn) {
      if (!el._handlers) el._handlers = {};
      if (!el._handlers[t]) el._handlers[t] = [];
      el._handlers[t].push(fn);
    },
    setAttribute() {},
    getAttribute() {
      return null;
    },
    style: {},
  };
  return el;
}

function boot(invokeImpl) {
  const sched = makeScheduler();
  const doc = {
    _ids: new Map(),
    getElementById(id) {
      if (!doc._ids.has(id)) {
        const el = makeElement(doc, "div");
        el.id = id;
        doc._ids.set(id, el);
      }
      return doc._ids.get(id);
    },
    createElement: (tag) => makeElement(doc, tag),
    createTextNode: (text) => ({ textContent: String(text), children: [], parentNode: null }),
    querySelector: () => null,
    // Simulate the fixed markup: no element advertises data-i18n="noOutput",
    // so the i18n repaint cannot select the log container.
    querySelectorAll: (sel) => (sel === "[data-i18n]" ? [] : []),
    addEventListener() {},
  };
  doc.body = makeElement(doc, "body");
  doc.documentElement = makeElement(doc, "html");
  const handlers = {};
  const win = {
    addEventListener() {},
    __TAURI__: {
      core: { invoke: async (cmd, args) => invokeImpl(cmd, args) },
      event: {
        listen: async (evt, cb) => {
          if (!handlers[evt]) handlers[evt] = [];
          handlers[evt].push(cb);
          return () => {};
        },
      },
    },
  };
  const g = globalThis;
  const saved = { window: g.window, document: g.document };
  Object.defineProperty(g, "window", { value: win, configurable: true });
  Object.defineProperty(g, "document", { value: doc, configurable: true });
  g.fetch = async () => ({ ok: false, text: async () => "" });
  const st = {
    setTimeout: g.setTimeout,
    setInterval: g.setInterval,
    clearTimeout: g.clearTimeout,
    clearInterval: g.clearInterval,
  };
  g.setTimeout = sched.setTimeout;
  g.setInterval = sched.setInterval;
  g.clearTimeout = sched.clearTimeout;
  g.clearInterval = sched.clearInterval;
  let error = null;
  try {
    new Function(script)();
  } catch (e) {
    error = e;
  }
  return {
    sched,
    doc,
    handlers,
    error,
    restore() {
      Object.defineProperty(g, "window", { value: saved.window, configurable: true });
      Object.defineProperty(g, "document", { value: saved.document, configurable: true });
      g.setTimeout = st.setTimeout;
      g.setInterval = st.setInterval;
      g.clearTimeout = st.clearTimeout;
      g.clearInterval = st.clearInterval;
    },
  };
}

async function settle() {
  for (let i = 0; i < 5; i++) await new Promise((r) => setImmediate(r));
}

const baseStatus = {
  enginePhase: "running",
  running: true,
  launcherVersion: "0.1.114",
  activeVersion: "0.1.1",
  port: 3081,
  actualPort: 3081,
  versions: ["0.1.1"],
  installedVersions: ["0.1.1"],
  latestRemote: null,
  updateAvailable: false,
  currentOp: null,
  currentOps: [],
  console: [],
};

console.log("3. appended logs survive status polls");
{
  const h = boot((cmd) => {
    if (cmd === "get_status") return { ...baseStatus, language: "en" };
    if (cmd === "tail_logs") return "[launcher] starting harness\nengine running on 3081";
    return {};
  });
  check("no init crash", h.error === null, h.error && String(h.error));
  await settle();
  // Let several poll cycles run (refresh every 3s, refreshLog every 5s).
  h.sched.advance(15000);
  await settle();
  const tail = h.doc.getElementById("log-tail");
  const texts = tail.children.map((c) => String(c.textContent));
  check("log lines rendered", tail.children.length >= 2, `got ${tail.children.length}`);
  check(
    "log content appended, not placeholder",
    texts.some((t) => /starting harness/.test(t)),
    texts.join("|").slice(0, 160),
  );
  check(
    "never reverted to legacy placeholder",
    !texts.some((t) => /No output yet/.test(t)),
    texts.join("|").slice(0, 160),
  );
  h.restore();
}

console.log("4. empty log shows localised hint");
{
  const h = boot((cmd) => {
    if (cmd === "get_status") return { ...baseStatus, language: "en" };
    if (cmd === "tail_logs") return "";
    return {};
  });
  await settle();
  h.sched.advance(6000);
  await settle();
  const tail = h.doc.getElementById("log-tail");
  const texts = tail.children.map((c) => String(c.textContent));
  check("empty state renders one hint line", tail.children.length === 1, `got ${tail.children.length}`);
  check(
    "empty hint is the noLogs string",
    texts.some((t) => /\(no logs yet\)/.test(t)),
    texts.join("|").slice(0, 160),
  );
  h.restore();
}

console.log("");
console.log(`${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
