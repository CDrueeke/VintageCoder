#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt je System eine Include-Datei mit benannten Notenkonstanten und Frequenztabelle:
data/sys/{c64,c128,mega65,plus4,vic20,atari8bit}/notes.inc

Alles folgt aus den physikalischen Formeln: gleichstufige Stimmung, A4 = 440 Hz, Notenwert 1 = C0 … 58 = A4 … 96 = B7
(wie im Tracker, crates/vc-media/src/song.rs).  Die Tabellen sind unmittelbar mit dem Notenwert indiziert (Eintrag 0 = "keine Note").
Je Chip eine PAL- und eine NTSC-Tabelle; gewählt wird mit  -D NTSC=1  (oder .const NTSC = 1 vor dem Include) wie bei music.inc.

Aufruf:
  python3 tools/gen_notes.py            schreibt alle Dateien neu
  python3 tools/gen_notes.py --check    vergleicht mit den eingecheckten Dateien (Abweichung = Fehler, Exit 1)
"""
import argparse
import math
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
NAMES = ["C", "CS", "D", "DS", "E", "F", "FS", "G", "GS", "A", "AS", "B"]
ALIAS = {"CS": "DB", "DS": "EB", "FS": "GB", "GS": "AB", "AS": "BB"}   # englische b-Namen (Des, Es, Ges, As, B)
GERMAN = {"C": "C", "CS": "Cis", "D": "D", "DS": "Dis", "E": "E", "F": "F", "FS": "Fis", "G": "G", "GS": "Gis", "A": "A", "AS": "B", "B": "H"}
FIRST, LAST = 1, 96
A4 = 58


def hz(n):
    """Frequenz des Notenwerts n (C0 = 1, A4 = 58 = 440 Hz)."""
    return 440.0 * 2 ** ((n - A4) / 12)


def rnd(x):
    """Kaufmännisch runden (x.5 nach oben), unabhängig von Fließkomma-Gerade/Ungerade-Regel."""
    return int(math.floor(x + 0.5))


def bytes_block(vals, per=12, fmt="${:02x}"):
    return "\n".join("        .byte " + ", ".join(fmt.format(v) for v in vals[i:i + per]) for i in range(0, len(vals), per))


def table(label, vals, comment=None):
    out = []
    if comment:
        out.append("// " + comment)
    out.append(label + ":")
    # Eintrag 0 steht allein, danach Oktaven zu je 12 Noten
    out.append(bytes_block(vals[:1]))
    out.append(bytes_block(vals[1:]))
    return "\n".join(out)


# ---------------------------------------------------------------- Chips
# Jeder Chip liefert für einen Takt: Tabellen (Liste von (Label, Werte, Kommentar)) und Grenznoten (lowest, highest) ohne Falten/Kappen.

def sid(clock):
    vals = [0] + [min(65535, rnd(hz(n) * 16777216 / clock)) for n in range(FIRST, LAST + 1)]
    hi_ok = max(n for n in range(FIRST, LAST + 1) if hz(n) * 16777216 / clock <= 65535.4)
    tabs = [("note_sid_lo", [v & 255 for v in vals], "SID-Frequenzzähler (16 Bit), niedriges Byte: Fn = f * 2^24 / Takt"),
            ("note_sid_hi", [v >> 8 for v in vals], "SID-Frequenzzähler, hohes Byte")]
    return tabs, 1, hi_ok, vals


def ted(clock):
    """TED: f = Takt/8 / (1024 - N).  Töne unter 110 Hz fallen um ganze Oktaven nach oben (N = 0 liefert nur ~108 Hz)."""
    base = clock / 8.0
    lo_ok = min(n for n in range(FIRST, LAST + 1) if base / 1024 <= hz(n))
    vals = [0]
    for n in range(FIRST, LAST + 1):
        f = hz(n)
        while f < base / 1024 - 1e-9:
            f *= 2
        vals.append(min(1023, rnd(1024 - base / f)))
    tabs = [("note_ted_lo", [v & 255 for v in vals], "TED-Tonwert N (10 Bit), niedriges Byte: N = 1024 - Takt/8 / f"),
            ("note_ted_hi", [v >> 8 for v in vals], "TED-Tonwert N, obere 2 Bit")]
    return tabs, lo_ok, LAST, vals


def vic(clock):
    """VIC-I: f = Takt * m / (256 * (127 - N)), m = 1 (Bass), 2 (Alt), 4 (Sopran); Wert = $80 | N.  Zu tiefe/hohe Töne werden oktaviert."""
    tabs = []
    for name, m, text in (("sopr", 4, "Sopran ($900C)"), ("alto", 2, "Alt ($900B)"), ("bass", 1, "Bass ($900A)")):
        vals = [0]
        for n in range(FIRST, LAST + 1):
            f = hz(n)
            while clock * m / (256 * f) > 126.5:
                f *= 2
            while clock * m / (256 * f) < 0.5:
                f /= 2
            k = max(1, min(127, rnd(clock * m / (256 * f))))
            vals.append(0x80 | (127 - k))
        tabs.append(("note_vic_" + name, vals, "VIC-I-Registerwerte ($80 | N), " + text))
    return tabs, None, None, None


def pokey(clock):
    """POKEY: 8 Bit bei 64 kHz: f = (Takt/28) / (2 (AUDF + 1));  16 Bit bei Takt: f = Takt / (2 (AUDF + 7))."""
    b8 = clock / 28.0
    v8 = [255] + [rnd(b8 / (2 * hz(n))) - 1 for n in range(FIRST, LAST + 1)]
    lo8 = min(n for n in range(FIRST, LAST + 1) if rnd(b8 / (2 * hz(n))) - 1 <= 255)
    v8 = [255] + [(v if v <= 255 else 255) for v in v8[1:]]
    v16 = [0] + [rnd(clock / (2 * hz(n))) - 7 for n in range(FIRST, LAST + 1)]
    assert all(0 <= v <= 65535 for v in v16)
    tabs = [("note_pokey8", v8, "POKEY 8 Bit (64 kHz), AUDF-Wert; $FF = nicht darstellbar (tiefer als NOTE_POKEY8_LOWEST)"),
            ("note_pokey16_lo", [v & 255 for v in v16], "POKEY 16 Bit (Kanäle 1+2 oder 3+4 gekoppelt, Takt 1,79 MHz), niedriges Byte (AUDF1 bzw. AUDF3)"),
            ("note_pokey16_hi", [v >> 8 for v in v16], "POKEY 16 Bit, hohes Byte (AUDF2 bzw. AUDF4)")]
    return tabs, lo8, LAST, v8


CLOCKS = {
    "sid": (985248, 1022727),
    "ted": (886724, 894886),
    "vic": (1108405, 1022727),
    "pokey": (1773447, 1789790),
}


# ---------------------------------------------------------------- Dateien
def header(sysname, chip, lines):
    h = ["// notes.inc – Notenkonstanten und Frequenztabelle " + sysname + " (" + chip + ").",
         "// ERZEUGT durch tools/gen_notes.py – nicht von Hand ändern (Änderungen im Skript, dann neu erzeugen; `--check` prüft).",
         "// Einbinden: #include <" + SYSDIR[sysname] + "/notes.inc>"]
    h += ["//"] + ["// " + l if l else "//" for l in lines]
    return "\n".join(h)


SYSDIR = {"C64": "c64", "C128": "c128", "MEGA65": "mega65", "Plus/4": "plus4", "VC-20": "vic20", "Atari 8 Bit": "atari8bit"}

COMMON_DOC = [
    "Notenwerte: NOTE_C0 = 1 … NOTE_A4 = 58 (440 Hz) … NOTE_B7 = 96, gleichstufig, wie im Tracker (C-4 = NOTE_C4).",
    "Schreibweise englisch: CS = Cis (C#), DB = Des (Db) usw.; B ist das deutsche H, BB das deutsche B (AS und BB sind derselbe Ton).",
    "Liedformat der Spielroutinen: NOTE_HOLD (0) = Note halten, NOTE_REST (96) = Pause (Gate aus).  Achtung: NOTE_B7 hat denselben Wert",
    "wie NOTE_REST; als Liednote nutzbar sind daher NOTE_C0 … NOTE_AS7 (1..95).  Als reiner Tabellenindex ist NOTE_B7 gültig.",
    "Die Tabellen sind mit dem Notenwert indiziert (Eintrag 0 = keine Note):  ldx #NOTE_A4 / lda note_..._lo,x.",
    "Selbst anlegen lässt sich ein Ton als  NOTE_C4 + 7  (Quinte) oder  NOTE_C4 + 12 * 2  (zwei Oktaven höher).",
    "PAL ist Standard; mit  -D NTSC=1  (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.  Ohne Tabelle (nur Konstanten): NOTES_NO_TABLE definieren.",
]


def constants():
    o = [".const NOTE_HOLD = 0", ".const NOTE_REST = 96", ".const NOTE_FIRST = 1", ".const NOTE_LAST = 96", ""]
    for n in range(FIRST, LAST + 1):
        name = NAMES[(n - 1) % 12]
        o.append(f".const NOTE_{name}{(n - 1) // 12} = {n}")
        if name in ALIAS:
            o.append(f".const NOTE_{ALIAS[name]}{(n - 1) // 12} = {n}")
    return "\n".join(o)


def limits_lines(lo, hi):
    return lo, hi


def build(sysname, chip, doc, kind):
    """kind: 'sid' | 'ted' | 'vic' | 'pokey'"""
    pal_clock, ntsc_clock = CLOCKS[kind]
    gen = {"sid": sid, "ted": ted, "vic": vic, "pokey": pokey}[kind]
    variants = [(True, ntsc_clock, gen(ntsc_clock)), (False, pal_clock, gen(pal_clock))]
    parts = [header(sysname, chip, COMMON_DOC + [""] + doc), "", ".ifndef NOTE_C0", "", constants(), ""]
    lim = {"sid": "NOTE_TABLE_HIGHEST", "ted": "NOTE_TABLE_LOWEST", "pokey": "NOTE_POKEY8_LOWEST"}.get(kind)
    why = {"sid": "höchster Ton ohne Kappung bei 65535 (darüber liefert die Tabelle 65535)",
           "ted": "tiefster Ton ohne Oktavfaltung (darunter steht der Eintrag ganze Oktaven höher)",
           "pokey": "tiefster mit 8 Bit darstellbarer Ton (darunter steht $FF)"}.get(kind)
    if lim:
        parts.append(".ifdef NTSC")
        for idx, (ntsc, clock, (tabs, lo, hi, _)) in enumerate(variants):
            if idx == 1:
                parts.append(".else")
            parts.append(f".const {lim} = {hi if kind == 'sid' else lo}   // {'NTSC' if ntsc else 'PAL'}: {why}")
        parts.append(".endif")
        parts.append("")
    parts.append(".ifndef NOTES_NO_TABLE")
    parts.append(".ifdef NTSC")
    for idx, (ntsc, clock, (tabs, lo, hi, _)) in enumerate(variants):
        if idx == 1:
            parts.append(".else")
        parts.append(f"// {'NTSC' if ntsc else 'PAL'}: Takt {clock} Hz")
        for label, vals, comment in tabs:
            parts.append(table(label, vals, comment))
    parts.append(".endif")
    parts.append(".endif")
    parts.append("")
    parts.append(".endif")
    return "\n".join(parts) + "\n"


FILES = {
    "c64": ("C64", "SID", "sid", [
        "SID-Tabellen: note_sid_lo / note_sid_hi, 16-Bit-Frequenzzähler Fn = f * 2^24 / Takt (PAL 985248 Hz, NTSC 1022727 Hz).",
        "Oberhalb von NOTE_TABLE_HIGHEST ist der Zähler voll (65535); höhere Noten klingen zu tief.  Registerlage: Stimme 1 $D400/$D401, 2 $D407/$D408, 3 $D40E/$D40F.",
        "Beispiel (Stimme 1):  ldx #NOTE_A4  /  lda note_sid_lo,x  /  sta $d400  /  lda note_sid_hi,x  /  sta $d401"]),
    "c128": ("C128", "SID", "sid", [
        "SID-Tabellen: note_sid_lo / note_sid_hi, 16-Bit-Frequenzzähler Fn = f * 2^24 / Takt (PAL 985248 Hz, NTSC 1022727 Hz).",
        "Oberhalb von NOTE_TABLE_HIGHEST ist der Zähler voll (65535); höhere Noten klingen zu tief.  Registerlage wie beim C64 ($D400 ff.).",
        "Beispiel (Stimme 1):  ldx #NOTE_A4  /  lda note_sid_lo,x  /  sta $d400  /  lda note_sid_hi,x  /  sta $d401"]),
    "mega65": ("MEGA65", "SID, vier Chips", "sid", [
        "SID-Tabellen: note_sid_lo / note_sid_hi, 16-Bit-Frequenzzähler Fn = f * 2^24 / Takt (PAL 985248 Hz, NTSC 1022727 Hz; wie die C64-Musikdemo).",
        "Die vier SIDs liegen bei $D400, $D420, $D440, $D460; die Tabellen gelten für alle.",
        "Oberhalb von NOTE_TABLE_HIGHEST ist der Zähler voll (65535); höhere Noten klingen zu tief.",
        "Beispiel (SID 1, Stimme 1):  ldx #NOTE_A4  /  lda note_sid_lo,x  /  sta $d400  /  lda note_sid_hi,x  /  sta $d401"]),
    "plus4": ("Plus/4", "TED", "ted", [
        "TED-Tabellen: note_ted_lo / note_ted_hi, 10-Bit-Tonwert N, f = (Takt / 8) / (1024 - N)  (PAL 886724 Hz, NTSC 894886 Hz Takt).",
        "Der TED erreicht nur etwa 108 Hz (N = 0) nach unten; Töne unterhalb von NOTE_TABLE_LOWEST stehen um ganze Oktaven nach oben gefaltet.",
        "Registerlage: Stimme 1 $FF0E (niedrig) / $FF12 Bits 0..1 (hoch), Stimme 2 $FF0F / $FF10.  Bit 2 und höher von $FF12 gehören dem Zeichensatz – nur maskiert schreiben."]),
    "vic20": ("VC-20", "VIC-I", "vic", [
        "VIC-I-Tabellen: note_vic_sopr ($900C), note_vic_alto ($900B), note_vic_bass ($900A); Wert = $80 | N (Bit 7 = Stimme an), f = Takt * m / (256 * (127 - N)).",
        "m = 4 (Sopran), 2 (Alt), 1 (Bass); Takt PAL 1108405 Hz, NTSC 1022727 Hz.  Jede Stimme hat nur etwa 7 Oktaven Umfang:",
        "Töne außerhalb (Bass tief, Sopran sehr tief/hoch) sind um ganze Oktaven in den Bereich verschoben – die Tonart bleibt, die Lage nicht.",
        "Das Rauschen ($900D) hat keine Tonhöhe und hier keine Tabelle."]),
    "atari8bit": ("Atari 8 Bit", "POKEY", "pokey", [
        "POKEY-Tabellen: note_pokey8 (AUDF, 8 Bit, 64-kHz-Takt: f = (Takt/28) / (2 (AUDF + 1))),",
        "note_pokey16_lo / note_pokey16_hi (16 Bit, gekoppelte Kanäle, voller Takt: f = Takt / (2 (AUDF + 7))).  Takt PAL 1773447 Hz, NTSC 1789790 Hz.",
        "Mit 8 Bit sind Töne unter NOTE_POKEY8_LOWEST nicht darstellbar (Tabelle $FF); die 16-Bit-Tabelle deckt alle 96 Noten ab.",
        "Reiner Ton ohne Verzerrung (AUDC Bits 5..7 = 10x) vorausgesetzt; mit Verzerrung ändert sich die Frequenz durch die Polynomzähler.",
        "Beispiel (Kanal 1, 8 Bit):  ldx #NOTE_A4  /  lda note_pokey8,x  /  sta $d200"]),
}


def generate():
    return {f"data/sys/{d}/notes.inc": build(*FILES[d][:1], FILES[d][1], FILES[d][3], FILES[d][2]) for d in FILES}


def main():
    ap = argparse.ArgumentParser(
        description="""Erzeugt je System eine Include-Datei mit benannten Notenkonstanten und Frequenztabelle: data/sys/{c64,c128,mega65,plus4,vic20,atari8bit}/notes.inc

