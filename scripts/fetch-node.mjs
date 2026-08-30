// Fetch the Node.js runtime used as the Tauri sidecar binary.
import { createWriteStream } from "node:fs";
import { mkdir, readFile, access, rename, unlink, writeFile, chmod, stat } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { gunzipSync } from "node:zlib";
import path from "node:path";
import { fileURLToPath } from "node:url";

const NODE_VERSION = process.env.DSH_NODE_VERSION ?? "v24.20.0";
const TRIPLE = process.env.DSH_TARGET_TRIPLE ?? "aarch64-apple-darwin";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outDir = path.join(root, "src-tauri", "binaries");
const outFile = path.join(outDir, "node-" + TRIPLE);
const tarballStem = "node-" + NODE_VERSION + "-darwin-arm64";
const url = "https://nodejs.org/dist/" + NODE_VERSION + "/" + tarballStem + ".tar.gz";

async function exists(p) { try { await access(p); return true; } catch { return false; } }

async function download(url, dest) {
  console.log("[fetch-node] downloading", url);
  const res = await fetch(url);
  if (!res.ok) throw new Error("download failed: " + res.status + " " + res.statusText);
  await pipeline(Readable.fromWeb(res.body), createWriteStream(dest));
}

async function extractNode(tarball, destDir) {
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
    if (name === tarballStem + "/bin/node" && typeflag !== "5") {
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
if (await exists(outFile)) {
  console.log("[fetch-node] sidecar already present:", path.basename(outFile));
} else {
  const tgz = path.join(outDir, ".node.tgz");
  await download(url, tgz);
  const extracted = await extractNode(tgz, outDir);
  await rename(extracted, outFile);
  await unlink(tgz).catch(() => {});
  const info = await stat(outFile);
  console.log("[fetch-node] installed", outFile, (info.size / 1e6).toFixed(1), "MB");
}
