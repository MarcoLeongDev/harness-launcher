// Contract: language switches push over events; polling is fallback.
// set_language must emit launcher://language, and both launcher-owned pages
// must repaint via applyLanguage on that event while keeping their poll.
// Run: node scripts/test-events.mjs
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const settingsJs = readFileSync(path.join(root, "src-tauri", "resources", "settings.js"), "utf8");
const stoppedJs = readFileSync(path.join(root, "src-tauri", "resources", "stopped.js"), "utf8");
const commandsRs = readFileSync(path.join(root, "src-tauri", "src", "commands.rs"), "utf8");
let failures = 0;
function check(name, cond, extra) {
  if (cond) console.log(`  PASS ${name}`);
  else {
    failures++;
    console.error(`  FAIL ${name} ${extra || ""}`);
  }
}
console.log("events: backend emits language");
check(
  "set_language emits launcher://language",
  commandsRs.includes('"launcher://language"') && commandsRs.includes("app.emit("),
);
check(
  "language payload carries the code",
  commandsRs.includes('"language": lang') || commandsRs.includes('{ "language": lang }'),
);
console.log("events: pages repaint on push");
for (const [name, src] of [
  ["settings.js", settingsJs],
  ["stopped.js", stoppedJs],
]) {
  check(`${name} listens for launcher://language`, src.includes('"launcher://language"'));
  const at = src.indexOf('"launcher://language"');
  const window_ = src.slice(at, at + 400);
  check(`${name} repaint calls applyLanguage`, window_.includes("applyLanguage"));
}
console.log("events: polling retained as fallback");
check("settings.js keeps status poll", settingsJs.includes("setInterval(refresh, 3000)"));
check("stopped.js keeps status poll", stoppedJs.includes("setInterval(refresh, 3000)"));
if (failures > 0) {
  console.error(`test-events: ${failures} failures`);
  process.exit(1);
}
console.log("test-events: all green");
