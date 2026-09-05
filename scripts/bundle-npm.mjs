// Vendor a working npm CLI into src-tauri/resources/npm. A raw extracted
// registry tarball of npm is NOT runnable standalone (bundled deps are laid
// out for the installer to flatten), so we install npm with the build
// machine's npm (arborist) and ship node_modules/npm as the vendored root.
// NOTE: the build sandbox no-ops whole-tree cp/rename, so the final copy
// is a manual per-file write.
//
// Supply-chain policy (SN3): the npm version is PINNED together with the
// registry integrity hash below; the tarball bytes are verified BEFORE
// extraction and the build FAILS CLOSED on mismatch. Bumping the version
// REQUIRES updating both constants from the packument
// (https://registry.npmjs.org/npm/<version> -> dist.integrity).
// DSH_NPM_VERSION override is verified against that version's packument
// instead (transport trust only — prefer pinning).
import { createWriteStream } from "node:fs";
import { mkdir, readFile, writeFile, rm, access, chmod, readdir } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { gunzipSync } from "node:zlib";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PINNED_NPM_VERSION = "12.0.2";
const PINNED_NPM_INTEGRITY = "sha512-uIXokLlBj6FpNUTQX1PmT5pz7BlIN9QlixX+zdaSNHsd0qUXsbDLr50xzY6Sw7cJVr0uzHKDOle0swmPW/p5Qw==";
const NPM_VERSION = process.env.DSH_NPM_VERSION ?? PINNED_NPM_VERSION;
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const destDir = path.join(root, "src-tauri", "resources", "npm");
const marker = path.join(destDir, ".version");

async function exists(p) { try { await access(p); return true; } catch { return false; } }

const version = NPM_VERSION;
console.log("[bundle-npm] vendoring npm@" + version);

/// Expected tarball integrity: the repo pin, or the version's packument
/// dist.integrity for overrides (fail closed when unreachable).
async function expectedIntegrity(ver) {
  if (ver === PINNED_NPM_VERSION) return PINNED_NPM_INTEGRITY;
  console.log("[bundle-npm] non-pinned version", ver, "— verifying against packument integrity");
  const res = await fetch("https://registry.npmjs.org/npm/" + encodeURIComponent(ver));
  if (!res.ok) throw new Error("cannot fetch packument for npm@" + ver + ": " + res.status);
  const integrity = (await res.json())?.dist?.integrity;
  if (typeof integrity !== "string" || !integrity.startsWith("sha512-")) {
    throw new Error("no sha512 integrity in packument for npm@" + ver);
  }
  return integrity;
}

function verifyIntegrity(buf, integrity) {
  const m = /^(sha512)-(.+)$/.exec(integrity);
  if (!m) throw new Error("unsupported integrity format: " + integrity);
  const got = createHash("sha512").update(buf).digest("base64");
  if (got !== m[2]) throw new Error("[bundle-npm] CHECKSUM MISMATCH for npm-" + version + ".tgz");
  console.log("[bundle-npm] checksum OK (sha512:" + got.slice(0, 16) + "…)");
}

if (!process.argv.includes("--force") && await exists(marker)) {
  const cur = (await readFile(marker, "utf8")).trim();
  if (cur === version) { console.log("[bundle-npm] vendored npm", version, "already present"); process.exit(0); }
  console.log("[bundle-npm] version changed, re-vendoring");
  await rm(destDir, { recursive: true, force: true });
}
await vendoring(version);

async function copyTree(src, dest, rel) {
  const entries = await readdir(src, { withFileTypes: true });
  await mkdir(dest, { recursive: true });
  for (const entry of entries) {
    const s = path.join(src, entry.name);
    const d = path.join(dest, entry.name);
    const rel2 = rel ? rel + "/" + entry.name : entry.name;
    if (entry.isDirectory()) {
      await copyTree(s, d, rel2);
    } else {
      await writeFile(d, await readFile(s));
      if (rel2.startsWith("bin/")) await chmod(d, 0o755);
    }
  }
}

async function vendoring(version) {
  const stage = path.join(root, "target", "npm-stage-" + version);
  await rm(stage, { recursive: true, force: true });
  await mkdir(stage, { recursive: true });

  const url = "https://registry.npmjs.org/npm/-/npm-" + version + ".tgz";
  console.log("[bundle-npm] fetching", url);
  const res = await fetch(url);
  if (!res.ok) throw new Error("npm tarball fetch failed: " + res.status);
  const tgz = path.join(stage, ".npm.tgz");
  await pipeline(Readable.fromWeb(res.body), createWriteStream(tgz));
  verifyIntegrity(await readFile(tgz), await expectedIntegrity(version));

  const unpack = path.join(stage, "unpack");
  await mkdir(unpack, { recursive: true });
  const pkgRoot = path.join(unpack, "package");
  const xtr = spawnSync("tar", ["-xzf", tgz, "-C", unpack], { stdio: ["ignore", "pipe", "pipe"] });
  if (xtr.status !== 0) {
    console.error("[bundle-npm] tar extraction failed:", (xtr.stderr || "").toString());
    throw new Error("tar extraction failed");
  }
  await rm(tgz, { force: true });

  const runNpm = (args) => {
    const r = spawnSync(process.platform === "win32" ? "npm.cmd" : "npm", args, { cwd: unpack, stdio: ["ignore", "pipe", "pipe"] });
    if (r.status !== 0) {
      console.error("[bundle-npm] npm step failed:", (r.stderr || "").toString(), (r.stdout || "").toString());
      return false;
    }
    return true;
  };
  const ok = runNpm([
    "install", "--prefix", path.join(stage, "installed"),
    "--no-audit", "--no-fund", "--loglevel=warn", "--ignore-scripts",
    pkgRoot
  ]);
  if (!ok) throw new Error("could not materialize a working npm tree (build machine needs npm)");
  const depCheck = spawnSync("npm", ["ls", "--prefix", path.join(stage, "installed"), "--depth=0", "npm-normalize-package-bin", "--json"], { cwd: unpack, stdio: ["ignore", "pipe", "pipe"] });
  console.log("[bundle-npm] flattened tree deps:", (depCheck.stdout || "").toString().slice(0, 120).replace(/\n/g, " "));

  const srcPkg = path.join(stage, "installed", "node_modules", "npm");
  await rm(destDir, { recursive: true, force: true });
  let copied = 0;
  await copyTree(srcPkg, destDir, "").then(() => {});
  for await (const _ of walk(srcPkg)) { copied++; }
  await rm(stage, { recursive: true, force: true });
  await writeFile(marker, version + "\n").catch((e) => console.warn("[bundle-npm] marker write skipped:", e.code));
  console.log("[bundle-npm] vendored npm " + version + " -> " + destDir);

  const nodeSidecar = path.join(root, "src-tauri", "binaries", "node-aarch64-apple-darwin");
  if (await exists(nodeSidecar)) {
    const check = spawnSync(nodeSidecar, [path.join(destDir, "bin", "npm-cli.js"), "--version"], { encoding: "utf8" });
    console.log("[bundle-npm] verified:", (check.stdout || "?").trim(), "(npm " + version + ")");
    if (check.status !== 0) throw new Error("vendored npm failed to run: " + (check.stderr || ""));
  }
}

async function* walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(p);
    else yield p;
  }
}
