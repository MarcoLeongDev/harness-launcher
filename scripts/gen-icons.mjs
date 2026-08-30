// Icon generation: app icon, menu-bar template glyph and brand logo, all
// derived from logo/DSH Launcher.png.
// Image math is delegated to macOS `sips` (decode/resize/encode) — a plain
// Node hand-rolled PNG decoder previously turned the logo into black
// transparent garbage. Node only parses a sips-produced BMP to build the
// monochrome menu-bar template glyph.
import { deflateSync } from "node:zlib";
import { execFileSync } from "node:child_process";
import { mkdir, writeFile, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const LOGO_PATH = path.join(root, "logo", "DSH Launcher.png");
const SIPS = "/usr/bin/sips";

function sips(args, opts = {}) {
  return execFileSync(SIPS, args, { stdio: ["ignore", "pipe", "ignore"], ...opts });
}

function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const t = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crc]);
}
function encodePng(width, height, rgba) {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0); ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; ihdr[9] = 6;
  const raw = Buffer.alloc((width * 4 + 1) * height);
  for (let y = 0; y < height; y++) {
    raw[y * (width * 4 + 1)] = 0;
    rgba.copy(raw, y * (width * 4 + 1) + 1, y * width * 4, (y + 1) * width * 4);
  }
  return Buffer.concat([sig, chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw)), chunk("IEND", Buffer.alloc(0))]);
}

/// Resize src PNG to a square of `size` pixels, writing an 8-bit PNG.
function resizePng(src, out, size) {
  // sips -z takes pixelHeight pixelWidth; input logo is square, output square.
  sips(["-z", String(size), String(size), "-s", "format", "png", src, "--out", out]);
}

/// Decode a plain 24/32-bit BMP (BITMAPINFOHEADER) produced by sips.
function decodeBmp(buf) {
  const off = buf.readUInt32LE(10);
  const w = buf.readInt32LE(18);
  const hRaw = buf.readInt32LE(22);
  const height = Math.abs(hRaw);
  const bpp = buf.readUInt16LE(28);
  const stride = (((w * bpp) / 8 + 3) >> 2) << 2;
  const rgba = Buffer.alloc(w * height * 4);
  for (let y = 0; y < height; y++) {
    const rowOff = off + (hRaw > 0 ? (height - 1 - y) * stride : y * stride);
    for (let x = 0; x < w; x++) {
      const i = rowOff + x * (bpp >> 3);
      const d = (y * w + x) * 4;
      rgba[d] = buf[i + 2]; rgba[d + 1] = buf[i + 1]; rgba[d + 2] = buf[i];
      rgba[d + 3] = bpp === 32 ? buf[i + 3] : 255;
    }
  }
  return { width: w, height, rgba };
}

/// Monochrome template glyph: the dark logo mark becomes black with alpha;
/// the light background becomes transparent. (macOS template images are black
/// + alpha and are tinted automatically by the menu bar.)
function glyphAlpha(rgba, w, h) {
  const out = Buffer.alloc(w * h);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4;
      const lum = 0.299 * rgba[i] + 0.587 * rgba[i + 1] + 0.114 * rgba[i + 2];
      // Background is near-white: treat anything lighter than 225 as
      // transparent so only the dark logo mark is silhouetted. Dark marks
      // (navy/teal, lum <~150) reach full alpha; soft gray shadows stay.
      const a = lum > 225 ? 0 : Math.max(0, Math.min(255, Math.round((255 - lum) * 3)));
      out[y * w + x] = a;
    }
  }
  return out;
}

/// Bounding box of visible mark pixels.
function bbox(alpha, w, h) {
  let minX = w, minY = h, maxX = -1, maxY = -1;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      if (alpha[y * w + x] > 8) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
  }
  return maxX < 0 ? { x: 0, y: 0, w, h } : { x: minX, y: minY, w: maxX - minX + 1, h: maxY - minY + 1 };
}

