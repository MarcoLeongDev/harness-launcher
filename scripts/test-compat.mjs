// Behavioral tests for src-tauri/resources/compat.js (old-WebKit shims).
//
// Simulates the pre-18.4 Safari WKWebView that bricks fresh installs:
//
//   "Failed to load plugins / failed to import loader entry <id>
//    (@deepseek-ai/dsh-client-ui-sidebar-documentpreview):
//    Can't find variable: Iterator"
//
// The documentpreview bundle (pdfjs-dist) runs at import time:
//   if (typeof Iterator.prototype.join !== "function") ...
// which throws ReferenceError when the Iterator global is absent. The test
// deletes the modern builtins inside a fresh vm context (old-WebKit shape),
// runs compat.js with `window` aliased to the global (like a real browser),
// and verifies the crash line no longer throws plus the helpers work.
//
// Run: node scripts/test-compat.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const JS = readFileSync(path.join(root, "src-tauri", "resources", "compat.js"), "utf8");

let failures = 0;
function check(name, cond, extra = "") {
  if (cond) console.log(`  PASS ${name}`);
  else {
    failures++;
    console.error(`  FAIL ${name} ${extra}`);
  }
}

// Fresh vm context shaped like old WebKit: modern builtins removed, and
// `window` aliased to the global so `window.X` IS bare `X` (as in browsers).
function makeOldContext() {
  const sandbox = {};
  sandbox.window = sandbox;
  sandbox.globalThis = sandbox;
  vm.createContext(sandbox);
  // Strip modern APIs to simulate Safari < 15.4 / < 18.4. Each delete is
  // guarded: the host Node may or may not provide the API.
  vm.runInContext(
    [
      "try { delete globalThis.Iterator; } catch (e) {}",
      "try { globalThis.Iterator = undefined; } catch (e) {}",
      "try { delete Array.prototype.findLast; } catch (e) {}",
      "try { delete Array.prototype.findLastIndex; } catch (e) {}",
      "try { delete Array.prototype.at; } catch (e) {}",
      "try { delete Object.hasOwn; } catch (e) {}",
      "try { delete Promise.withResolvers; } catch (e) {}",
      "try { delete globalThis.structuredClone; } catch (e) {}",
      // URLSearchParams.size getter (configurable on modern engines).
      "try { delete URLSearchParams.prototype.size; } catch (e) {}",
      // ReadableStream asyncIterator.
      "try { if (typeof Symbol !== 'undefined' && Symbol.asyncIterator && typeof ReadableStream !== 'undefined' && ReadableStream.prototype) delete ReadableStream.prototype[Symbol.asyncIterator]; } catch (e) {}",
    ].join("\n"),
    sandbox,
  );
  return sandbox;
}

function runCompat(sandbox) {
  vm.runInContext(JS, sandbox, { filename: "compat.js" });
}

// ---- 1. Boots once ----------------------------------------------------------
console.log("compat: boots once");
{
  const ctx = makeOldContext();
  runCompat(ctx);
  check("install guard set", vm.runInContext("window.__DSH_COMPAT__ === true", ctx));
  runCompat(ctx);
  check("re-evaluation keeps guard", vm.runInContext("window.__DSH_COMPAT__ === true", ctx));
}

// ---- 2. The exact pdfjs crash line ------------------------------------------
console.log("compat: Iterator crasher");
{
  const ctx = makeOldContext();
  // Without the shim, the pdfjs import-time line throws. In real Safari this
  // is ReferenceError "Can't find variable: Iterator"; in the Node vm it is
  // the same shape (ReferenceError when deleted, TypeError when stubbed to
  // undefined) — either way the import cannot proceed.
  let threw = false;
  try {
    vm.runInContext('typeof Iterator.prototype.join !== "function"', ctx);
  } catch (e) {
    threw = true;
  }
  check("old WebKit throws on the pdfjs line", threw);
  runCompat(ctx);
  let ok = false;
  let joinType = "";
  try {
    joinType = vm.runInContext("typeof Iterator.prototype.join", ctx);
    ok = joinType === "function";
  } catch (e) {
    ok = false;
  }
  check("Iterator.prototype.join exists after shim", ok, `got ${joinType}`);
}

// ---- 3. Iterator.join works --------------------------------------------------
console.log("compat: Iterator.join");
{
  const ctx = makeOldContext();
  runCompat(ctx);
  const out = vm.runInContext("Iterator.from([1, 2, 3]).join('-')", ctx);
  check("join concatenates via Iterator.from", out === "1-2-3", `got ${out}`);
  const empty = vm.runInContext("Iterator.from([]).join(',')", ctx);
  check("join of empty iterator is empty string", empty === "", `got ${empty}`);
}

// ---- 4. findLast / findLastIndex ---------------------------------------------
console.log("compat: findLast");
{
  const ctx = makeOldContext();
  runCompat(ctx);
  // Mirrors the plugin's own usage: pick the last PDF-looking param value.
  const found = vm.runInContext(
    "[...['a.txt', 'b.pdf', 'c.txt']].findLast(function (v) { return /\\.pdf$/i.test(v); })",
    ctx,
  );
  check("findLast picks last match", found === "b.pdf", `got ${found}`);
  const idx = vm.runInContext("[1, 2, 3, 2].findLastIndex(function (v) { return v === 2; })", ctx);
  check("findLastIndex picks last index", idx === 3, `got ${idx}`);
  const miss = vm.runInContext("[1, 2].findLast(function () { return false; })", ctx);
  check("findLast miss is undefined", miss === undefined, `got ${miss}`);
}

// ---- 5. at / hasOwn / withResolvers ------------------------------------------
console.log("compat: at/hasOwn/withResolvers");
{
  const ctx = makeOldContext();
  runCompat(ctx);
  check("at(-1) reads from end", vm.runInContext("[1, 2, 3].at(-1)", ctx) === 3);
  check("at(out of range) is undefined", vm.runInContext("[1].at(5)", ctx) === undefined);
  check("Object.hasOwn works", vm.runInContext("Object.hasOwn({ a: 1 }, 'a') === true", ctx) === true);
  const wr = vm.runInContext(
    "var w = Promise.withResolvers(); typeof w.promise.then === 'function' && typeof w.resolve === 'function' && typeof w.reject === 'function'",
    ctx,
  );
  check("Promise.withResolvers resolves shape", wr === true);
}

// ---- 6. Natives are never overwritten -----------------------------------------
console.log("compat: preserves natives");
{
  const sandbox = {};
  sandbox.window = sandbox;
  sandbox.globalThis = sandbox;
  vm.createContext(sandbox);
  const nativeFindLast = vm.runInContext(
    "typeof Array.prototype.findLast === 'function' ? Array.prototype.findLast : null",
    sandbox,
  );
  runCompat(sandbox);
  const after = vm.runInContext("Array.prototype.findLast", sandbox);
  if (nativeFindLast) {
    check("native findLast preserved", after === nativeFindLast);
  } else {
    check("findLast polyfilled when absent", typeof after === "function");
  }
}

// ---- 7. Conservative syntax ----------------------------------------------------
console.log("compat: conservative syntax");
{
  check("no optional chaining", JS.indexOf("?.") === -1);
  check("no nullish coalescing", JS.indexOf("??") === -1);
}

if (failures) {
  console.error(`\ntest-compat: ${failures} failure(s)`);
  process.exit(1);
}
console.log("\ntest-compat: all green");
