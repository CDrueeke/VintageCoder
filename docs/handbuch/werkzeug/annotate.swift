// Beschriftet Rohaufnahmen nach bilder/manifest.json: swift annotate.swift <handbuch-ordner> [id ...]
// Marken: {"typ":"rahmen","x":..,"y":..,"w":..,"h":..,"nr":1} (Koordinaten in Pixeln der Rohaufnahme, Ursprung oben links) – Rahmen plus nummerierter Kreis;
// {"typ":"pfeil","x1":..,"y1":..,"x2":..,"y2":..,"nr":2}. Signalfarbe Orange #FF9F1C, 3 px. Ausgabe: bilder/<id>.png, Breite 1600 px.
import AppKit
let a = CommandLine.arguments
let base = URL(fileURLWithPath: a[1])
let only = Set(a.dropFirst(2))
// Rohbilder: Standard bilder/roh/, abweichend über die Umgebungsvariable HANDBUCH_ROH
let rohDir = ProcessInfo.processInfo.environment["HANDBUCH_ROH"].map { URL(fileURLWithPath: $0) } ?? base.appendingPathComponent("bilder/roh")
let man = try! JSONSerialization.jsonObject(with: Data(contentsOf: base.appendingPathComponent("bilder/manifest.json"))) as! [String: Any]
let orange = NSColor(red: 1, green: 0.624, blue: 0.11, alpha: 1)
for e in man["bilder"] as! [[String: Any]] {
  let id = e["id"] as! String
  if !only.isEmpty && !only.contains(id) { continue }
  guard let src = NSBitmapImageRep(data: try! Data(contentsOf: rohDir.appendingPathComponent("\(id).png"))) else { continue }
  let W = src.pixelsWide, H = src.pixelsHigh
  let s = 1600.0 / Double(W), ow = 1600, oh = Int((Double(H) * s).rounded())
  let out = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: ow, pixelsHigh: oh, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
  NSGraphicsContext.saveGraphicsState()
  NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: out)
  NSGraphicsContext.current!.imageInterpolation = .high
  src.draw(in: NSRect(x: 0, y: 0, width: ow, height: oh))
  func P(_ x: Double, _ y: Double) -> NSPoint { NSPoint(x: x * s, y: Double(oh) - y * s) }
  func badge(_ n: Int, at p: NSPoint) {
    let r = 15.0
    orange.setFill(); NSBezierPath(ovalIn: NSRect(x: p.x - r, y: p.y - r, width: 2 * r, height: 2 * r)).fill()
    let t = NSAttributedString(string: "\(n)", attributes: [.font: NSFont.boldSystemFont(ofSize: 19), .foregroundColor: NSColor(white: 0.08, alpha: 1)])
    let sz = t.size(); t.draw(at: NSPoint(x: p.x - sz.width / 2, y: p.y - sz.height / 2))
  }
  for m in (e["marken"] as? [[String: Any]] ?? []) {
    func d(_ k: String) -> Double { (m[k] as? NSNumber)?.doubleValue ?? 0 }
    let n = (m["nr"] as? Int) ?? 0
    orange.setStroke()
    if (m["typ"] as? String) == "pfeil" {
      let p1 = P(d("x1"), d("y1")), p2 = P(d("x2"), d("y2"))
      let b = NSBezierPath(); b.lineWidth = 3; b.move(to: p1); b.line(to: p2); b.stroke()
      let ang = atan2(p2.y - p1.y, p2.x - p1.x), hl = 16.0
      let h = NSBezierPath(); h.lineWidth = 3
      h.move(to: p2); h.line(to: NSPoint(x: p2.x - hl * cos(ang - 0.45), y: p2.y - hl * sin(ang - 0.45)))
      h.move(to: p2); h.line(to: NSPoint(x: p2.x - hl * cos(ang + 0.45), y: p2.y - hl * sin(ang + 0.45))); h.stroke()
      if n > 0 { badge(n, at: p1) }
    } else {
      let o = P(d("x"), d("y")); let r = NSRect(x: o.x, y: o.y - d("h") * s, width: d("w") * s, height: d("h") * s)
      let b = NSBezierPath(roundedRect: r, xRadius: 6, yRadius: 6); b.lineWidth = 3; b.stroke()
      if n > 0 { badge(n, at: NSPoint(x: r.minX, y: r.maxY)) }
    }
  }
  NSGraphicsContext.restoreGraphicsState()
  try! out.representation(using: .png, properties: [:])!.write(to: base.appendingPathComponent("bilder/\(id).png"))
  print("\(id).png")
}
