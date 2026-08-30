// Deploy the built .app bundle to /Applications.
// Usage:
//   node scripts/deploy.mjs               # install release bundle (keeps any running instance)
//   node scripts/deploy.mjs --relaunch    # also quit the running instance and relaunch the new build
import { execFileSync } from "node:child_process";
import { existsSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bundle = path.join(root, "src-tauri", "target", "release", "bundle", "macos", "DeepSeek Harness Launcher.app");
const dest = "/Applications/DeepSeek Harness Launcher.app";
const relaunch = process.argv.includes("--relaunch");

if (!existsSync(bundle)) {
  console.error("Release bundle not found:", bundle);
  console.error("Run `npm run build` first (assets + tauri build).");
  process.exit(1);
}
if (!existsSync("/Applications")) {
  console.error("/Applications not found — this deploy script is macOS-only.");
  process.exit(1);
}

console.log("Deploying");
console.log("  from:", bundle);
console.log("  to:  ", dest);

// Menu-bar (agent) app: mark LSUIElement so macOS never shows a Dock icon,
// not even a brief flash while the process launches (the runtime also sets
// ActivationPolicy::Accessory). PlistBuddy is a macOS built-in.
function ensureAgent(bundlePath) {
  const plist = path.join(bundlePath, "Contents", "Info.plist");
  try {
    execFileSync("/usr/libexec/PlistBuddy", ["-c", "Set :LSUIElement true", plist], { stdio: "ignore" });
  } catch {
    execFileSync("/usr/libexec/PlistBuddy", ["-c", "Add :LSUIElement bool true", plist], { stdio: "ignore" });
  }
  const out = execFileSync("/usr/libexec/PlistBuddy", ["-c", "Print :LSUIElement", plist], { encoding: "utf8" });
  console.log("  LSUIElement:", out.trim());
}
ensureAgent(bundle);

// ditto merges over an existing bundle, which keeps any currently running
// instance's loaded binaries intact until the user relaunches the app.
execFileSync("ditto", [bundle, dest], { stdio: "inherit" });

const mainBin = path.join(dest, "Contents", "MacOS", "dsh-launcher");
if (!existsSync(mainBin)) {
  console.error("Deployed app is missing its main binary:", mainBin);
  process.exit(1);
}
const size = (statSync(mainBin).size / 1e6).toFixed(1);
console.log(`Installed DeepSeek Harness Launcher.app (main binary ${size} MB, sidecar node + vendored npm bundled).`);

if (relaunch) {
  console.log("Quitting the running instance…");
  try {
    execFileSync("osascript", ["-e", 'tell application "DeepSeek Harness Launcher" to quit'], { stdio: "ignore" });
  } catch {
    /* not running — fine */
  }
  // Wait for the old instance to release before opening the new one.
  const delay = (ms) => new Promise((r) => setTimeout(r, ms));
  await delay(1500);
  console.log("Launching the new build…");
  execFileSync("open", ["-a", "DeepSeek Harness Launcher"], { stdio: "inherit" });
} else {
  console.log("A running instance (if any) keeps running; relaunch it later to pick up this build.");
}
