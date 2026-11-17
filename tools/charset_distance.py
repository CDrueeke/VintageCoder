#!/usr/bin/env python3
# SPDX-License-Identifier: CC0-1.0
"""Abstandsprüfung der eigenen Zeichensätze (data/charsets/) gegen Zeichensätze, die der Anwender LOKAL hat (Ebene A des Konzepts).

Nur Zahlen: je eigenem Zeichen der kleinste Hamming-Abstand (von 64 Punkten) zu jedem Zeichen der Vergleichsdateien (gleicher Code und beliebiger Code,
auch invertiert, um ±1 Punkt verschoben, gespiegelt), Anzahl exakter Treffer, Verteilung. Es werden nie Zeichenbilder des Vergleichs ausgegeben oder gespeichert,
und das Werkzeug legt nichts davon im Repository ab. Die Vergleichsdateien gibt der Anwender selbst an (rohe 8-Byte-je-Zeichen-Abbilder, z. B. seine ROM-Dateien);
dieses Werkzeug wird vom Projektinhaber lokal ausgeführt, nie in der CI und nie von einer Sitzung mit Original-ROMs.

Aufruf:
  python3 tools/charset_distance.py --selbst                   Kalibrierung ohne Fremddaten: die eigenen Sätze untereinander
  python3 tools/charset_distance.py DATEI.bin [DATEI2.bin ...]  Vergleich gegen eigene Abbilder (Beginn jeder Datei = Zeichen 0, je 8 Byte)
  Option --stempel: setzt in den Sätzen, die den Schwellenwert erreichen, origin.check (Datum, min_distance, result "bestanden"); ohne sie wird nichts geschrieben.
  Option --schwelle N: kleinster erlaubter Abstand für die übrigen nicht ausgenommenen Zeichen (Vorgabe 6).
  Option --schwelle-zeichen N: kleinster erlaubter Abstand für Buchstaben und Symbole/Ziffern (Vorgabe 3).
Die Schwelle gilt klassenweise (Entscheidung des Projektinhabers 2026-11-17): Buchstaben sowie Symbole und Ziffern 3, alle übrigen nicht ausgenommenen
Zeichen (Zeichen wie [ £ ] ↑ ←, Grafik) 6. Zu jedem nicht bestandenen Satz nennt die Ausgabe die betroffenen CODES mit ihrem Abstand (Zahlen in
Klammern), nach Klassen geordnet, nie Zeichenbilder des Vergleichs.
Nur Standardbibliothek."""
import datetime, glob, json, os, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CH = os.path.join(ROOT, "data", "charsets")
SCHWELLE = 6            # übrige Zeichen (Sonderzeichen, Grafik)
SCHWELLE_ZEICHEN = 3    # Buchstaben, Symbole und Ziffern
KLASSEN = ("Buchstabe", "Symbol/Ziffer", "übrig")


def klasse(eintrag, code):
    """Klasse eines Zeichencodes im eigenen Satz: 'Buchstabe', 'Symbol/Ziffer' oder 'übrig' (nur echte Grafikzeichen, die weder geometrisch noch Muster sind).
    Die zweite Hälfte der 256er-Sätze ist die Bitnegation der ersten und hat dieselbe Klasse. Codes 27 bis 31 ([ £ ] ↑ ←) sind Symbole."""
    c = code % 128
    if eintrag["system"] == "atari8":
        # ANTIC-Codes: 0 bis 63 = ASCII 32 bis 95 (Symbole/Ziffern 0 bis 32 und 59 bis 63, Buchstaben 33 bis 58), 64 bis 95 Steuerzeichen-Grafik (Pfeile, Esc: Symbole),
        # 96 bis 127 = ASCII 96 bis 127 (Buchstaben 97 bis 122). International: 64 bis 90 sind Akzentbuchstaben (72 = £), 96 = ¡, 123 = Ä.
        if eintrag["satz"] == "international":
            if 64 <= c <= 90 and c != 72 or c == 123:
                return "Buchstabe"
            if c in (72, 96):
                return "Symbol/Ziffer"
        if 33 <= c <= 58 or 97 <= c <= 122:
            return "Buchstabe"
        if c <= 32 or 59 <= c <= 63 or 92 <= c <= 95 or 125 <= c <= 127:
            return "Symbol/Ziffer"
        return "übrig"
    if 1 <= c <= 26 or (eintrag["satz"].endswith("lower") and 65 <= c <= 90):
        return "Buchstabe"
    return "Symbol/Ziffer" if c <= 63 else "übrig"


def lade_satz(rel):
    d = json.load(open(os.path.join(CH, rel), encoding="utf-8"))
    return d


def bits_of(rows):
    """Zeichen als 64-Bit-Zahl (8 Zeilen zu 8 Punkten); nur 1-Bit-8×8-Sätze."""
    v = 0
    for r in rows:
        v = (v << 8) | int(r, 2)
    return v


def varianten(v):
    """Alle zu vergleichenden Fassungen eines Zeichens: invertiert, um ±1 Punkt verschoben (waagerecht/senkrecht), gespiegelt."""
    m = (1 << 64) - 1
    out = set()
    base = [v, v ^ m]
    rows = [(v >> (8 * (7 - i))) & 255 for i in range(8)]
    sp = sum((int(format(r, "08b")[::-1], 2)) << (8 * (7 - i)) for i, r in enumerate(rows))
    base += [sp, sp ^ m, int("".join(format(r, "08b") for r in rows[::-1]), 2)]
    for b in base:
        out.add(b)
        rr = [(b >> (8 * (7 - i))) & 255 for i in range(8)]
        for dx in (-1, 1):
            s = [((r << 1) & 255) if dx > 0 else (r >> 1) for r in rr]
            out.add(sum(x << (8 * (7 - i)) for i, x in enumerate(s)))
        for dy in (-1, 1):
            s = rr[1:] + [0] if dy > 0 else [0] + rr[:-1]
            out.add(sum(x << (8 * (7 - i)) for i, x in enumerate(s)))
    return out


