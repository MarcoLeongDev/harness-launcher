import { spawnSync } from "node:child_process";
import { mkdir, writeFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

async function exists(p) { try { await access(p); return true; } catch { return false; } }
function run(script, args) {
  const r = spawnSync(process.execPath, [path.join(root, "scripts", script), ...(args || [])], { stdio: "inherit" });
  if (r.status !== 0) { console.error("prepare:", script, "failed"); process.exit(r.status ?? 1); }
}
function npx(cliArgs) {
  const bin = path.join(root, "node_modules", ".bin", "tauri");
  if (process.platform === "win32") {
    return spawnSync("npx.cmd", cliArgs, { cwd: root, stdio: "inherit" });
  }
  if (exists(bin)) return spawnSync(bin, cliArgs, { stdio: "inherit" });
  return spawnSync("npx", cliArgs, { cwd: root, stdio: "inherit" });
}

run("gen-icons.mjs");
run("gen-symbols.mjs");
const iconRes = npx(["icon", "src-tauri/icons/icon.png"]);
if (iconRes.status !== 0) {
  console.warn("[prepare] tauri icon step failed (needs @tauri-apps/cli installed); bundle icons may be missing");
}
run("fetch-node.mjs");
run("bundle-npm.mjs");
await mkdir(path.join(root, "web-dist"), { recursive: true });
const html = '<!doctype html><html><head><meta charset="utf-8"/><title>Harness Launcher</title></head><body><p>Harness Launcher shell - the harness window is created at runtime.</p></body></html>';
await writeFile(path.join(root, "web-dist", "index.html"), html);
console.log("[prepare] all assets ready");
