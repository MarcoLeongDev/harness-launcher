import { deflateSync } from "node:zlib";
import { mkdir, writeFile, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const LOGO_PATH = path.join(root, "logo", "DSH Launcher.png");

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

// Decode an 8-bit, non-interlaced PNG (color types 0/2/3/6) to RGBA.
function decodePng(buf) {
  let pos = 8, w = 0, h = 0, bit = 0, col = 0, interlace = 0;
  const idat = [];
  let trns = null;
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos);
    const type = buf.toString("ascii", pos + 4, pos + 8);
    if (type === "IHDR") {
      w = buf.readUInt32BE(pos + 8); h = buf.readUInt32BE(pos + 12);
      bit = buf[pos + 16]; col = buf[pos + 17]; interlace = buf[pos + 20];
    } else if (type === "IDAT") {
      idat.push(buf.subarray(pos + 8, pos + 8 + len));
    } else if (type === "tRNS") {
      trns = buf.subarray(pos + 8, pos + 8 + len);
    }
    pos += 12 + len;
  }
  if (bit !== 8 || interlace !== 0) throw new Error("unsupported png: bit=" + bit + " interlace=" + interlace);
  const channels = col === 6 ? 4 : col === 2 ? 3 : col === 0 ? 1 : col === 3 ? 1 : 0;
  if (!channels) throw new Error("unsupported png color type " + col);
  const raw = deflateSync(Buffer.concat(idat));
  const stride = w * channels + 1;
  const out = Buffer.alloc(h * stride);
  for (let y = 0; y < h; y++) {
    const f = raw[y * stride];
    const row = out.subarray(y * stride + 1, (y + 1) * stride);
    const prev = y > 0 ? out.subarray((y - 1) * stride + 1, y * stride) : null;
    for (let x = 0; x < w * channels; x++) {
      let a = raw[y * stride + 1 + x];
      const l = x >= channels ? row[x - channels] : 0;
      const u = prev ? prev[x] : 0;
      const ul = prev && x >= channels ? prev[x - channels] : 0;
      if (f === 1) a += l;
      else if (f === 2) a += u;
      else if (f === 3) a += (l + u) >> 1;
      else if (f === 4) {
        const p = l + u - ul, pa = Math.abs(p - l), pb = Math.abs(p - u), pc = Math.abs(p - ul);
        a += (pa <= pb && pa <= pc) ? l : (pb <= pc ? u : ul);
      }
      row[x] = a & 0xff;
    }
  }
  const rgba = Buffer.alloc(w * h * 4);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const src = y * stride + 1 + x * channels;
      const dst = (y * w + x) * 4;
      if (col === 2) {
        const r = out[src], g = out[src + 1], b = out[src + 2];
        let a = 255;
        if (trns && trns.length >= 3 && r === trns[0] && g === trns[1] && b === trns[2]) a = 0;
        rgba[dst] = r; rgba[dst + 1] = g; rgba[dst + 2] = b; rgba[dst + 3] = a;
      } else if (col === 6) {
        rgba[dst] = out[src]; rgba[dst + 1] = out[src + 1]; rgba[dst + 2] = out[src + 2]; rgba[dst + 3] = out[src + 3];
      } else if (col === 0) {
        const v = out[src]; let a = 255;
        if (trns && trns.length >= 1 && v === trns[0]) a = 0;
        rgba[dst] = v; rgba[dst + 1] = v; rgba[dst + 2] = v; rgba[dst + 3] = a;
      }
    }
  }
  return { width: w, height: h, rgba };
}

function resizeBilinear(src, sw, sh, dw, dh) {
  const out = Buffer.alloc(dw * dh * 4);
  const xr = sw / dw, yr = sh / dh;
  for (let dy = 0; dy < dh; dy++) {
    const srcY = (dy + 0.5) * yr - 0.5;
    const y0 = Math.max(0, Math.floor(srcY));
    const y1 = Math.min(sh - 1, y0 + 1);
    const fy = srcY - y0;
    for (let dx = 0; dx < dw; dx++) {
      const srcX = (dx + 0.5) * xr - 0.5;
      const x0 = Math.max(0, Math.floor(srcX));
      const x1 = Math.min(sw - 1, x0 + 1);
      const fx = srcX - x0;
      for (let c = 0; c < 4; c++) {
        const i00 = (y0 * sw + x0) * 4 + c, i10 = (y0 * sw + x1) * 4 + c;
        const i01 = (y1 * sw + x0) * 4 + c, i11 = (y1 * sw + x1) * 4 + c;
        const top = src[i00] * (1 - fx) + src[i10] * fx;
        const bot = src[i01] * (1 - fx) + src[i11] * fx;
        out[(dy * dw + dx) * 4 + c] = Math.round(top * (1 - fy) + bot * fy);
      }
    }
  }
  return out;
}

function distToSegment(px, py, ax, ay, bx, by) {
  const abx = bx - ax, aby = by - ay;
  const len2 = abx * abx + aby * aby;
  let t = ((px - ax) * abx + (py - ay) * aby) / len2;
  t = Math.max(0, Math.min(1, t));
  const dx = px - (ax + t * abx), dy = py - (ay + t * aby);
  return Math.sqrt(dx * dx + dy * dy);
}

// Crisp black "D"-chevron for the macOS menu bar (template image).
function drawTrayIcon(size) {
  const buf = Buffer.alloc(size * size * 4);
  const lw = size * 0.16;
  const x0 = size * 0.28, y0 = size * 0.22, x1 = size * 0.72, y1 = size * 0.5;
  const x2 = size * 0.28, y2 = size * 0.78;
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const p = x + 0.5, q = y + 0.5;
    const d1 = distToSegment(p, q, x0, y0, x1, y1);
    const d2 = distToSegment(p, q, x1, y1, x2, y2);
    const a = Math.min(d1, d2) < lw ? 255 : 0;
    const i = (y * size + x) * 4;
    buf[i] = 0; buf[i + 1] = 0; buf[i + 2] = 0; buf[i + 3] = a;
  }
  return buf;
}

const iconsDir = path.join(root, "src-tauri", "icons");
const trayDir = path.join(root, "src-tauri", "resources", "tray");
const brandDir = path.join(root, "src-tauri", "resources", "brand");
await mkdir(iconsDir, { recursive: true });
await mkdir(trayDir, { recursive: true });
await mkdir(brandDir, { recursive: true });

console.log("[gen-icons] reading logo", LOGO_PATH);
const logoBuf = await readFile(LOGO_PATH);
const { width: lw, height: lh, rgba } = decodePng(logoBuf);
console.log("[gen-icons] logo", lw + "x" + lh);

// 1) App icon: the full logo resized to the 1024 square tauri icon source.
await writeFile(path.join(iconsDir, "icon.png"), encodePng(1024, 1024, resizeBilinear(rgba, lw, lh, 1024, 1024)));

// 2) Tray icons: the black D-chevron (guaranteed crisp at 16-18 px).
await writeFile(path.join(trayDir, "tray-icon.png"), encodePng(32, 32, drawTrayIcon(32)));
await writeFile(path.join(trayDir, "tray-icon@2x.png"), encodePng(64, 64, drawTrayIcon(64)));

// 3) Brand logo for the Control Panel header (retina-friendly 256 px).
await writeFile(path.join(brandDir, "logo.png"), encodePng(256, 256, resizeBilinear(rgba, lw, lh, 256, 256)));

console.log("[gen-icons] icon.png (1024), tray-icon.png (32), tray-icon@2x.png (64), brand/logo.png (256) written");
