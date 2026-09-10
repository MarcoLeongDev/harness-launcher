// Contract: locales/*.json is the single i18n source of truth.
// Every language shares one key set per section (tray/stopped/panel), the
// stopped + panel tables in stopped.js/settings.js match the JSON values
// exactly, and tray.rs carries no hardcoded translated strings.
// Run: node scripts/test-i18n-sync.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const res = (...p) => path.join(root, "src-tauri", "resources", ...p);
const LANGS = ["en", "zh-Hant", "zh-Hans", "ja", "es"];
const SECTIONS = ["tray", "stopped", "panel"];
let failures = 0;
function check(name, cond, extra) {
  if (cond) console.log(`  PASS ${name}`);
  else {
    failures++;
    console.error(`  FAIL ${name} ${extra || ""}`);
  }
}
function extractTable(file) {
  const src = readFileSync(file, "utf8");
  const marker = "const LOCALES = ";
  const lit = src.slice(src.indexOf(marker) + marker.length);
  let depth = 0,
    end = -1,
    inStr = false,
    strCh = "",
    esc = false;
  for (let i = 0; i < lit.length; i++) {
    const c = lit[i];
    if (inStr) {
      if (esc) esc = false;
      else if (c === "\\") esc = true;
      else if (c === strCh) inStr = false;
      continue;
    }
    if (c === '"' || c === "'") {
      inStr = true;
      strCh = c;
      continue;
    }
    if (c === "{") depth++;
    if (c === "}") {
      depth--;
      if (depth === 0) {
        end = i + 1;
        break;
      }
    }
  }
  return new Function(`return (${lit.slice(0, end)})`)();
}
const docs = {};
for (const lang of LANGS) docs[lang] = JSON.parse(readFileSync(res("locales", `${lang}.json`), "utf8"));
console.log("i18n-sync: shared key sets");
for (const section of SECTIONS) {
  const base = Object.keys(docs.en[section]).sort();
  check(`en ${section} is non-empty`, base.length > 0);
  for (const lang of LANGS) {
    const keys = docs[lang] ? Object.keys(docs[lang][section] || {}).sort() : [];
    check(
      `${lang} ${section} matches en key set`,
      JSON.stringify(keys) === JSON.stringify(base),
      `got [${keys}]`,
    );
    for (const k of base)
      check(
        `${lang} ${section}.${k} non-empty`,
        typeof docs[lang][section]?.[k] === "string" && docs[lang][section][k].length > 0,
      );
  }
}
console.log("i18n-sync: stopped.js matches JSON");
const stoppedJs = extractTable(res("stopped.js"));
for (const lang of LANGS) {
  check(
    `stopped.js ${lang} deep-equals JSON`,
    JSON.stringify(stoppedJs[lang]) === JSON.stringify(docs[lang].stopped),
  );
}
console.log("i18n-sync: settings.js matches JSON");
const panelJs = extractTable(res("settings.js"));
for (const lang of LANGS) {
  check(
    `settings.js ${lang} deep-equals JSON`,
    JSON.stringify(panelJs[lang]) === JSON.stringify(docs[lang].panel),
  );
}
console.log("i18n-sync: tray.rs has no hardcoded strings");
const trayRs = readFileSync(path.join(root, "src-tauri", "src", "tray.rs"), "utf8");
const samples = [
  docs["zh-Hant"].tray.open,
  docs.ja.tray.quit,
  docs.es.tray.restart,
  docs["zh-Hans"].tray.browser,
];
for (const s of samples) check(`tray.rs no longer hardcodes "${s}"`, !trayRs.includes(s));
check(
  "tray.rs reads locales via include_str",
  trayRs.includes("resources/locales/") || trayRs.includes("resources/locales"),
);
if (failures > 0) {
  console.error(`test-i18n-sync: ${failures} failures`);
  process.exit(1);
}
console.log("test-i18n-sync: all green");
