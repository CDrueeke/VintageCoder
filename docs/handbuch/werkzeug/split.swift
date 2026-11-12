import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

func load(_ p: String) -> CGImage {
    let src = CGImageSourceCreateWithURL(URL(fileURLWithPath: p) as CFURL, nil)!
    return CGImageSourceCreateImageAtIndex(src, 0, nil)!
}
let a = CommandLine.arguments
let dark = load(a[1]), light = load(a[2])
let w = dark.width, h = dark.height
precondition(light.width == w && light.height == h, "Bilder haben verschiedene Größe")
let topX = Double(w) * 0.62, botX = Double(w) * 0.38
let H = Double(h)
let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                    space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
ctx.interpolationQuality = .high
ctx.draw(dark, in: CGRect(x: 0, y: 0, width: w, height: h))
// helle Seite: rechts der Diagonale (Koordinaten oben links, in CoreGraphics y gespiegelt)
ctx.saveGState()
ctx.beginPath()
ctx.move(to: CGPoint(x: topX, y: H))
ctx.addLine(to: CGPoint(x: Double(w), y: H))
ctx.addLine(to: CGPoint(x: Double(w), y: 0))
ctx.addLine(to: CGPoint(x: botX, y: 0))
ctx.closePath()
ctx.clip()
ctx.draw(light, in: CGRect(x: 0, y: 0, width: w, height: h))
ctx.restoreGState()
// Trennlinie: dunkler Rand, darüber weiß, damit sie auf beiden Seiten sichtbar bleibt
for (width, gray, alpha) in [(9.0, 0.35, 0.55), (5.0, 1.0, 1.0)] {
    ctx.setLineCap(.butt)
    ctx.setLineWidth(width)
    ctx.setStrokeColor(CGColor(gray: gray, alpha: alpha))
    ctx.beginPath()
    ctx.move(to: CGPoint(x: topX, y: H))
    ctx.addLine(to: CGPoint(x: botX, y: 0))
    ctx.strokePath()
}
let out = ctx.makeImage()!
let dst = CGImageDestinationCreateWithURL(URL(fileURLWithPath: a[3]) as CFURL, UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dst, out, nil)
precondition(CGImageDestinationFinalize(dst))
print("ok \(w)x\(h)")
