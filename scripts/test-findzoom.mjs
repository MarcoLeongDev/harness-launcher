// Behavioral tests for src-tauri/resources/findzoom.js (webview find + zoom).
//
// Runs the injected script in node:vm against a minimal DOM stub (no browser
// needed) and drives it with synthetic keyboard/input events.
//
// Covered contract:
//   1. boots once: re-evaluation never double-binds shortcuts or duplicates UI
//   2. zoom: Cmd/Ctrl =/- /0 steps ±10% within 50–200%, resets to 100%,
//      applies documentElement zoom, persists per origin, restores on load
//   3. find: Cmd/Ctrl+F opens + focuses the bar, typing counts matches,
//      Enter/Shift+Enter navigates, Esc closes and clears the selection
//
// Run: node scripts/test-findzoom.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const JS = readFileSync(path.join(root, "src-tauri", "resources", "findzoom.js"), "utf8");

let failures = 0;
function check(name, cond, extra = "") {
  if (cond) console.log(`  PASS ${name}`);
  else {
    failures++;
    console.error(`  FAIL ${name} ${extra}`);
  }
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---- Minimal DOM stub -------------------------------------------------------
const SHOW_TEXT = 4;
const FILTER_ACCEPT = 1;
const FILTER_REJECT = 2;
const FILTER_SKIP = 3;

function makeClassList() {
  const set = new Set();
  return {
    add: (c) => set.add(c),
    remove: (c) => set.delete(c),
    contains: (c) => set.has(c),
    toggle: (c, force) => {
      const on = force === undefined ? !set.has(c) : !!force;
      if (on) set.add(c);
      else set.delete(c);
      return on;
    },
  };
}

function makeContext({ storage = {}, bodyText = "", withHighlight = false } = {}) {
  const byId = new Map();
  const store = new Map(Object.entries(storage));
  const listeners = { window: {}, doc: {} };
  const findCalls = [];
  let selectionCleared = 0;

  function register(el) {
    if (el.id) byId.set(el.id, el);
    for (const child of el.children) register(child);
  }

  // Parses the flat (non-nested) innerHTML the script uses for the find bar.
  function parseFlatHtml(parent, html) {
    for (const m of html.matchAll(/<(\w+)([^>]*?)>/g)) {
      const el = makeEl(m[1]);
      const attrs = m[2];
      const id = attrs.match(/id="([^"]+)"/);
      if (id) el.setAttribute("id", id[1]);
      const ph = attrs.match(/placeholder="([^"]+)"/);
      if (ph) el.setAttribute("placeholder", ph[1]);
      parent.appendChild(el);
    }
  }

  function makeEl(tag) {
    const el = {
      nodeName: String(tag).toUpperCase(),
      nodeType: 1,
      children: [],
      parentNode: null,
      style: {},
      classList: makeClassList(),
      textContent: "",
      value: "",
      _innerHTML: "",
      _listeners: {},
      id: "",
      setAttribute(k, v) {
        if (k === "id") {
          this.id = v;
          byId.set(v, this);
        } else this[k] = v;
      },
      getAttribute(k) {
        return this[k];
      },
      appendChild(child) {
        child.parentNode = this;
        this.children.push(child);
        register(child);
        return child;
      },
      addEventListener(type, fn) {
        (this._listeners[type] ||= []).push(fn);
      },
      dispatch(type, event = {}) {
        event.target ||= this;
        event.preventDefault ||= () => (event.defaultPrevented = true);
        event.stopPropagation ||= () => {};
        for (const fn of this._listeners[type] || []) fn(event);
        return event;
      },
      querySelector(sel) {
        if (sel.startsWith("#")) return byId.get(sel.slice(1)) || null;
        return null;
      },
      focus() {
        document.activeElement = this;
      },
      select() {},
      click() {
        this.dispatch("click");
      },
    };
    Object.defineProperty(el, "innerHTML", {
      get: () => el._innerHTML,
      set: (html) => {
        el._innerHTML = html;
        parseFlatHtml(el, html);
      },
    });
    Object.defineProperty(el, "childElementCount", { get: () => el.children.length });
    return el;
  }

  function makeText(value, parent) {
    return { nodeType: 3, nodeValue: value, parentNode: parent };
  }

  const head = makeEl("head");
  const html = makeEl("html");
  const body = makeEl("body");
  html.appendChild(head);
  html.appendChild(body);
  // Page content for find tests: plain text nodes (no framework involved).
  const pageText = makeText(bodyText, body);
  body.children.push(pageText);

  function collectTextNodes(node, out) {
    for (const child of node.children || []) {
      if (child.nodeType === 3) out.push(child);
      else collectTextNodes(child, out);
    }
    return out;
  }

  const document = {
    readyState: "complete",
    head,
    body,
    documentElement: html,
    activeElement: null,
    createElement: (tag) => makeEl(tag),
    getElementById: (id) => byId.get(id) || null,
    addEventListener(type, fn) {
      (listeners.doc[type] ||= []).push(fn);
    },
    createTreeWalker(rootNode, whatToShow, filter) {
      const all = collectTextNodes(rootNode, []);
      const accepted = all.filter((node) => {
        if (whatToShow !== SHOW_TEXT) return false;
        try {
          return filter.acceptNode(node) === FILTER_ACCEPT;
        } catch {
          return false;
        }
      });
      let i = -1;
      const walker = {
        currentNode: rootNode,
        nextNode() {
          i++;
          if (i < accepted.length) {
            walker.currentNode = accepted[i];
            return walker.currentNode;
          }
          return null;
        },
      };
      return walker;
    },
  };

  const window = {
    localStorage: {
      getItem: (k) => (store.has(k) ? store.get(k) : null),
      setItem: (k, v) => store.set(String(k), String(v)),
    },
    addEventListener(type, fn) {
      (listeners.window[type] ||= []).push(fn);
    },
    find(query) {
      findCalls.push(query);
      return String(query).length > 0;
    },
    getSelection: () => ({
      removeAllRanges: () => selectionCleared++,
      toString: () => "",
    }),
  };

  const sandbox = {
    window,
    document,
    NodeFilter: { SHOW_TEXT, FILTER_ACCEPT, FILTER_REJECT, FILTER_SKIP },
    CSS: withHighlight ? { highlights: new Map() } : {},
    console,
    setTimeout,
    clearTimeout,
  };
  sandbox.window.window = sandbox.window;
  vm.createContext(sandbox);
  vm.runInContext(JS, sandbox, { filename: "findzoom.js" });

  function keydown(event) {
    const e = {
      key: "",
      code: "",
      metaKey: false,
      ctrlKey: false,
      altKey: false,
      shiftKey: false,
      target: document.activeElement || body,
      defaultPrevented: false,
      preventDefault() {
        this.defaultPrevented = true;
      },
      stopPropagation() {},
      ...event,
    };
    for (const fn of listeners.window.keydown || []) fn(e);
    return e;
  }

  return { document, window, body, byId, store, findCalls, keydown, sandbox,
    selectionCleared: () => selectionCleared };
}

