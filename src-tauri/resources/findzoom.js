// Harness Launcher find-in-page + zoom — browser-style Cmd/Ctrl+F and
// Cmd/Ctrl +/−/0 for the harness-content webview only.
// Injected at documentStart by the Rust shell (initialization_script) into the
// main window; it runs on engine-served pages where the overlay panel cannot
// rely on page cooperation. Launcher-owned `dsh-ui:` pages (Control Panel,
// engine-stopped splash) never need browser find — the bar would appear
// unstyled there — so this script bails out early on that protocol.
// Self-contained except for benign IPCs (`set_zoom` window-local page zoom,
// `text_field_focus` menubar focus policy): no network, no page-JS
// interaction — DOM reads/writes only, plus one per-origin localStorage key
// for the zoom level.
//
// Focus contract (latched, never timed): opening the bar latches TYPING and
// lifts the menubar no-focus policy exactly once; typing only counts and
// highlights (never moves selection, never calls focus()); navigation happens
// only on explicit user gestures (Enter / Shift+Enter / prev-next buttons);
// closing latches back exactly once. No timer, poll, or search callback ever
// touches focus or the policy.
(() => {
  if (window.__DSH_FINDZOOM__) return;
  window.__DSH_FINDZOOM__ = true;
  // No find UI on launcher-owned pages (Control Panel / stopped splash).
  try {
    if (window.location && window.location.protocol === "dsh-ui:") return;
  } catch (_e) {
    /* location unreadable: fall through to engine behavior */
  }

  function invoke(cmd, args) {
    const core = window.__TAURI__?.core;
    if (core && typeof core.invoke === "function") return core.invoke(cmd, args || {});
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function")
      return window.__TAURI_INTERNALS__.invoke(cmd, args || {}, undefined);
    return Promise.reject(new Error("Tauri IPC unavailable in this context"));
  }

  function hasIpc() {
    const core = window.__TAURI__?.core;
    if (core && typeof core.invoke === "function") return true;
    const internals = window.__TAURI_INTERNALS__;
    return !!(internals && typeof internals.invoke === "function");
  }

  const FIND_BAR_ID = "dsh-fz-bar";
  const ZOOM_BADGE_ID = "dsh-fz-zoom";
  const ZOOM_KEY = "dsh-zoom";
  const MIN_ZOOM = 50;
  const MAX_ZOOM = 200;
  const ZOOM_STEP = 10;

  // The DOM may not exist yet at documentStart: queue DOM work until ready.
  function onReady(fn) {
    if (document.readyState !== "loading") fn();
    else document.addEventListener("DOMContentLoaded", fn, { once: true });
  }

  // ---- Zoom ---------------------------------------------------------------
  function readZoom() {
    let z = 100;
    try {
      const raw = window.localStorage.getItem(ZOOM_KEY);
      if (raw !== null) z = parseInt(raw, 10);
    } catch (_e) {
      /* storage unavailable: session-only zoom */
    }
    // Strict range, never clamp: only levels this UI can write are honored,
    // so stale values from older builds (e.g. "1") reset to 100% instead of
    // landing on a 50% floor that breaks page layout.
    if (!Number.isInteger(z) || z < MIN_ZOOM || z > MAX_ZOOM) z = 100;
    return z;
  }

  // CSS-zoom fallback for contexts without Tauri IPC (tests, plain browsers).
  // Never used inside the launcher: native page zoom keeps every page metric
  // (innerWidth, getBoundingClientRect, offsetWidth) in one coordinate space,
  // while root CSS zoom mixes scaled and unscaled units and misplaces
  // fixed-anchored popups (e.g. the engine model menu drifting off-screen at
  // any non-100% level).
  function cssZoom(z) {
    try {
      if (document.documentElement) document.documentElement.style.zoom = `${z}%`;
    } catch (_e) {
      /* headless/shadow contexts */
    }
  }

  function applyZoom(z) {
    z = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, Math.round(z)));
    try {
      window.localStorage.setItem(ZOOM_KEY, String(z));
    } catch (_e) {
      /* ignore */
    }
    // Native webview zoom first; clear any stale CSS zoom once it lands so
    // the two never stack.
    if (!hasIpc()) {
      cssZoom(z);
      return z;
    }
    invoke("set_zoom", { scale: z / 100 })
      .then(() => {
        try {
          if (document.documentElement) document.documentElement.style.zoom = "";
        } catch (_e) {
          /* ignore */
        }
      })
      .catch(() => {
        cssZoom(z);
      });
    return z;
  }

  let badgeTimer = null;
  function showZoomBadge(z) {
    const badge = document.getElementById(ZOOM_BADGE_ID);
    if (!badge) return;
    badge.textContent = `${z}%`;
    badge.classList.add("on");
    if (badgeTimer) clearTimeout(badgeTimer);
    badgeTimer = setTimeout(() => {
      badge.classList.remove("on");
    }, 1200);
  }

  let currentZoom = readZoom();
  onReady(() => {
    currentZoom = applyZoom(currentZoom); // restore persisted level, no badge
  });

  function zoomBy(delta) {
    currentZoom = applyZoom(currentZoom + delta);
    showZoomBadge(currentZoom);
  }
  function zoomReset() {
    currentZoom = applyZoom(100);
    showZoomBadge(currentZoom);
  }

  // ---- Find bar ------------------------------------------------------------
  let lastQuery = "";
  let searchedQuery = null; // query currently reflected by count/highlights
  let lastIndex = 0; // 0 = counted but not navigated; else 1-based current match
  let totalMatches = 0;
  let debounceTimer = null;
  let savedFocus = null;

  // Latched focus-policy state. All `text_field_focus` sends go through
  // setPolicy, which dedupes: one send on entering TYPING, silence while
  // searching, one send on genuine exit. Transient blurs (e.g. selection
  // changes from `window.find`) never flap the OS policy.
  // `null` means "unknown" (never sent, or the last send failed): the next
  // evaluation re-sends instead of trusting a stale latch — so a rejected
  // IPC or another window's OFF send can never leave typing starved.
  let focusPolicyOn = null;
  let findOpen = false;
  function setPolicy(on) {
    on = !!on;
    if (on === focusPolicyOn) return;
    focusPolicyOn = on;
    invoke("text_field_focus", { focused: on }).catch(() => {
      // Send failed: forget the latch so the next focus/blur re-asserts.
      focusPolicyOn = null;
    });
  }
  function isEditable(el) {
    if (!el || el.nodeType !== 1) return false;
    const tag = (el.tagName || el.nodeName || "").toUpperCase();
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    try {
      if (el.isContentEditable) return true;
    } catch (_e) {
      /* ignore */
    }
    return false;
  }
  function clearSelection() {
    try {
      if (window.getSelection) window.getSelection().removeAllRanges();
    } catch (_e) {
      /* ignore */
    }
  }

  function uiRoot() {
    return document.getElementById(FIND_BAR_ID);
  }

  function isUiNode(node) {
    const bar = uiRoot();
    const badge = document.getElementById(ZOOM_BADGE_ID);
    while (node) {
      if (node === bar || node === badge) return true;
      node = node.parentNode;
    }
    return false;
  }

  // Count matches without touching the DOM (non-destructive: never wraps page
  // content, so framework-driven pages are unaffected). Skips our own UI and
  // non-rendered elements.
  function countMatches(query) {
    if (!query) return 0;
    const q = query.toLowerCase();
    let count = 0;
    let walker;
    try {
      walker = document.createTreeWalker(document.body || document.documentElement, NodeFilter.SHOW_TEXT, {
        acceptNode: (node) => {
          if (!node.nodeValue || node.nodeValue.toLowerCase().indexOf(q) === -1) {
            return NodeFilter.FILTER_SKIP;
          }
          const parent = node.parentNode;
          if (!parent) return NodeFilter.FILTER_REJECT;
          const tag = parent.nodeName;
          if (
            tag === "SCRIPT" ||
            tag === "STYLE" ||
            tag === "NOSCRIPT" ||
            tag === "TEXTAREA" ||
            tag === "INPUT"
          ) {
            return NodeFilter.FILTER_REJECT;
          }
          if (isUiNode(node)) return NodeFilter.FILTER_REJECT;
          return NodeFilter.FILTER_ACCEPT;
        },
      });
    } catch (_e) {
      return 0;
    }
    let guard = 0;
    while (walker.nextNode()) {
      const text = walker.currentNode.nodeValue.toLowerCase();
      let from = 0;
      let at = text.indexOf(q, from);
      while (at !== -1) {
        count++;
        from = at + q.length;
        if (++guard > 20000) return count; // pathological pages: cap the walk
        at = text.indexOf(q, from);
      }
      if (++guard > 20000) break;
    }
    return count;
  }

  // Best-effort all-match highlight via the CSS Custom Highlight API
  // (non-destructive ranges; silently skipped where unsupported).
  function highlightAll(query) {
    try {
      clearHighlight();
      if (
        !query ||
        !("highlights" in CSS) ||
        typeof Highlight === "undefined" ||
        typeof Range === "undefined"
      ) {
        return;
      }
      const q = query.toLowerCase();
      const walker = document.createTreeWalker(
        document.body || document.documentElement,
        NodeFilter.SHOW_TEXT,
        {
          acceptNode: (node) => {
            if (!node.nodeValue || node.nodeValue.toLowerCase().indexOf(q) === -1) {
              return NodeFilter.FILTER_SKIP;
            }
            const parent = node.parentNode;
            if (!parent) return NodeFilter.FILTER_REJECT;
            const tag = parent.nodeName;
            if (
              tag === "SCRIPT" ||
              tag === "STYLE" ||
              tag === "NOSCRIPT" ||
              tag === "TEXTAREA" ||
              tag === "INPUT"
            ) {
              return NodeFilter.FILTER_REJECT;
            }
            if (isUiNode(node)) return NodeFilter.FILTER_REJECT;
            return NodeFilter.FILTER_ACCEPT;
          },
        },
      );
      const ranges = [];
      let guard = 0;
      while (walker.nextNode() && ranges.length < 1000) {
        const node = walker.currentNode;
        const text = node.nodeValue.toLowerCase();
        let from = 0;
        let at = text.indexOf(q, from);
        while (at !== -1 && ranges.length < 1000) {
          const r = new Range();
          r.setStart(node, at);
          r.setEnd(node, at + q.length);
          ranges.push(r);
          from = at + q.length;
          if (++guard > 20000) break;
          at = text.indexOf(q, from);
        }
        if (guard > 20000) break;
      }
      if (ranges.length) CSS.highlights.set("dsh-find", new Highlight.apply(null, ranges));
    } catch (_e) {
      /* highlight is decorative; navigation still works */
    }
  }

  function clearHighlight() {
    try {
      if ("highlights" in CSS) CSS.highlights.delete("dsh-find");
    } catch (_e) {
      /* ignore */
    }
  }

  function updateCount() {
    const bar = uiRoot();
    if (!bar) return;
    const label = bar.querySelector("#dsh-fz-count");
    if (!label) return;
    if (!lastQuery) {
      label.textContent = "";
      return;
    }
    if (totalMatches === 0) label.textContent = "No results";
    else if (lastIndex < 1) label.textContent = `${totalMatches} match${totalMatches === 1 ? "" : "es"}`;
    else label.textContent = `${Math.min(lastIndex, totalMatches)} of ${totalMatches}`;
  }

  // Navigate with the platform-native find (selection + scroll into view),
  // which works in every webview without us moving the user's viewport by hand.
  // Returns whether a match was selected.
  function nativeFind(query, backwards) {
    try {
      return window.find(query, false, !!backwards, true, false, false, false);
    } catch (_e) {
      return false;
    }
  }

  // Hold the field after a user-initiated navigation (Enter / prev-next).
  // Gesture-only: called synchronously inside user-gesture handlers so a
  // selection-stealing page cannot strand focus. Never called on timers,
  // polls, debounces, or search callbacks.
  function holdField() {
    try {
      const bar = uiRoot();
      const field = bar ? bar.querySelector("#dsh-fz-input") : null;
      if (field && document.activeElement !== field) field.focus();
    } catch (_e) {
      /* element may be gone */
    }
  }

  function findNavigate(backwards) {
    if (!lastQuery) return;
    // First gesture on this query: start from a clean selection so forward
    // lands on match 1 and backward on match N (typing stays passive, so it
    // may have left an unrelated page selection behind).
    if (lastIndex < 1) clearSelection();
    if (nativeFind(lastQuery, backwards)) {
      // Advance the "n of m" indicator in navigation order (wraps around).
      if (totalMatches > 0) {
        lastIndex = backwards
          ? lastIndex <= 1
            ? totalMatches
            : lastIndex - 1
          : lastIndex >= totalMatches
            ? 1
            : lastIndex + 1;
      }
    }
    holdField();
    updateCount();
  }

  // Passive recount: count + highlight + label only. Never moves the
  // selection, never clears it, never touches focus — safe to run on the
  // typing debounce while the user is mid-word.
  function runSearch() {
    const bar = uiRoot();
    if (!bar) return;
    const input = bar.querySelector("#dsh-fz-input");
    const query = input ? input.value : "";
    lastQuery = query;
    totalMatches = countMatches(query);
    highlightAll(query);
    lastIndex = 0; // counted but not navigated; Enter navigates on gesture
    searchedQuery = query;
    updateCount();
  }

  function scheduleSearch() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      debounceTimer = null;
      runSearch();
    }, 150);
  }

  function openFind() {
    const bar = uiRoot();
    if (!bar) return;
    // Latch TYPING exactly once, BEFORE focusing, so the first keystroke
    // already lands in the field. Later focusin noise is deduped by setPolicy.
    findOpen = true;
    setPolicy(true);
    if (!bar.classList.contains("open")) {
      try {
        savedFocus = document.activeElement;
      } catch (_e) {
        savedFocus = null;
      }
      bar.classList.add("open");
    }
    const input = bar.querySelector("#dsh-fz-input");
    if (input) {
      input.focus();
      input.select();
    }
    // Passive recount when reopened with a (possibly page-changed) query;
    // never navigates on its own.
    runSearch();
  }

  function closeFind() {
    const bar = uiRoot();
    if (!bar) return;
    findOpen = false;
    // Drop any pending recount: a debounce armed just before Esc would
    // otherwise fire after close and re-apply highlights to a closed bar.
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    bar.classList.remove("open");
    lastQuery = "";
    searchedQuery = null;
    totalMatches = 0;
    lastIndex = 0;
    clearHighlight();
    try {
      if (window.getSelection) window.getSelection().removeAllRanges();
    } catch (_e) {
      /* ignore */
    }
    updateCount();
    if (savedFocus?.focus) {
      try {
        savedFocus.focus();
      } catch (_e) {
        /* element may be gone */
      }
      savedFocus = null;
    }
    // Never leave focus on the now-hidden field: further keystrokes would go
    // nowhere visible, and Cmd+F would reselect instead of reopening.
    // blur() first (real de-focus); body.focus() only as a fallback for the
    // case where blur did not take.
    try {
      const field = bar.querySelector("#dsh-fz-input");
      if (field && document.activeElement === field) {
        if (typeof field.blur === "function") field.blur();
        if (document.activeElement === field && document.body && typeof document.body.focus === "function") {
          document.body.focus();
        }
      }
    } catch (_e) {
      /* ignore */
    }
    // Latched exit: exactly one policy send based on where focus landed
    // (saved element when restored, body otherwise). The focusout tick below
    // will also fire but dedupes to a no-op.
    try {
      let editable = false;
      try {
        editable = isEditable(document.activeElement);
      } catch (_e2) {
        editable = false;
      }
      setPolicy(editable);
    } catch (_e) {
      /* ignore */
    }
  }

  function buildBar() {
    if (uiRoot()) return;
    const root = document.body || document.documentElement;
    if (!root) return;
    const bar = document.createElement("div");
    bar.id = FIND_BAR_ID;
    bar.setAttribute("role", "search");
    bar.setAttribute("aria-label", "Find in page");
    // Hidden until opened (Cmd/Ctrl+F); the `hidden`-like default keeps it out
    // of the tab order and out of assistive-tech browse mode when closed.
    bar.innerHTML =
      '<input id="dsh-fz-input" type="text" placeholder="Find in page" aria-label="Find in page">' +
      '<span id="dsh-fz-count" aria-live="polite"></span>' +
      '<button id="dsh-fz-prev" aria-label="Previous match" title="Previous (Shift+Enter)">&#9650;</button>' +
      '<button id="dsh-fz-next" aria-label="Next match" title="Next (Enter)">&#9660;</button>' +
      '<button id="dsh-fz-close" aria-label="Close find bar" title="Close (Esc)">&#10005;</button>';
    (document.body || root).appendChild(bar);

    const input = bar.querySelector("#dsh-fz-input");
    input.addEventListener("input", scheduleSearch);
    // Swallow key events that must not reach the page (keypress/keyup have no
    // behavior of their own here; blocking them starves page-level thieves).
    input.addEventListener("keypress", (e) => {
      e.stopPropagation();
    });
    input.addEventListener("keyup", (e) => {
      e.stopPropagation();
    });
    input.addEventListener("keydown", (e) => {
      const mod = e.metaKey || e.ctrlKey;
      const key = e.key;
      // Modified keys other than our own shortcuts (copy/paste, etc.) keep
      // their default behavior and are page-irrelevant: let them through.
      if (
        mod &&
        key !== "f" &&
        key !== "F" &&
        key !== "+" &&
        key !== "=" &&
        key !== "-" &&
        key !== "_" &&
        key !== "0"
      ) {
        return;
      }
      // Anything else typed here belongs to the find UI. Keep it from
      // bubbling to page listeners: web apps that refocus their composer on
      // any keydown would otherwise steal the field on every keystroke.
      e.stopPropagation();
      if (mod && (key === "f" || key === "F")) {
        e.preventDefault();
        input.select();
        return;
      }
      // Zoom stays available while the find field has focus (as in browsers).
      if (mod && (key === "+" || key === "=")) {
        e.preventDefault();
        zoomBy(ZOOM_STEP);
        return;
      }
      if (mod && (key === "-" || key === "_")) {
        e.preventDefault();
        zoomBy(-ZOOM_STEP);
        return;
      }
      if (mod && key === "0") {
        e.preventDefault();
        zoomReset();
        return;
      }
      if (key === "Enter") {
        e.preventDefault();
        // Flush any pending keystrokes, recount passively, then navigate
        // exactly once. The old path auto-navigated inside runSearch AND here,
        // double-stepping past the first match.
        if (debounceTimer) {
          clearTimeout(debounceTimer);
          debounceTimer = null;
        }
        if (input.value !== searchedQuery) runSearch();
        findNavigate(!!e.shiftKey);
      } else if (e.key === "Escape") {
        e.preventDefault();
        closeFind();
      }
    });
    bar.querySelector("#dsh-fz-prev").addEventListener("click", () => {
      findNavigate(true);
    });
    bar.querySelector("#dsh-fz-next").addEventListener("click", () => {
      findNavigate(false);
    });
    bar.querySelector("#dsh-fz-close").addEventListener("click", closeFind);
  }

  function injectStyles() {
    if (document.getElementById("dsh-fz-style")) return;
    const head = document.head || document.documentElement;
    if (!head) return;
    const style = document.createElement("style");
    style.id = "dsh-fz-style";
    style.textContent =
      "#" +
      FIND_BAR_ID +
      " { all: initial; display: none; position: fixed; top: 12px; right: 12px;" +
      " z-index: 2147483646; align-items: center; gap: 6px;" +
      ' font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;' +
      " font-size: 13px; color: #e6edf3; background: rgba(15, 23, 42, 0.95);" +
      " border: 1px solid #334155; border-radius: 8px; padding: 6px 8px;" +
      " box-shadow: 0 8px 28px rgba(0,0,0,0.45); }" +
      "#" +
      FIND_BAR_ID +
      ".open { display: flex; }" +
      "#" +
      FIND_BAR_ID +
      " * { box-sizing: border-box; }" +
      "#dsh-fz-input { background: #0f172a; color: #e6edf3; border: 1px solid #475569;" +
      " border-radius: 6px; padding: 5px 8px; font-size: 13px; width: 200px; outline: none; }" +
      "#dsh-fz-input:focus { border-color: #60a5fa; }" +
      "#dsh-fz-count { color: #94a3b8; font-size: 12px; min-width: 64px; text-align: right;" +
      " white-space: nowrap; }" +
      "#" +
      FIND_BAR_ID +
      " button { background: transparent; border: 1px solid transparent;" +
      " color: #cbd5e1; border-radius: 6px; padding: 4px 8px; font-size: 12px; cursor: pointer; }" +
      "#" +
      FIND_BAR_ID +
      " button:hover { background: #1e293b; }" +
      "#" +
      FIND_BAR_ID +
      " button:focus-visible { outline: 2px solid #60a5fa; outline-offset: 1px; }" +
      "#" +
      ZOOM_BADGE_ID +
      " { all: initial; display: none; position: fixed; right: 16px; bottom: 16px;" +
      ' z-index: 2147483646; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;' +
      " font-size: 13px; color: #e6edf3; background: rgba(15, 23, 42, 0.92);" +
      " border: 1px solid #334155; border-radius: 999px; padding: 6px 12px; }" +
      "#" +
      ZOOM_BADGE_ID +
      ".on { display: block; }" +
      "::highlight(dsh-find) { background: rgba(250, 204, 21, 0.55); color: inherit; }";
    head.appendChild(style);
  }

  function buildBadge() {
    if (document.getElementById(ZOOM_BADGE_ID)) return;
    const badge = document.createElement("div");
    badge.id = ZOOM_BADGE_ID;
    badge.setAttribute("aria-live", "polite");
    (document.body || document.documentElement).appendChild(badge);
  }

  onReady(() => {
    injectStyles();
    buildBar();
    buildBadge();
  });

  // ---- Menubar focus policy (latched) ---------------------------------------
  // The app idles as a menubar Accessory (never steals focus). TYPING is
  // latched by openFind/closeFind above (setPolicy dedupes), so this section
  // only covers page-owned editables (e.g. the harness composer): lift on
  // focusin, restore on settled blur. While the find bar is open the latch
  // is held unconditionally — transient blurs from selection changes or
  // timers never send OFF mid-search.
  document.addEventListener("focusin", (e) => {
    if (isEditable(e.target)) setPolicy(true);
  });
  document.addEventListener("focusout", () => {
    setTimeout(() => {
      // Re-check at FIRE time: the bar may have opened between scheduling and
      // now, in which case the search owns the policy and this is a no-op.
      if (findOpen) return;
      let editable = false;
      try {
        editable = isEditable(document.activeElement);
      } catch (_e) {
        editable = false;
      }
      setPolicy(editable);
    }, 0);
  });
  // App-switch boundaries: window blur restores Accessory and clears the
  // latch, so re-deriving on window focus re-asserts TYPING (bar open or an
  // editable focused) even if another window changed the shared OS policy.
  // Latch-deduped, so a single activation cannot send twice or loop back.
  window.addEventListener("blur", () => {
    setPolicy(false);
  });
  window.addEventListener("focus", () => {
    if (findOpen) {
      setPolicy(true);
      return;
    }
    let editable = false;
    try {
      editable = isEditable(document.activeElement);
    } catch (_e) {
      editable = false;
    }
    setPolicy(editable);
  });

  // ---- Shortcuts ------------------------------------------------------------
  // Browser conventions: Cmd on macOS, Ctrl elsewhere. Handled at the window
  // level in the bubble phase so page inputs keep working, except our own bar
  // input (its key handler stops propagation, so these never double-fire).
  // Consumed shortcuts never reach the page (browsers reserve them too).
  window.addEventListener("keydown", (e) => {
    const mod = e.metaKey || e.ctrlKey;
    if (!mod || e.altKey) return;
    const inFindInput = e.target && e.target.id === "dsh-fz-input";
    const key = e.key;
    const code = e.code || "";

    if ((key === "f" || key === "F") && !e.shiftKey) {
      e.preventDefault();
      e.stopPropagation();
      openFind();
      return;
    }
    if (inFindInput) return; // let the bar's own key handler own other keys
    if (key === "+" || key === "=" || code === "NumpadAdd") {
      e.preventDefault();
      e.stopPropagation();
      zoomBy(ZOOM_STEP);
    } else if (key === "-" || key === "_" || code === "NumpadSubtract") {
      e.preventDefault();
      e.stopPropagation();
      zoomBy(-ZOOM_STEP);
    } else if (key === "0" || code === "Numpad0") {
      e.preventDefault();
      e.stopPropagation();
      zoomReset();
    }
  });

  // Escape outside the input also closes the bar (same as browsers).
  window.addEventListener("keydown", (e) => {
    if (e.key !== "Escape") return;
    if (e.target && e.target.id === "dsh-fz-input") return; // handled above
    const bar = uiRoot();
    if (bar?.classList.contains("open")) closeFind();
  });
})();
