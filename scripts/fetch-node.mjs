// Fetch the Node.js runtime used as the Tauri sidecar binary.
//
// Supply-chain policy (SN3): the tarball is verified by SHA256 before
// extraction and the build FAILS CLOSED on mismatch. The default version is
// pinned together with its checksum below; bumping the version REQUIRES
// updating PINNED_NODE_SHA256 from https://nodejs.org/dist/vX.Y.Z/SHASUMS256.txt.
// An env override (DSH_NODE_VERSION) is verified against that version's
// published SHASUMS256.txt instead (transport trust only — prefer pinning).
import { createWriteStream } from "node:fs";
import { mkdir, readFile, access, rename, unlink, writeFile, chmod, stat } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { gunzipSync } from "node:zlib";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PINNED_NODE_VERSION = "v24.20.0";
const PINNED_NODE_SHA256 = {
  "aarch64-apple-darwin": "40e5607e5ecb3db9192723776da2d75d966260fc74a7a9e731c1bd67dda96bc8",
  "x86_64-apple-darwin": "9e5b2644cf107befb6aefca676b96d3296bc10138096f022ed378d6233ed81f4",
};
// Nodejs dist asset arch per Rust target triple (macOS universal = both).
function assetArch(triple) {
  if (triple === "aarch64-apple-darwin") return "arm64";
  if (triple === "x86_64-apple-darwin") return "x64";
  throw new Error("unsupported node sidecar triple: " + triple);
}
// Triples to materialize: explicit DSH_TARGET_TRIPLE, else both macOS arches
// so `tauri build --target universal-apple-darwin` always finds its sidecars.
const TRIPLES = process.env.DSH_TARGET_TRIPLE
  ? [process.env.DSH_TARGET_TRIPLE]
  : ["aarch64-apple-darwin", "x86_64-apple-darwin"];
const NODE_VERSION = process.env.DSH_NODE_VERSION ?? PINNED_NODE_VERSION;
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outDir = path.join(root, "src-tauri", "binaries");

async function exists(p) { try { await access(p); return true; } catch { return false; } }

async function download(url, dest) {
  console.log("[fetch-node] downloading", url);
  const res = await fetch(url);
  if (!res.ok) throw new Error("download failed: " + res.status + " " + res.statusText);
  await pipeline(Readable.fromWeb(res.body), createWriteStream(dest));
}

function sha256Hex(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

/// Expected tarball checksum: the repo pin for the pinned version + known
/// triple, otherwise the version's published SHASUMS256.txt (fail closed).
async function expectedSha256(version, triple, stem) {
  if (version === PINNED_NODE_VERSION && PINNED_NODE_SHA256[triple]) {
    return PINNED_NODE_SHA256[triple];
  }
  console.log("[fetch-node] non-pinned version", version, "— verifying against published SHASUMS256.txt");
  const res = await fetch("https://nodejs.org/dist/" + version + "/SHASUMS256.txt");
  if (!res.ok) throw new Error("cannot fetch SHASUMS256.txt for " + version + ": " + res.status);
  const text = await res.text();
  for (const line of text.split("\n")) {
    const [hash, name] = line.trim().split(/\s+/);
    if (name === stem + ".tar.gz" && /^[0-9a-f]{64}$/.test(hash || "")) return hash;
  }
  throw new Error("no checksum entry for " + stem + ".tar.gz in SHASUMS256.txt");
}

async function extractNode(tarball, destDir, stem) {
  const buf = await readFile(tarball);
  const body = gunzipSync(buf);
  let off = 0;
  let written = false;
  while (off + 512 <= body.length) {
    const header = body.subarray(off, off + 512);
    const name = header.subarray(0, 100).toString("utf8").replace(/\0.*$/, "");
    if (name === "") break;
    const size = parseInt(header.subarray(124, 136).toString("utf8").trim().replace(/\0.*$/, ""), 8);
    const typeflag = String.fromCharCode(header[156]);
    const dataStart = off + 512;
    if (name === stem + "/bin/node" && typeflag !== "5") {
      const out = path.join(destDir, "node");
      await writeFile(out, body.subarray(dataStart, dataStart + size));
      await chmod(out, 0o755);
      written = true;
      console.log("[fetch-node] extracted", name, size, "bytes");
      break;
    }
    off = dataStart + Math.ceil(size / 512) * 512;
  }
  if (!written) throw new Error("node binary not found in tarball");
  return path.join(destDir, "node");
}

await mkdir(outDir, { recursive: true });
for (const triple of TRIPLES) {
  const stem = "node-" + NODE_VERSION + "-darwin-" + assetArch(triple);
  const url = "https://nodejs.org/dist/" + NODE_VERSION + "/" + stem + ".tar.gz";
  const outFile = path.join(outDir, "node-" + triple);
  const pinMarker = path.join(outDir, ".node.pinned." + triple);
  const pinWant = NODE_VERSION + " " + (await expectedSha256(NODE_VERSION, triple, stem)) + "\n";
  if ((await exists(outFile)) && (await exists(pinMarker)) && (await readFile(pinMarker, "utf8")) === pinWant) {
    console.log("[fetch-node] verified sidecar already present:", path.basename(outFile));
    continue;
  }
  const tgz = path.join(outDir, ".node-" + triple + ".tgz");
  await download(url, tgz);
  const sum = sha256Hex(await readFile(tgz));
  // Re-derived on every run — the marker only gates the skip path above,
  // never the comparison.
  const authoritative = await expectedSha256(NODE_VERSION, triple, stem);
  if (sum !== authoritative) {
    await unlink(tgz).catch(() => {});
    throw new Error("[fetch-node] CHECKSUM MISMATCH for " + stem + ".tar.gz: got " + sum + ", want " + authoritative);
  }
  console.log("[fetch-node] checksum OK", triple, "(sha256:" + sum.slice(0, 16) + "…)");
  const extracted = await extractNode(tgz, outDir, stem);
  await rename(extracted, outFile);
  await unlink(tgz).catch(() => {});
  await writeFile(pinMarker, pinWant);
  const info = await stat(outFile);
  console.log("[fetch-node] installed", outFile, (info.size / 1e6).toFixed(1), "MB");
}

// macOS universal builds (`tauri build --target universal-apple-darwin`) expect
// ONE sidecar named node-universal-apple-darwin: lipo the verified arch slices.
const UNIVERSAL_TRIPLE = "universal-apple-darwin";
const UNIVERSAL_SLICES = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
if (UNIVERSAL_SLICES.every((t) => TRIPLES.includes(t))) {
  const slices = UNIVERSAL_SLICES.map((t) => path.join(outDir, "node-" + t));
  const universal = path.join(outDir, "node-" + UNIVERSAL_TRIPLE);
  const uniMarker = path.join(outDir, ".node.pinned." + UNIVERSAL_TRIPLE);
  const uniWant = NODE_VERSION + " " + UNIVERSAL_SLICES.map((t) => PINNED_NODE_SHA256[t] ?? "?").join("+") + "\n";
  const uniCurrent = (await exists(universal)) && (await exists(uniMarker))
    ? await readFile(uniMarker, "utf8").catch(() => "")
    : "";
  if (uniCurrent === uniWant) {
    console.log("[fetch-node] universal sidecar already present:", path.basename(universal));
  } else {
    execFileSync("lipo", ["-create", ...slices, "-output", universal]);
    await chmod(universal, 0o755);
    await writeFile(uniMarker, uniWant);
    const info = await stat(universal);
    console.log("[fetch-node] universal sidecar", universal, (info.size / 1e6).toFixed(1), "MB");
  }
}