// ---- 1. Boots once -----------------------------------------------------------
console.log("findzoom: boots once");
{
  const ctx = makeContext();
  const bars = () =>
    ctx.body.children.filter((c) => c.id === "dsh-fz-bar").length;
  check("install guard set", ctx.sandbox.window.__DSH_FINDZOOM__ === true);
  check("exactly one find bar built", bars() === 1);
  check("zoom badge built", !!ctx.document.getElementById("dsh-fz-zoom"));
  check("find style built", !!ctx.document.getElementById("dsh-fz-style"));
  vm.runInContext(JS, ctx.sandbox, { filename: "findzoom.js (re-eval)" });
  check("re-evaluation adds no second bar", bars() === 1);
}

// ---- 2. Zoom -----------------------------------------------------------------
console.log("findzoom: zoom");
{
  const ctx = makeContext();
  const zoom = () => ctx.document.documentElement.style.zoom;
  check("starts at 100%", zoom() === "100%");
  const e1 = ctx.keydown({ key: "=", metaKey: true });
  check("Cmd+= zooms in (prevented)", e1.defaultPrevented && zoom() === "110%");
  check("badge shows 110%", ctx.document.getElementById("dsh-fz-zoom").textContent === "110%");
  check("zoom persisted", ctx.store.get("dsh-zoom") === "110");
  ctx.keydown({ key: "-", ctrlKey: true });
  check("Ctrl+- zooms back out", zoom() === "100%");
  ctx.keydown({ key: "+", metaKey: true });
  ctx.keydown({ key: "0", metaKey: true });
  check("Cmd+0 resets to 100%", zoom() === "100%");
  // No modifier: page keeps the key.
  const plain = ctx.keydown({ key: "=" });
  check("bare = is not hijacked", !plain.defaultPrevented && zoom() === "100%");
  // Numpad codes work too.
  ctx.keydown({ key: "Add", code: "NumpadAdd", metaKey: true });
  check("numpad add zooms in", zoom() === "110%");
}
{
  // Persisted level is restored + clamped on load.
  const lo = makeContext({ storage: { "dsh-zoom": "130" } });
  check("restores 130% on load", lo.document.documentElement.style.zoom === "130%");
  const hi = makeContext({ storage: { "dsh-zoom": "500" } });
  check("clamps 500% to 200%", hi.document.documentElement.style.zoom === "200%");
  const bad = makeContext({ storage: { "dsh-zoom": "junk" } });
  check("garbage falls back to 100%", bad.document.documentElement.style.zoom === "100%");
}

