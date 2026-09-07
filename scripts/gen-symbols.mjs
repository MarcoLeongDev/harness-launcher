// Generate tray menu item glyphs from SF Symbols (verbatim names).
// Output: src-tauri/resources/tray/symbol/<item-id>.png, embedded into the
// binary by tray.rs (include_bytes!) and rendered as template images by the
// locally patched muda (see src-tauri/vendor/muda + [patch.crates-io]).
// Run automatically by scripts/prepare.mjs (dev and build).
import { execFileSync } from "node:child_process";
import { mkdir, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outDir = path.join(root, "src-tauri", "resources", "tray", "symbol");
const swift = path.join(root, "scripts", "render-symbols.swift");

// Menu item id -> SF Symbol name (mapping specified by the user; verbatim).
const SYMBOLS = [
  ["open", "house"],
  ["settings", "gearshape"],
  ["start", "play"],
  ["stop", "stop"],
  ["restart", "arrow.counterclockwise"],
  ["browser", "arrow.up.right.square"],
  ["quit", "power"],
];

await rm(outDir, { recursive: true, force: true });
await mkdir(outDir, { recursive: true });
const manifest = SYMBOLS.map(([item, symbol]) => ({ symbol, out: path.join(outDir, item + ".png") }));
const tmp = path.join(os.tmpdir(), "dsh-symbols-" + process.pid + ".json");
await writeFile(tmp, JSON.stringify(manifest));
try {
  execFileSync("/usr/bin/swift", [swift, tmp], { stdio: "inherit" });
} finally {
  await rm(tmp, { force: true });
}
for (const m of manifest) {
  const s = await stat(m.out); // throws (and fails the build) if a PNG is missing
  if (s.size < 100) {
    console.error("[gen-symbols] suspiciously small PNG:", m.out);
    process.exit(5);
  }
}
console.log("[gen-symbols] wrote " + manifest.length + " symbol PNGs to " + outDir);
