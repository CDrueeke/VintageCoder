#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Baut aus Handbuch (docs/handbuch) und IDE-Hilfe (crates/vc-ide/help) das IDE-Handbuch und die VCASM-Dokumentation
als PDF und als Markdown.

Aufruf:  python3 tools/dokumentation_pdf.py <Repo-Ordner> <Ausgabeordner> [--lang de|en]
         (--lang en: englische Kapitel aus docs/handbuch/en und crates/vc-ide/help/en; Ergebnis VintageCoder-Handbuch-en.pdf,
          VCASM-Dokumentation-en.pdf, markdown-handbuch-en/, markdown-vcasm-en/; die Bilder sind vorerst dieselben (deutsche Oberfläche))
Braucht Python 3 mit dem Modul `markdown` (pip install markdown) und einen Chromium-Browser fuer den PDF-Druck
(Standard: Microsoft Edge auf macOS; anderer Browser ueber die Umgebungsvariable VC_PDF_BROWSER).
Ergebnis: VintageCoder-Handbuch.pdf, VCASM-Dokumentation.pdf, markdown-handbuch/, markdown-vcasm/ (plus die HTML-Zwischendateien)."""
import html, os, re, subprocess, sys, base64, datetime
import markdown

LANG = "de"
_args, _it = [], iter(sys.argv[1:])
for _a in _it:
    if _a == "--lang": LANG = next(_it, "de")
    elif _a.startswith("--lang="): LANG = _a.split("=", 1)[1]
    else: _args.append(_a)
if LANG not in ("de", "en"): sys.exit("--lang: de oder en")
if len(_args) != 2: sys.exit(__doc__)
REPO = _args[0]
OUT = _args[1]
SUF = "" if LANG == "de" else "-en"
# Texte je Sprache (Deckblatt, Kopfzeilen, Rechtstext); Deutsch ist Quelle
TXT = {
  "de": {"toc": "Inhalt", "glossary": "Glossar", "tip": "Tipp", "caution": "Achtung", "date": "%d.%m.%Y",
         "hb_title": "VintageCoder – Handbuch", "hb_sub": "Handbuch der Entwicklungsumgebung", "hb_line": "Von der Ersteinrichtung bis zum Debugger",
         "asm_title": "VCASM – Dokumentation", "asm_sub": "Der Assembler von VintageCoder", "asm_line": "Einführung und Referenz",
         "version": "Version", "stand": "Stand", "hinweis_b": "Hinweis.",
         "hinweis": "VintageCoder enthält wahrscheinlich noch viele Fehler. Die Dokumentation kann dem Programm vorauseilen oder hinterherhinken.",
         "legal_h": "Lizenz und Haftung",
         "legal": "VintageCoder steht unter der GNU General Public License, Version 3 oder später. Was du mit VintageCoder erzeugst, gehört dir und ist frei nutzbar; die Sprache VCASM und die Dateiformate sind frei; Systemdateien, Vorlagen und Beispiele stehen unter MIT-0 (Einzelheiten: LICENSING.md). Die Software wird ohne jede Gewährleistung bereitgestellt, insbesondere ohne Haftung für Datenverlust oder Schäden an Daten und Hardware. Herkunft und Lizenzen enthaltener Fremdinhalte: THIRD-PARTY-NOTICES.md im Quelltext.",
         "intro": "Einführung", "ref": "Referenz", "asm_first": "Assembler (VCASM) für Einsteiger",
         "hb_md": "Handbuch der Entwicklungsumgebung. Version {v}, Stand {d}. Die Software enthält wahrscheinlich noch viele Fehler.",
         "asm_md": "Einführung und Referenz des Assemblers von VintageCoder. Version {v}, Stand {d}.",
         "asm_pages": ["Assembler: Aufbau, Adressierungsarten, undokumentierte Befehle", "Ausdrücke und Funktionen", "Direktiven", "Texte und Zeichensätze",
                       "Makros, Bedingungen und Schleifen", "Systembibliotheken", "Steckmodule (CRT, CAR)", "Spickzettel und Fehlercodes"]},
  "en": {"toc": "Contents", "glossary": "Glossary", "tip": "Tip", "caution": "Caution", "date": "%Y-%m-%d",
         "hb_title": "VintageCoder - Manual", "hb_sub": "Manual of the development environment", "hb_line": "From first-run setup to the debugger",
         "asm_title": "VCASM - Documentation", "asm_sub": "The assembler of VintageCoder", "asm_line": "Introduction and reference",
         "version": "Version", "stand": "Date", "hinweis_b": "Note.",
         "hinweis": "VintageCoder probably still contains many errors. The documentation may be ahead of the program or behind it.",
         "legal_h": "License and liability",
         "legal": "VintageCoder is released under the GNU General Public License, version 3 or later. What you create with VintageCoder belongs to you and is free to use; the VCASM language and the file formats are free; system files, templates and examples are under MIT-0 (details: LICENSING.md). The software is provided without any warranty, in particular without liability for loss of data or damage to data and hardware. Origin and licenses of included third-party content: THIRD-PARTY-NOTICES.md in the source code. The German text is authoritative.",
         "intro": "Introduction", "ref": "Reference", "asm_first": "Assembler (VCASM) for beginners",
         "hb_md": "Manual of the development environment. Version {v}, date {d}. The software probably still contains many errors.",
         "asm_md": "Introduction and reference of the VintageCoder assembler. Version {v}, date {d}.",
         "asm_pages": ["Assembler: structure, addressing modes, undocumented instructions", "Expressions and functions", "Directives", "Text and character sets",
                       "Macros, conditions and loops", "System libraries", "Cartridges (CRT, CAR)", "Cheat sheet and error codes"]},
}[LANG]
EDGE = os.environ.get("VC_PDF_BROWSER", "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge")
HB = os.path.join(REPO, "docs/handbuch")
HELP = os.path.join(REPO, "crates/vc-ide/help") if LANG == "de" else os.path.join(REPO, "crates/vc-ide/help/en")
HBL = HB if LANG == "de" else os.path.join(HB, "en")  # Kapitel und Glossar der Sprache (Bilder liegen immer in HB/bilder)
VERSION = re.search(r'^version = "([^"]+)"', open(os.path.join(REPO, "Cargo.toml")).read(), re.M).group(1)
STAND = datetime.date.today().strftime(TXT["date"])

CSS = """
@page { size: A4; margin: 22mm 18mm 22mm 18mm;
  @bottom-center { content: counter(page); font: 9pt 'Helvetica Neue', Arial, sans-serif; color: #667; }
  @bottom-left { content: string(doctitle); font: 9pt 'Helvetica Neue', Arial, sans-serif; color: #889; }
}
@page :first { @bottom-center { content: none; } @bottom-left { content: none; } margin: 0; }
html { font: 10.2pt/1.5 'Helvetica Neue', Arial, sans-serif; color: #1b2230; }
body { margin: 0; }
.doctitle { string-set: doctitle content(); display: none; }
h1, h2, h3, h4 { font-family: 'Helvetica Neue', Arial, sans-serif; color: #0b3a66; line-height: 1.25; }
h1 { font-size: 22pt; margin: 0 0 6mm; padding-bottom: 2mm; border-bottom: 2px solid #0a63b5; }
h2 { font-size: 15pt; margin: 8mm 0 3mm; break-after: avoid; }
h3 { font-size: 12pt; margin: 6mm 0 2mm; break-after: avoid; }
h4 { font-size: 10.5pt; margin: 4mm 0 1mm; break-after: avoid; }
section.kap { break-before: page; }
p { margin: 0 0 3mm; orphans: 3; widows: 3; }
code { font: 9pt Menlo, 'SF Mono', Consolas, monospace; background: #eef2f8; padding: 0 2px; border-radius: 2px; }
pre { font: 8.4pt/1.4 Menlo, 'SF Mono', Consolas, monospace; background: #f3f5f9; border: 1px solid #d5dbe6; border-left: 3px solid #0a63b5;
  padding: 2.5mm 3mm; margin: 0 0 4mm; white-space: pre-wrap; word-break: break-word; break-inside: avoid; }
pre code { background: none; padding: 0; font: inherit; }
table { border-collapse: collapse; width: 100%; margin: 0 0 4mm; font-size: 9pt; break-inside: auto; }
th, td { border: 1px solid #ccd3e0; padding: 1.4mm 2mm; text-align: left; vertical-align: top; }
th { background: #e6eef8; color: #0b3a66; }
tr { break-inside: avoid; }
blockquote { margin: 0 0 4mm; padding: 2mm 4mm; background: #eef5fd; border-left: 4px solid #0a63b5; break-inside: avoid; }
blockquote p:last-child { margin-bottom: 0; }
blockquote.tipp { background: #e8f6ee; border-color: #0b7a43; }
blockquote.achtung { background: #fdf1dc; border-color: #c07a00; }
figure { margin: 0 0 5mm; text-align: center; break-inside: avoid; }
figure img { max-width: 100%; max-height: 120mm; border: 1px solid #ccd3e0; }
figcaption { font-size: 8.8pt; color: #566074; margin-top: 1.5mm; }
a { color: #0a63b5; text-decoration: none; }
ul, ol { margin: 0 0 3mm; padding-left: 6mm; }
li { margin-bottom: 1mm; }
.cover { height: 297mm; box-sizing: border-box; padding: 55mm 25mm 25mm; background: #0d1b2e; color: #fff; break-after: page; position: relative; }
.cover .logo { height: 34mm; margin-bottom: 14mm; }
.cover h1 { color: #fff; font-size: 34pt; border: 0; margin: 0 0 4mm; }
.cover .sub { font-size: 15pt; color: #9fc4ea; margin: 0 0 18mm; }
.cover .meta { font-size: 10.5pt; color: #c8d6e8; line-height: 1.7; }
.cover .hinweis { position: absolute; left: 25mm; right: 25mm; bottom: 22mm; border: 1px solid #c07a00; background: #2a2110; color: #f5d28a; padding: 4mm 5mm; font-size: 9.5pt; }
.cover .bar { position: absolute; left: 0; right: 0; bottom: 0; height: 5mm; background: linear-gradient(90deg,#c0263f 0 20%,#e8a000 0 40%,#2ba84a 0 60%,#0a63b5 0 80%,#7b38c2 0); }
.toc h1 { border: 0; }
.toc ol { list-style: none; padding: 0; counter-reset: x; }
.toc li { margin: 0 0 1.6mm; font-size: 10.5pt; }

.toc .grp { margin-top: 4mm; font-weight: bold; color: #0b3a66; }
.legal { font-size: 9pt; color: #566074; }
"""

LOGO = os.path.join(REPO, "installer/macos/VintageCoder-1024.png")


def b64(path):
    with open(path, "rb") as f:
        return "data:image/png;base64," + base64.b64encode(f.read()).decode()


def split_front(text):
    m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
    if not m:
        return {}, text
    fm = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            fm[k.strip()] = v.strip()
    return fm, text[m.end():]


def render(md_text, prefix, known, imgdir=None):
    """Markdown -> HTML. known: Menge der Abschnitts-IDs mit Anker; prefix kennzeichnet den Quelltyp."""
    # Bilder: ![alt](bild:id "Unterschrift")
    def img(m):
        alt, bid, cap = m.group(1), m.group(2), m.group(3) or alt
        p = os.path.join(HB, "bilder", bid + ".png")
        if not os.path.exists(p):
            return ""
        return f'\n\n<figure><img src="{b64(p)}" alt="{html.escape(alt)}"><figcaption>{html.escape(cap)}</figcaption></figure>\n\n'
    md_text = re.sub(r'!\[([^\]]*)\]\(bild:([a-z0-9-]+)(?: "([^"]*)")?\)', img, md_text)

    # Verweise: kapitel:/help:/hilfe: -> Anker, sonst reiner Text
    def link(m):
        text, kind, target = m.group(1), m.group(2), m.group(3)
        key = f"{'k' if kind == 'kapitel' else 'h'}-{target}"
        if key in known:
            return f"[{text}](#{key})"
        return text
    md_text = re.sub(r'\[([^\]]+)\]\((kapitel|help|hilfe):([a-z0-9-]+)\)', link, md_text)
    body = markdown.markdown(md_text, extensions=["tables", "fenced_code", "sane_lists"])
    # Hinweisarten
    body = re.sub(r'<blockquote>\s*<p><strong>(%s)' % TXT["tip"], r'<blockquote class="tipp"><p><strong>\1', body)
    body = re.sub(r'<blockquote>\s*<p><strong>(%s)' % TXT["caution"], r'<blockquote class="achtung"><p><strong>\1', body)
    return body


def cover(title, sub, lines):
    meta = "<br>".join(lines)
    return f"""<div class="cover"><img class="logo" src="{b64(LOGO)}"><h1>{html.escape(title)}</h1>
<p class="sub">{html.escape(sub)}</p><p class="meta">{meta}</p>
<div class="hinweis"><b>{html.escape(TXT["hinweis_b"])}</b> {html.escape(TXT["hinweis"])}</div>
<div class="bar"></div></div>"""


def page(title, parts):
    return f"""<!doctype html><html lang="{LANG}"><head><meta charset="utf-8"><title>{html.escape(title)}</title>
<style>{CSS}</style></head><body><div class="doctitle">{html.escape(title)}</div>{''.join(parts)}</body></html>"""


def toc(entries):
    out = ['<section class="toc"><h1>' + TXT['toc'] + '</h1><ol>']
    for kind, anchor, title in entries:
        if kind == "grp":
            out.append(f'<li class="grp">{html.escape(title)}</li>')
        else:
            out.append(f'<li><a href="#{anchor}">{html.escape(title)}</a></li>')
    out.append("</ol></section>")
    return "".join(out)


def pdf(htmlfile, pdffile):
    subprocess.run([EDGE, "--headless=new", "--disable-gpu", "--no-pdf-header-footer", "--no-sandbox", "--generate-pdf-document-outline",
                    f"--print-to-pdf={pdffile}", "file://" + htmlfile],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=600)


def chapters():
    res = []
    d = os.path.join(HBL, "kapitel")
    for fn in sorted(os.listdir(d)):
        if not fn.endswith(".md"):
            continue
        fm, body = split_front(open(os.path.join(d, fn), encoding="utf-8").read())
        res.append((fm.get("id", fn[:-3]), fm, body))
    return res


def build_ide():
    chs = chapters()
    known = {f"k-{cid}" for cid, _, _ in chs} | {"k-glossar"}
    parts = [cover("VintageCoder", TXT["hb_sub"],
                   [f"{TXT['version']} {VERSION}", f"{TXT['stand']} {STAND}", TXT["hb_line"], "https://vintagecoder.drueeke.de"])]
    entries, secs = [], []
    for cid, fm, body in chs:
        title = fm.get("titel") or re.search(r"^# (.+)$", body, re.M).group(1)
        n = fm.get("reihenfolge")
        label = title
        entries.append(("kap", f"k-{cid}", label))
        secs.append(f'<section class="kap" id="k-{cid}">{render(body, "k", known)}</section>')
    gl = open(os.path.join(HBL, "GLOSSAR.md"), encoding="utf-8").read()
    fm, glbody = split_front(gl)
    entries.append(("kap", "k-glossar", TXT["glossary"]))
    secs.append(f'<section class="kap" id="k-glossar">{render(glbody, "k", known)}</section>')
    parts.append(toc(entries))
    parts += secs
    parts.append('<section class="kap legal"><h2>%s</h2><p>%s</p></section>' % (html.escape(TXT["legal_h"]), html.escape(TXT["legal"])))
    return TXT["hb_title"], "".join(parts) and page(TXT["hb_title"], parts)


ASM_IDS = ["assembler", "asm-expr", "asm-directives", "asm-text", "asm-macros", "asm-library", "cartridges", "asm-index"]
ASM_PAGES = list(zip(ASM_IDS, TXT["asm_pages"]))


def build_asm():
    chs = {cid: (fm, body) for cid, fm, body in chapters()}
    known = {"k-assembler"} | {f"h-{pid}" for pid, _ in ASM_PAGES}
    parts = [cover("VCASM", TXT["asm_sub"],
                   [f"{TXT['version']} {VERSION}", f"{TXT['stand']} {STAND}", TXT["asm_line"], "6502, 6510, 8502, 7501, Sally, 45GS02", "https://vintagecoder.drueeke.de"])]
    entries, secs = [], []
    fm, body = chs["assembler"]
    entries.append(("grp", "", TXT["intro"]))
    entries.append(("kap", "k-assembler", TXT["asm_first"]))
    secs.append(f'<section class="kap" id="k-assembler">{render(body, "k", known)}</section>')
    entries.append(("grp", "", TXT["ref"]))
    for pid, ptitle in ASM_PAGES:
        text = open(os.path.join(HELP, pid + ".md"), encoding="utf-8").read()
        entries.append(("kap", f"h-{pid}", ptitle))
        secs.append(f'<section class="kap" id="h-{pid}">{render(text, "h", known)}</section>')
    parts.append(toc(entries))
    parts += secs
    return TXT["asm_title"], page(TXT["asm_title"], parts)


def md_convert(text, known, with_images):
    """Handbuch-Markdown -> Standard-Markdown mit Anker-Verweisen und relativen Bildern."""
    used = []
    def img(m):
        alt, bid, cap = m.group(1), m.group(2), m.group(3) or m.group(1)
        if not os.path.exists(os.path.join(HB, "bilder", bid + ".png")):
            return ""
        used.append(bid)
        return f"![{cap}](bilder/{bid}.png)"
    text = re.sub(r'!\[([^\]]*)\]\(bild:([a-z0-9-]+)(?: "([^"]*)")?\)', img, text)
    def link(m):
        t, kind, target = m.group(1), m.group(2), m.group(3)
        key = f"{'k' if kind == 'kapitel' else 'h'}-{target}"
        return f"[{t}](#{key})" if key in known else t
    text = re.sub(r'\[([^\]]+)\]\((kapitel|help|hilfe):([a-z0-9-]+)\)', link, text)
    return text, used


def write_md(outdir, name, title, intro, items, known):
    """items: (anker, titel, markdown). Schreibt <name>.md und kopiert benutzte Bilder nach bilder/."""
    import shutil
    os.makedirs(os.path.join(outdir, "bilder"), exist_ok=True)
    out = [f"# {title}\n", intro + "\n", "## " + TXT["toc"] + "\n"]
    for a, t, _ in items:
        out.append(f"- [{t}](#{a})")
    out.append("")
    for a, t, body in items:
        text, used = md_convert(body, known, True)
        for bid in used:
            shutil.copy(os.path.join(HB, "bilder", bid + ".png"), os.path.join(outdir, "bilder", bid + ".png"))
        out.append(f'\n---\n\n<a id="{a}"></a>\n\n{text.strip()}\n')
    open(os.path.join(outdir, name + ".md"), "w", encoding="utf-8").write("\n".join(out))


def md_ide(outdir):
    chs = chapters()
    known = {f"k-{cid}" for cid, _, _ in chs} | {"k-glossar"}
    items = []
    for cid, fm, body in chs:
        title = fm.get("titel") or re.search(r"^# (.+)$", body, re.M).group(1)
        items.append((f"k-{cid}", title, body))
    _, glbody = split_front(open(os.path.join(HBL, "GLOSSAR.md"), encoding="utf-8").read())
    items.append(("k-glossar", TXT["glossary"], glbody))
    write_md(outdir, "VintageCoder-Handbuch" + SUF, TXT["hb_title"], TXT["hb_md"].format(v=VERSION, d=STAND), items, known)


def md_asm(outdir):
    chs = {cid: (fm, body) for cid, fm, body in chapters()}
    known = {"k-assembler"} | {f"h-{pid}" for pid, _ in ASM_PAGES}
    items = [("k-assembler", TXT["asm_first"], chs["assembler"][1])]
    for pid, t in ASM_PAGES:
        items.append((f"h-{pid}", t, open(os.path.join(HELP, pid + ".md"), encoding="utf-8").read()))
    write_md(outdir, "VCASM-Dokumentation" + SUF, TXT["asm_title"], TXT["asm_md"].format(v=VERSION, d=STAND), items, known)


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for name, (title, doc) in {"VintageCoder-Handbuch" + SUF: build_ide(), "VCASM-Dokumentation" + SUF: build_asm()}.items():
        h = os.path.join(OUT, name + ".html")
        open(h, "w", encoding="utf-8").write(doc)
        pdf(h, os.path.join(OUT, name + ".pdf"))
        print("fertig:", name)
    md_ide(os.path.join(OUT, "markdown-handbuch" + SUF))
    md_asm(os.path.join(OUT, "markdown-vcasm" + SUF))
    print("Markdown fertig")
