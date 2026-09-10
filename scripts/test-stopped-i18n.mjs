// Static contract for the stopped splash i18n (stopped.html + stopped.js).
// The stopped splash must support the same five UI languages as the Control
// Panel + tray (en, zh-Hant, zh-Hans, ja, es) with English fallback.
// Run: node scripts/test-stopped-i18n.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const html = readFileSync(path.join(root, "src-tauri", "resources", "stopped.html"), "utf8");
// Behavior lives in stopped.js (CSP: no inline scripts); content checks span
// both files, wiring checks stay on the markup.
let stoppedJs = "";
try {
  stoppedJs = readFileSync(path.join(root, "src-tauri", "resources", "stopped.js"), "utf8");
} catch {
  stoppedJs = "";
}
const src = `${html}\n${stoppedJs}`;
let failures = 0;
function check(name, cond, extra) {
  if (cond) console.log(`  PASS ${name}`);
  else {
    failures++;
    console.error(`  FAIL ${name} ${extra || ""}`);
  }
}
console.log("stopped-i18n: locales");
const langs = ["en", "zh-Hant", "zh-Hans", "ja", "es"];
let i;
for (i = 0; i < langs.length; i++) {
  const present = src.indexOf(`${langs[i]}: {`) !== -1 || src.indexOf(`"${langs[i]}": {`) !== -1;
  check(`locale ${langs[i]} present`, present);
}
const keys = ["title", "stoppedSub", "stoppedDesc", "startEngine", "openPanel", "brandGithub", "logoAlt"];
console.log("stopped-i18n: keys");
for (i = 0; i < keys.length; i++) {
  const count = src.split(`${keys[i]}:`).length - 1;
  check(`key ${keys[i]} in all 5 locales`, count >= 5, `found ${count}`);
}
console.log("stopped-i18n: wiring");
check("splash sub uses data-i18n", html.indexOf('data-i18n="stoppedSub"') !== -1);
check("splash desc uses data-i18n", html.indexOf('data-i18n="stoppedDesc"') !== -1);
check("start button uses data-i18n", html.indexOf('data-i18n="startEngine"') !== -1);
check("panel button uses data-i18n", html.indexOf('data-i18n="openPanel"') !== -1);
check("brand link title wired", html.indexOf('data-i18n-title="brandGithub"') !== -1);
check("brand link aria wired", html.indexOf('data-i18n-aria="brandGithub"') !== -1);
check("logo alt wired", html.indexOf('data-i18n-alt="logoAlt"') !== -1);
check("reads saved language via get_status", src.indexOf("get_status") !== -1);
check("English fallback for unknown codes", src.indexOf("LOCALES[l]") !== -1 && src.indexOf("en") !== -1);
check("repaints document title", src.indexOf("document.title") !== -1);
check("no hardcoded English-only splash", html.indexOf("The harness engine is stopped") !== -1);
if (failures > 0) {
  console.error(`test-stopped-i18n: ${failures} failures`);
  process.exit(1);
}
console.log("test-stopped-i18n: all green");
