// Render SF Symbols to black+alpha PNGs for the tray menu items.
// Usage: swift render-symbols.swift <manifest.json>
//   manifest: [{"symbol": "<SF Symbol name>", "out": "<abs .png path>"}, ...]
// Rendered at 36x36 (18pt @2x, the size muda sets on menu item icons).
// The glyph is forced to pure black + original alpha so it tints cleanly
// as a template image. Fails loudly (non-zero exit) if a symbol name is
// unknown or renders empty — never ship a blank icon.
import AppKit
import Foundation

struct Entry: Decodable {
    let symbol: String
    let out: String
}

guard CommandLine.arguments.count > 1 else {
    FileHandle.standardError.write(Data("usage: render-symbols.swift <manifest.json>\n".utf8))
    exit(2)
}

let data = try Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1]))
let entries: [Entry]
do { entries = try JSONDecoder().decode([Entry].self, from: data) }
catch {
    FileHandle.standardError.write(Data("bad manifest: \(error)\n".utf8))
    exit(2)
}

let px = 36
var failed = false
for entry in entries {
    guard let img = NSImage(systemSymbolName: entry.symbol, accessibilityDescription: entry.symbol) else {
        FileHandle.standardError.write(Data("SYMBOL NOT FOUND: \(entry.symbol)\n".utf8))
        failed = true
        continue
    }
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: px, pixelsHigh: px,
        bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
        colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    img.draw(in: NSRect(x: 0, y: 0, width: px, height: px), from: .zero,
             operation: .sourceOver, fraction: 1.0)
    NSGraphicsContext.restoreGraphicsState()

    var ink = 0
    for y in 0..<px {
        for x in 0..<px {
            if let c = rep.colorAt(x: x, y: y), c.alphaComponent > 0.02 {
                rep.setColor(NSColor(calibratedWhite: 0.0, alpha: c.alphaComponent), atX: x, y: y)
                ink += 1
            }
        }
    }
    let pct = Double(ink) / Double(px * px) * 100.0
    if pct < 1.0 {
        FileHandle.standardError.write(Data("SYMBOL RENDERED EMPTY: \(entry.symbol) (ink \(pct)%)\n".utf8))
        failed = true
        continue
    }
    let png = rep.representation(using: .png, properties: [:])!
    try png.write(to: URL(fileURLWithPath: entry.out))
    print("[gen-symbols] \(entry.symbol) -> \(entry.out) (ink \(String(format: "%.1f", pct))%)")
}

exit(failed ? 3 : 0)
