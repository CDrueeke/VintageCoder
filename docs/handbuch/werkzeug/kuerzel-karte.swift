// Rendert die Übersichtskarte der Tastenkürzel (kein Screenshot): swift kuerzel-karte.swift <ausgabe.png>
// Inhalt wie Tabelle in Kapitel 20 (kuerzel-faq.md); Cmd gilt auf dem Mac, Strg unter Windows und Linux.
import AppKit
let groups: [(String, [(String, String)])] = [
 ("Bauen und Starten", [("F7","Projekt bauen"),("Umschalt+F7","aktive Datei bauen"),("F5","bauen und starten"),("Umschalt+F5","aktive Datei starten"),("F6","Startprogramm debuggen"),("Umschalt+F6","aktive Datei debuggen")]),
 ("Debugger", [("F8","Weiter"),("F11","Schritt (JSR hinein)"),("F10","Über (JSR durch)"),("Umschalt+F11","Raus (bis Rücksprung)"),("Klick auf Zeilennummer","Haltepunkt setzen/löschen")]),
 ("Dateien", [("Cmd/Strg+Umschalt+N","Neues Projekt"),("Cmd/Strg+O","Projekt öffnen"),("Cmd/Strg+N","Neue Quelldatei"),("Cmd/Strg+S","Speichern"),("Cmd/Strg+Alt+S","Alle speichern"),("Cmd/Strg+W","Datei schließen"),("Cmd/Strg+,","Einstellungen")]),
 ("Bearbeiten und Suchen", [("Cmd/Strg+Z","Rückgängig"),("Cmd/Strg+Umschalt+Z","Wiederholen"),("Cmd/Strg+F","Suchen"),("Cmd/Strg+Alt+F","Ersetzen"),("Cmd/Strg+L","Gehe zu Zeile"),("Cmd/Strg+Umschalt+F","In Projekt suchen"),("Cmd/Strg+Umschalt+Y","Zeichen einfügen"),("Ctrl+Leertaste","Vervollständigung"),("F1","Hilfe zum Wort")]),
 ("Ansicht", [("Cmd/Strg+K","Befehlspalette"),("Cmd/Strg+B","Seitenleiste"),("Cmd/Strg+Alt+B","Inspektor"),("Cmd/Strg+J","Fußpanel"),("Cmd/Strg+Umschalt+Enter","Maximieren"),("Cmd/Strg+\\","Editor teilen")]),
]
let W = 1600, H = 1000
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: W, pixelsHigh: H, bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState(); NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
NSColor(red: 0.05, green: 0.06, blue: 0.10, alpha: 1).setFill(); NSRect(x: 0, y: 0, width: W, height: H).fill()
let accent = NSColor(red: 0.0, green: 0.74, blue: 1.0, alpha: 1)
func text(_ s: String, _ x: Double, _ y: Double, _ size: Double, _ color: NSColor, mono: Bool = false, bold: Bool = false) {
  let f = mono ? NSFont.monospacedSystemFont(ofSize: size, weight: bold ? .bold : .regular) : (bold ? NSFont.boldSystemFont(ofSize: size) : NSFont.systemFont(ofSize: size))
  NSAttributedString(string: s, attributes: [.font: f, .foregroundColor: color]).draw(at: NSPoint(x: x, y: Double(H) - y - size))
}
text("Tastenkürzel", 50, 36, 34, .white, bold: true)
text("Cmd = Mac · Strg = Windows und Linux", 50, 82, 18, NSColor(white: 0.6, alpha: 1))
let cols = [0, 1, 2]; let colW = 500.0
var y = [120.0, 120.0, 120.0]
let place = [0, 0, 1, 2, 2]
for (gi, g) in groups.enumerated() {
  let c = place[gi]; let x = 50 + Double(c) * (colW + 25)
  text(g.0, x, y[c], 22, accent, bold: true); y[c] += 38
  for (k, v) in g.1 {
    let r = NSRect(x: x, y: Double(H) - y[c] - 28, width: 235, height: 30)
    NSColor(white: 0.14, alpha: 1).setFill(); NSBezierPath(roundedRect: r, xRadius: 6, yRadius: 6).fill()
    text(k, x + 8, y[c] + 4, 14, .white, mono: true)
    text(v, x + 248, y[c] + 3, 17, NSColor(white: 0.85, alpha: 1)); y[c] += 38
  }
  y[c] += 24
}
NSGraphicsContext.restoreGraphicsState()
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
