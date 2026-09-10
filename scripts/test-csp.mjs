// Static contract for the launcher CSP (no inline execution).
// Launcher-owned pages must load behavior/presentation from same-origin
// files: no inline script/style, no inline event handlers, and the
// protocol + Tauri policies must not contain unsafe-inline.
// Run: node scripts/test-csp.mjs
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const res = (...p) => path.join(root, "src-tauri", "resources", ...p);
const settingsHtml = readFileSync(res("settings.html"), "utf8");
const stoppedHtml = readFileSync(res("stopped.html"), "utf8");
const libRs = readFileSync(path.join(root, "src-tauri", "src", "lib.rs"), "utf8");
const tauriConf = JSON.parse(readFileSync(path.join(root, "src-tauri", "tauri.conf.json"), "utf8"));
let failures = 0;
function check(name, cond, extra) {
  if (cond) console.log(`  PASS ${name}`);
  else {
    failures++;
    console.error(`  FAIL ${name} ${extra || ""}`);
  }
}
const hasInlineScript = (h) => /<script(?![^>]*src=)[^>]*>/i.test(h);
console.log("csp: no inline execution in owned pages");
const pages = [
  ["settings.html", settingsHtml],
  ["stopped.html", stoppedHtml],
];
for (const [name, h] of pages) {
  check(`${name} has no inline script`, !hasInlineScript(h));
  check(`${name} has no style block`, !/<style[ \t\r\n>]/i.test(h));
  check(
    `${name} has no inline event handler`,
    !/ on(click|load|error|input|change|focus|mouseover)=/i.test(h),
  );
  check(`${name} has no javascript: URL`, h.toLowerCase().indexOf("javascript:") === -1);
}
console.log("csp: asset wiring");
check("settings.html links settings.css", settingsHtml.indexOf('href="/settings.css"') !== -1);
check("settings.html loads settings.js", settingsHtml.indexOf('src="/settings.js"') !== -1);
check("stopped.html links stopped.css", stoppedHtml.indexOf('href="/stopped.css"') !== -1);
check("stopped.html loads stopped.js", stoppedHtml.indexOf('src="/stopped.js"') !== -1);
const assets = ["settings.css", "settings.js", "stopped.css", "stopped.js"];
for (const f of assets) {
  const ok = existsSync(res(f)) && readFileSync(res(f), "utf8").trim().length > 0;
  check(`resource ${f} exists and is non-empty`, ok);
}
console.log("csp: protocol policy");
const cspMatch = libRs.match(/const DSH_UI_CSP: &str = "([^"]+)"/);
check("protocol CSP declared", !!cspMatch);
const proto = cspMatch ? cspMatch[1] : "";
check("protocol CSP has no unsafe-inline", proto.indexOf("unsafe-inline") === -1);
check("protocol CSP keeps same-origin scripts", proto.indexOf("script-src 'self'") !== -1);
check("protocol CSP keeps same-origin styles", proto.indexOf("style-src 'self'") !== -1);
check(
  "protocol CSP denies objects/framing",
  proto.indexOf("object-src 'none'") !== -1 && proto.indexOf("frame-ancestors 'none'") !== -1,
);
const routes = ["/settings.css", "/settings.js", "/stopped.css", "/stopped.js"];
for (const route of routes) {
  check(`protocol serves ${route}`, libRs.indexOf(`"${route}"`) !== -1);
}
console.log("csp: tauri config policy");
const confCsp = tauriConf?.app?.security?.csp;
check("tauri.conf csp is declared", typeof confCsp === "string" && confCsp.length > 0);
check(
  "tauri.conf csp has no unsafe-inline",
  typeof confCsp === "string" && confCsp.indexOf("unsafe-inline") === -1,
);
check("tauri.conf csp matches protocol policy", confCsp === proto);
if (failures > 0) {
  console.error(`test-csp: ${failures} failures`);
  process.exit(1);
}
console.log("test-csp: all green");