def ham(a, b):
    return bin(a ^ b).count("1")


def eigene_zeichen(eintrag):
    d = lade_satz(eintrag["datei"])
    if eintrag["hoehe"] != 8 or d["profile"] == "mega65.char.fcm":
        return None
    return [bits_of(f["rows"]) for f in d["frames"]]


def vergleich(satz_id, zeichen, ref, ausnahmen, eintrag=None, schwelle=SCHWELLE, schwelle_zeichen=SCHWELLE_ZEICHEN):
    """Liefert (kleinster Abstand über nicht ausgenommene Zeichen, exakte Treffer, Verteilung, zu nahe Codes je Klasse).
    Zu nahe Codes: {Klasse: [(Code, Abstand), …]}; leer ohne `eintrag`."""
    refv = set()
    for r in ref:
        refv |= varianten(r)
    verteilung = {}
    kleinster = 64
    exakt = 0
    zu_nah = {}
    for code, z in enumerate(zeichen):
        if str(code) in ausnahmen:
            continue
        dmin = min(ham(z, r) for r in refv)
        verteilung[dmin] = verteilung.get(dmin, 0) + 1
        kleinster = min(kleinster, dmin)
        if dmin == 0:
            exakt += 1
        if eintrag is not None:
            kl = klasse(eintrag, code)
            if dmin < (schwelle_zeichen if kl != "übrig" else schwelle):
                zu_nah.setdefault(kl, []).append((code, dmin))
    return kleinster, exakt, verteilung, zu_nah


def main(argv):
    if len(argv) < 2 or argv[1] in ("-h", "--help"):
        print(__doc__)
        return 0
    schwelle = SCHWELLE
    if "--schwelle" in argv:
        i = argv.index("--schwelle")
        schwelle = int(argv[i + 1])
        del argv[i:i + 2]
    schwelle_zeichen = SCHWELLE_ZEICHEN
    if "--schwelle-zeichen" in argv:
        i = argv.index("--schwelle-zeichen")
        schwelle_zeichen = int(argv[i + 1])
        del argv[i:i + 2]
    stempel = "--stempel" in argv
    argv = [a for a in argv if a != "--stempel"]
    index = json.load(open(os.path.join(CH, "index.json"), encoding="utf-8"))["saetze"]
    ausn = json.load(open(os.path.join(CH, "ausnahmen.json"), encoding="utf-8"))["saetze"]
    eigene = {e["id"]: e for e in index}
    if "--selbst" in argv:
        # Kalibrierung: C64 Groß gegen C64 Klein und Atari (unabhängige Sätze desselben Entwurfs, nur zur Kenntnis der Verteilung)
        for a, b in (("vc-c64-upper", "vc-atari-standard"), ("vc-c64-lower", "vc-atari-standard")):
            za, zb = eigene_zeichen(eigene[a]), eigene_zeichen(eigene[b])
            k, ex, v, _ = vergleich(a, za, zb, ausn[a])
            print("%s gegen %s: kleinster Abstand %d, exakte Treffer %d, Verteilung %s" % (a, b, k, ex, sorted(v.items())))
        return 0
    ref = []
    for pfad in argv[1:]:
        roh = open(pfad, "rb").read()
        for i in range(0, len(roh) - 7, 8):
            ref.append(int.from_bytes(roh[i:i + 8], "big"))
    ok = True
    for e in index:
        z = eigene_zeichen(e)
        if z is None:
            continue
        k, ex, v, zu_nah = vergleich(e["id"], z, ref, ausn[e["id"]], e, schwelle, schwelle_zeichen)
        gut = not zu_nah and ex == 0
        ok = ok and gut
        print("%-24s kleinster Abstand %2d, exakte Treffer %d, %s" % (e["id"], k, ex, "bestanden" if gut else "OFFEN: Zeichen im Studio prüfen/ändern oder in der Ausnahmeliste begründen"))
        print("   Verteilung (Abstand:Anzahl):", " ".join("%d:%d" % kv for kv in sorted(v.items())[:12]))
        for kl in KLASSEN:
            if kl in zu_nah:
                print("   %-14s Schwelle %d, %2d Codes: %s" % (kl, schwelle_zeichen if kl != "übrig" else schwelle, len(zu_nah[kl]), " ".join("%d(%d)" % x for x in zu_nah[kl])))
        if stempel and gut:
            p = os.path.join(CH, e["datei"])
            txt = open(p, encoding="utf-8").read()
            d = json.loads(txt)
            d["origin"]["check"].update({"date": datetime.date.today().isoformat(), "result": "bestanden", "min_distance": k, "schwellen": {"buchstabe_symbol": schwelle_zeichen, "uebrig": schwelle}})
            # nur die Kopfzeile origin neu schreiben, Zeichenzeilen bleiben byteweise erhalten
            zeilen = txt.split("\n")
            for i, z in enumerate(zeilen):
                if z.startswith('  "origin":'):
                    zeilen[i] = '  "origin": %s,' % json.dumps(d["origin"], ensure_ascii=False, separators=(",", ":"))
            open(p, "w", encoding="utf-8").write("\n".join(zeilen))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
