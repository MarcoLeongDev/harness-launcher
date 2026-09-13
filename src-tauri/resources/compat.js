// Harness Launcher WebKit-compat shims — runs FIRST at documentStart in the
// harness webview (before overlay.js, findzoom.js and any engine-served code).
//
// Root cause it covers (harness 0.1.5-rc.x, fresh installs on older Intel Macs):
//   "Failed to load plugins / failed to import loader entry <id>
//    (@deepseek-ai/dsh-client-ui-sidebar-documentpreview): Can't find
//    variable: Iterator"
//
// The documentpreview client bundle (pdfjs-dist 6.x) executes, at import time:
//     if (typeof Iterator.prototype.join !== "function")
//       Iterator.prototype.join = function (sep) { return [...this].join(sep); };
// `typeof Iterator.prototype...` dereferences the bare `Iterator` global first,
// so on WebKit without the ES2025 Iterator Helpers (Safari < 18.4) the import
// itself throws ReferenceError. Cordis wraps that as "failed to import loader
// entry ...", the boot screen shows "Failed to load plugins", and the whole UI
// is bricked — even though the engine process is healthy. The plugin's own
// code additionally uses `Array.prototype.findLast` (Safari 15.4+), and the
// bundled pdf.js uses `Object.hasOwn`, `Array.prototype.at`,
// `Promise.withResolvers`, `structuredClone` and `ReadableStream` async
// iteration, all missing on the older WKWebViews the launcher still supports
// (minimumSystemVersion 11.0 -> Safari 14).
//
// This file provides small, guarded, best-effort polyfills so the latest
// engine boots on those WebViews. Everything is feature-detected (never
// overwrites a native implementation), idempotent (safe across
// re-navigation), and written in conservative syntax so this file itself
// parses on the oldest supported WebKit (no optional chaining, no nullish
// coalescing, no class or async syntax, no template literals, no spread).
(function () {
  if (window.__DSH_COMPAT__) {
    return;
  }
  window.__DSH_COMPAT__ = true;

  const globalObj =
    typeof window !== "undefined" ? window : typeof globalThis !== "undefined" ? globalThis : this;

  function define(target, name, fn) {
    try {
      if (target && typeof target[name] === "undefined") {
        target[name] = fn;
      }
    } catch (_e) {
      /* read-only host object: leave it alone */
    }
  }

  function defineProto(prototype, name, fn) {
    try {
      if (prototype && typeof prototype[name] === "undefined") {
        // Non-enumerable like the native methods, so for-in loops keep
        // behaving the way engine code expects.
        try {
          Object.defineProperty(prototype, name, {
            value: fn,
            writable: true,
            configurable: true,
            enumerable: false,
          });
        } catch (_def) {
          prototype[name] = fn;
        }
      }
    } catch (_e) {
      /* ignore */
    }
  }

  // ---- Iterator -----------------------------------------------------------
  // The critical one: pdfjs-dist reads `Iterator.prototype.join` unguarded at
  // module top-level. Without an `Iterator` global that line throws before any
  // plugin code runs. Provide the global backed by the intrinsic
  // %IteratorPrototype% when it exists so Array.from(this) keeps working,
  // then ensure `join` itself.
  (function polyfillIterator() {
    let hasIterator = false;
    try {
      hasIterator = typeof globalObj.Iterator !== "undefined";
    } catch (_e) {
      hasIterator = false;
    }
    if (!hasIterator) {
      let iteratorProto = null;
      try {
        if (typeof Symbol !== "undefined" && Symbol.iterator && typeof Object.getPrototypeOf === "function") {
          const arrayIter = [][Symbol.iterator]();
          const arrayIterProto = Object.getPrototypeOf(arrayIter);
          // %IteratorPrototype% is the parent of the Array iterator prototype
          // (ES2015+); fall back to the Array iterator prototype itself when
          // the chain is shorter on very old engines.
          const parentProto = Object.getPrototypeOf(arrayIterProto);
          iteratorProto = parentProto || arrayIterProto;
        }
      } catch (_e) {
        iteratorProto = null;
      }
      if (!iteratorProto) {
        iteratorProto = {};
        try {
          if (typeof Symbol !== "undefined" && Symbol.iterator) {
            iteratorProto[Symbol.iterator] = function () {
              return this;
            };
          }
        } catch (_e) {
          /* ignore */
        }
      }
      const Stub = function Iterator() {
        throw new TypeError("Iterator is abstract");
      };
      try {
        Stub.prototype = iteratorProto;
      } catch (_e) {
        /* ignore */
      }
      define(globalObj, "Iterator", Stub);
      // The assignment above may have failed on a frozen global; re-check.
      try {
        hasIterator = typeof globalObj.Iterator !== "undefined";
      } catch (_e) {
        hasIterator = false;
      }
      if (!hasIterator) {
        return;
      }
    }
    let IteratorCtor = null;
    try {
      IteratorCtor = globalObj.Iterator;
    } catch (_e) {
      return;
    }
    if (!IteratorCtor || !IteratorCtor.prototype) {
      return;
    }

    // Iterator.prototype.join (proposal-iterator-join, used by pdf.js to avoid
    // intermediate Arrays). Eager via Array.from: correct for the finite
    // iterables pdf.js joins; infinite iterators would hang either way.
    if (typeof IteratorCtor.prototype.join !== "function") {
      defineProto(IteratorCtor.prototype, "join", function (separator) {
        const sep = typeof separator === "undefined" ? "," : String(separator);
        let parts = [];
        let consumed = false;
        try {
          parts = Array.from(this);
          consumed = true;
        } catch (_e) {
          consumed = false;
        }
        if (!consumed) {
          // Manual fallback for iterators Array.from cannot consume.
          try {
            if (this && typeof this.next === "function") {
              let guard = 0;
              let finished = false;
              while (!finished && guard < 1000000) {
                const step = this.next();
                if (step.done) {
                  finished = true;
                } else {
                  parts.push(step.value);
                  guard++;
                }
              }
            }
          } catch (_inner) {
            /* leave parts as collected */
          }
        }
        return Array.prototype.join.call(parts, sep);
      });
    }

    // Iterator.from: minimal eager-compatible form so future engine code that
    // wraps iterables keeps working. Returns an object inheriting from
    // Iterator.prototype with a delegating next().
    if (typeof IteratorCtor.from !== "function") {
      try {
        IteratorCtor.from = (obj) => {
          let inner = null;
          if (obj != null) {
            if (typeof obj.next === "function") {
              inner = obj;
            } else if (
              typeof Symbol !== "undefined" &&
              Symbol.iterator &&
              typeof obj[Symbol.iterator] === "function"
            ) {
              try {
                inner = obj[Symbol.iterator]();
              } catch (_e) {
                inner = null;
              }
            }
          }
          if (!inner || typeof inner.next !== "function") {
            throw new TypeError("Iterator.from: argument is not iterable");
          }
          const captured = inner;
          const wrapper = Object.create(IteratorCtor.prototype);
          wrapper.next = () => captured.next();
          try {
            if (typeof Symbol !== "undefined" && Symbol.iterator) {
              wrapper[Symbol.iterator] = () => wrapper;
            }
          } catch (_e) {
            /* ignore */
          }
          return wrapper;
        };
      } catch (_e) {
        /* ignore */
      }
    }

    // Iterator.prototype.toArray: eager helper so engine code that drains an
    // iterator keeps working on Safari < 18.4. Other ES2025 helpers (map,
    // filter, take, ...) are deliberately not polyfilled: engine code must
    // not silently depend on lazy semantics this shim cannot provide.
    defineProto(IteratorCtor.prototype, "toArray", function () {
      return Array.from(this);
    });
  })();

  // ---- Array.prototype.findLast / findLastIndex (ES2023, Safari 15.4+) -----
  // Used by the documentpreview plugin itself to pick PDF filenames out of URLs.
  defineProto(Array.prototype, "findLast", function (predicate, thisArg) {
    if (this == null) {
      throw new TypeError("findLast called on null");
    }
    if (typeof predicate !== "function") {
      throw new TypeError("findLast predicate must be a function");
    }
    const list = Object(this);
    const len = list.length >>> 0;
    for (let i = len - 1; i >= 0; i--) {
      const value = list[i];
      if (predicate.call(thisArg, value, i, list)) {
        return value;
      }
    }
    return undefined;
  });
  defineProto(Array.prototype, "findLastIndex", function (predicate, thisArg) {
    if (this == null) {
      throw new TypeError("findLastIndex called on null");
    }
    if (typeof predicate !== "function") {
      throw new TypeError("findLastIndex predicate must be a function");
    }
    const list = Object(this);
    const len = list.length >>> 0;
    for (let i = len - 1; i >= 0; i--) {
      if (predicate.call(thisArg, list[i], i, list)) {
        return i;
      }
    }
    return -1;
  });

  // ---- Array.prototype.at (ES2022, Safari 15.4+) ---------------------------
  defineProto(Array.prototype, "at", function (index) {
    if (this == null) {
      throw new TypeError("at called on null");
    }
    const list = Object(this);
    const len = list.length >>> 0;
    let n = Number(index);
    if (Number.isNaN(n)) {
      n = 0;
    } else if (n < 0) {
      n = Math.ceil(n);
    } else {
      n = Math.floor(n);
    }
    const k = n >= 0 ? n : len + n;
    if (k < 0 || k >= len) {
      return undefined;
    }
    return list[k];
  });

  // ---- Object.hasOwn (ES2022, Safari 15.4+) --------------------------------
  if (typeof Object === "function" && typeof Object.hasOwn !== "function") {
    define(Object, "hasOwn", (obj, prop) => {
      if (obj == null) {
        throw new TypeError("hasOwn called on null");
      }
      // biome-ignore lint/suspicious/noPrototypeBuiltins: this IS the
      // hasOwn polyfill — it must call the underlying prototype method,
      // never itself (auto-fixing to Object.hasOwn would recurse forever).
      return Object.prototype.hasOwnProperty.call(Object(obj), prop);
    });
  }

  // ---- Promise.withResolvers (ES2024, Safari 18+) --------------------------
  // Used heavily by the bundled pdf.js.
  if (typeof Promise === "function" && typeof Promise.withResolvers !== "function") {
    define(Promise, "withResolvers", () => {
      let resolveFn;
      let rejectFn;
      const promise = new Promise((resolve, reject) => {
        resolveFn = resolve;
        rejectFn = reject;
      });
      return { promise: promise, resolve: resolveFn, reject: rejectFn };
    });
  }

  // ---- structuredClone (Safari 15.4+) --------------------------------------
  // Best-effort only: JSON round-trip covers the plain-data clones the preview
  // UI performs. Anything with functions, DOM nodes, Blobs or cycles keeps
  // throwing (loud, like the native) instead of silently corrupting.
  if (typeof globalObj.structuredClone !== "function") {
    define(globalObj, "structuredClone", (value) => {
      if (value === undefined) {
        return undefined;
      }
      try {
        return JSON.parse(JSON.stringify(value));
      } catch (_e) {
        throw new Error("structuredClone polyfill: value is not cloneable");
      }
    });
  }

  // ---- URLSearchParams.size -------------------------------------------------
  // The preview plugin tests `searchParams.size > 0`; without the getter the
  // expression is `undefined > 0` (false) — no crash, but PDF names hidden in
  // query strings are missed. Count via forEach when the getter is absent.
  try {
    if (
      typeof URLSearchParams !== "undefined" &&
      URLSearchParams.prototype &&
      !("size" in URLSearchParams.prototype)
    ) {
      try {
        Object.defineProperty(URLSearchParams.prototype, "size", {
          get: function () {
            let count = 0;
            try {
              this.forEach(() => {
                count++;
              });
            } catch (_e) {
              /* leave count */
            }
            return count;
          },
          configurable: true,
          enumerable: false,
        });
      } catch (_e) {
        /* ignore */
      }
    }
  } catch (_e) {
    /* ignore */
  }

  // ---- ReadableStream async iteration ---------------------------------------
  // pdf.js iterates `streamTextContent()` with `for await...of`. Safari has no
  // ReadableStream asyncIterator through 26.x, so PDF text extraction throws
  // on old WebKit even after the import succeeds. Bridge via getReader()
  // without async-generator syntax so this file still parses everywhere.
  try {
    if (
      typeof ReadableStream !== "undefined" &&
      ReadableStream.prototype &&
      typeof Symbol !== "undefined" &&
      Symbol.asyncIterator &&
      !ReadableStream.prototype[Symbol.asyncIterator]
    ) {
      const asyncIterKey = Symbol.asyncIterator;
      const readableAsyncIterator = function () {
        let reader = null;
        try {
          reader = this.getReader();
        } catch (_e) {
          throw new TypeError("ReadableStream polyfill: getReader failed");
        }
        const captured = reader;
        let streamDone = false;
        const finish = () => {
          streamDone = true;
          try {
            captured.releaseLock();
          } catch (_e) {
            /* ignore */
          }
        };
        const iterator = {
          next: () => {
            if (streamDone) {
              return Promise.resolve({ value: undefined, done: true });
            }
            return captured.read().then(
              (result) => {
                if (result.done) {
                  finish();
                }
                return result;
              },
              (err) => {
                finish();
                throw err;
              },
            );
          },
          return: (value) => {
            finish();
            return Promise.resolve({ value: value, done: true });
          },
          throw: (err) => {
            finish();
            return Promise.reject(err);
          },
        };
        iterator[asyncIterKey] = () => iterator;
        return iterator;
      };
      try {
        Object.defineProperty(ReadableStream.prototype, asyncIterKey, {
          value: readableAsyncIterator,
          writable: true,
          configurable: true,
          enumerable: false,
        });
      } catch (_e) {
        ReadableStream.prototype[asyncIterKey] = readableAsyncIterator;
      }
    }
  } catch (_e) {
    /* ignore */
  }
})();
