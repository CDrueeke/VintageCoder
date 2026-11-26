# VintageCoder-Logo

Das offizielle Logo ist die Wortmarke **„VintageCoder.“** (Violett, Weiß und ein gelber Punkt) in JetBrains Mono Bold, die Kurzform **„VC.“** und das App-Icon.
Die Buchstaben stehen in den SVG-Dateien als **Pfade** (Umrisse der Schrift, SIL Open Font License 1.1); es wird keine Schrift eingebettet oder mitgeliefert.
Farben: Violett `#a99bff` · Weiß `#f1f1f4` · Gelb `#ffc83d` (der Punkt gehört zum Logo) · Hintergrund `#0d1117` (Verlauf im Icon `#171c26` → `#0d1117`).

Erzeugt wird alles von `tools/gen_logo.py` (SVG und der Baustein `ui/src/lib/components/Wortmarke.svelte`); die App-Icons in allen Größen erzeugt `tools/gen_icon.py` aus den PNG-Vorlagen hier.
Die Dateien nicht von Hand ändern.

| Datei | Verwendung |
|---|---|
| `vintagecoder-wortmarke.svg/.png` | „VintageCoder.“ auf dunklem Grund (Website, README, Präsentationen) |
| `vintagecoder-wortmarke-transparent.svg/.png` | wie oben ohne Hintergrund, nur auf dunklem Grund verwenden |
| `vintagecoder-wortmarke-hell.svg/.png` | für weißen Grund (Handbuch, Druck) |
| `vintagecoder-vc.svg` | Kurzform „VC.“ auf dunklem Grund |
| `vintagecoder-icon.svg/.png` | App-Icon „VC.“ (abgerundetes Quadrat, 1024 px) |
| `vintagecoder-icon-klein.svg/.png` | dasselbe mit größerer Schrift, wird für 16 und 32 px verwendet (bessere Lesbarkeit) |

## Neu erzeugen

```bash
python3 tools/gen_logo.py            # SVG und Svelte-Baustein
cd ui && for n in icon icon-klein; do node tools/svg2png.mjs ../branding/vintagecoder-$n.svg ../branding/vintagecoder-$n.png 1024; done
node tools/svg2png.mjs ../branding/vintagecoder-wortmarke.svg ../branding/vintagecoder-wortmarke.png 1389
node tools/svg2png.mjs ../branding/vintagecoder-wortmarke-transparent.svg ../branding/vintagecoder-wortmarke-transparent.png 1389
node tools/svg2png.mjs ../branding/vintagecoder-wortmarke-hell.svg ../branding/vintagecoder-wortmarke-hell.png 1389
cd .. && python3 tools/gen_icon.py   # macOS .icns, Windows .ico, Linux-PNG, Symbole der Anwendung
```

Schreibweise: **VintageCoder** (Groß-/Kleinschreibung, ein Wort). Der gelbe Punkt gehört zum Logo.

## Verwendung des Logos

Das Logo darf als Verweis auf VintageCoder (Referenz) verwendet werden, zum Beispiel neben einem Link oder in einer Quellenangabe, auch skaliert oder einfarbig. Es darf nicht anderweitig verändert, nicht als Zeichen eines anderen Produkts verwendet und nicht so eingesetzt werden, dass der Eindruck einer Zugehörigkeit oder Billigung entsteht.

Der Name „VintageCoder“ bleibt davon unberührt: Er darf nur wahrheitsgemäß als Herkunftsangabe genannt werden („gebaut mit VintageCoder“).