// ---- 3. Find -----------------------------------------------------------------
console.log("findzoom: find");
{
  const ctx = makeContext({ bodyText: "hello world, hello again" });
  const bar = () => ctx.document.getElementById("dsh-fz-bar");
  const input = () => ctx.document.getElementById("dsh-fz-input");
  const count = () => ctx.document.getElementById("dsh-fz-count").textContent;

  check("bar starts closed", !bar().classList.contains("open"));
  const e = ctx.keydown({ key: "f", metaKey: true });
  check("Cmd+F opens (prevented)", e.defaultPrevented && bar().classList.contains("open"));
  check("input focused + selected", ctx.document.activeElement === input());

  input().value = "hello";
  input().dispatch("input");
  await sleep(250); // debounce (150ms) fires
  check("counts 2 matches", count() === "1 of 2", `got "${count()}"`);
  check("native find navigated", ctx.findCalls.at(-1) === "hello");

  input().dispatch("keydown", { key: "Enter" });
  check("Enter goes to next match", ctx.findCalls.length >= 2 && count() === "2 of 2", `got "${count()}"`);
  input().dispatch("keydown", { key: "Enter", shiftKey: true });
  check("Shift+Enter wraps to previous", count() === "1 of 2", `got "${count()}"`);

  input().value = "missing-word";
  input().dispatch("input");
  await sleep(250);
  check("no-result label", count() === "No results", `got "${count()}"`);

  const clearedBefore = ctx.selectionCleared();
  input().dispatch("keydown", { key: "Escape" });
  check("Esc closes the bar", !bar().classList.contains("open"));
  check("Esc clears the selection", ctx.selectionCleared() > clearedBefore);
  check("bar keeps working after close", (() => {
    ctx.keydown({ key: "f", ctrlKey: true });
    return bar().classList.contains("open");
  })());
}

if (failures) {
  console.error(`\ntest-findzoom: ${failures} failure(s)`);
  process.exit(1);
}
console.log("\ntest-findzoom: all green");