Alles folgt aus den physikalischen Formeln (gleichstufige Stimmung, A4 = 440 Hz, Notenwert 1 = C0 ... 58 = A4 ... 96 = B7); je Chip eine PAL- und eine
NTSC-Tabelle (Auswahl beim Assemblieren mit -D NTSC=1). Ohne Option werden alle sechs notes.inc neu geschrieben (UTF-8, LF; vorhandene Dateien werden
überschrieben). Pfade gelten relativ zum Repository (aus dem Ort des Skripts bestimmt); der Arbeitsordner ist egal.""",
        epilog="""Beispiele:
  python3 tools/gen_notes.py            schreibt alle Dateien neu
  python3 tools/gen_notes.py --check    vergleicht nur mit den vorhandenen Dateien (schreibt nichts)""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--check", action="store_true",
                    help="nichts schreiben, sondern mit den vorhandenen Dateien vergleichen (Abweichung = Meldung auf der Fehlerausgabe, Exit-Code 1)")
    check = ap.parse_args().check
    bad = 0
    for rel, text in generate().items():
        path = os.path.join(ROOT, rel)
        if check:
            try:
                cur = open(path, encoding="utf-8").read()
            except OSError:
                cur = None
            if cur != text:
                print(f"ABWEICHUNG: {rel} entspricht nicht der Ausgabe von tools/gen_notes.py", file=sys.stderr)
                bad += 1
        else:
            with open(path, "w", encoding="utf-8", newline="\n") as f:
                f.write(text)
            print("geschrieben:", rel)
    if check:
        if bad:
            sys.exit(1)
        print("gen_notes.py --check: alle Dateien stimmen")


if __name__ == "__main__":
    main()