/// Box-average downscale of an alpha map into a target square with padding
/// around the (cropped) mark, preserving aspect ratio.
function fitAlpha(alpha, crop, srcSize, target) {
  const content = Math.max(crop.w, crop.h);
  const pad = 1 - 0.12; // ~12% padding around the mark
  const scale = (target * pad) / content;
  const outW = Math.max(1, Math.round(crop.w * scale));
  const outH = Math.max(1, Math.round(crop.h * scale));
  const res = Buffer.alloc(outW * outH);
  for (let y = 0; y < outH; y++) {
    for (let x = 0; x < outW; x++) {
      const sx0 = crop.x + Math.floor((x * crop.w) / outW);
      const sx1 = crop.x + Math.max(1, Math.floor(((x + 1) * crop.w) / outW));
      const sy0 = crop.y + Math.floor((y * crop.h) / outH);
      const sy1 = crop.y + Math.max(1, Math.floor(((y + 1) * crop.h) / outH));
      let sum = 0, n = 0;
      for (let yy = sy0; yy < sy1; yy++) {
        for (let xx = sx0; xx < sx1; xx++) {
          if (xx < srcSize && yy < srcSize) { sum += alpha[yy * srcSize + xx]; n++; }
        }
      }
      res[y * outW + x] = n ? Math.round(sum / n) : 0;
    }
  }
  // Center the scaled mark on a square canvas.
  const canvas = Buffer.alloc(target * target);
  const offX = Math.floor((target - outW) / 2);
  const offY = Math.floor((target - outH) / 2);
  for (let y = 0; y < outH; y++) {
    res.copy(canvas, (offY + y) * target + offX, y * outW, (y + 1) * outW);
  }
  return canvas;
}

/// Turn an alpha map into an RGBA buffer (black + alpha).
function alphaToRgba(a, size) {
  const out = Buffer.alloc(size * size * 4);
  for (let i = 0; i < size * size; i++) {
    out[i * 4] = 0; out[i * 4 + 1] = 0; out[i * 4 + 2] = 0; out[i * 4 + 3] = a[i];
  }
  return out;
}

const iconsDir = path.join(root, "src-tauri", "icons");
const trayDir = path.join(root, "src-tauri", "resources", "tray");
const brandDir = path.join(root, "src-tauri", "resources", "brand");
await mkdir(iconsDir, { recursive: true });
await mkdir(trayDir, { recursive: true });
await mkdir(brandDir, { recursive: true });

console.log("[gen-icons] logo:", LOGO_PATH);

// 1) App icon source for `tauri icon`: the full logo at 1024x1024.
const iconPng = path.join(iconsDir, "icon.png");
resizePng(LOGO_PATH, iconPng, 1024);
console.log("[gen-icons] wrote icons/icon.png (1024, full logo)");

// 2) Brand logo for the Control Panel header (retina-friendly 256px).
const brandPng = path.join(brandDir, "logo.png");
resizePng(LOGO_PATH, brandPng, 256);
console.log("[gen-icons] wrote resources/brand/logo.png (256, full logo)");

// 3) Menu-bar template glyph: silhouette of the logo mark.
const tmp = path.join(tmpdir(), `dsh-mark-${process.pid}.bmp`);
try {
  sips(["-z", "128", "128", "-s", "format", "bmp", LOGO_PATH, "--out", tmp]);
  const { width, height, rgba } = decodeBmp(await readFile(tmp));
  const alpha = glyphAlpha(rgba, width, height);
  const box = bbox(alpha, width, height);
  console.log(`[gen-icons] logo mark bbox: ${box.w}x${box.h} @ (${box.x},${box.y})`);
  const a64 = fitAlpha(alpha, box, width, 64);
  const a32 = fitAlpha(alpha, box, width, 32);
  await writeFile(path.join(trayDir, "tray-icon@2x.png"), encodePng(64, 64, alphaToRgba(a64, 64)));
  await writeFile(path.join(trayDir, "tray-icon.png"), encodePng(32, 32, alphaToRgba(a32, 32)));
  console.log("[gen-icons] wrote tray-icon.png (32) and tray-icon@2x.png (64), logo-mark template glyph");
} finally {
  await rm(tmp, { force: true });
}