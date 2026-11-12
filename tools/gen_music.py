#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt examples/{c64,mega65}/music_demo/lib/music.inc aus der Vorlage music.inc.in (die Vorlage ist die C64-Fassung mit drei Stimmen; die MEGA65-Fassung mit vier SIDs entsteht daraus durch Textersetzung). Die Frequenztabellen kommen aus <c64/notes.inc> bzw. <mega65/notes.inc> (tools/gen_notes.py).
Aufruf: gen_music.py  (schreibt beide music.inc neu)"""
import argparse, os
_ap = argparse.ArgumentParser(
    description="""Erzeugt examples/c64/music_demo/lib/music.inc und examples/mega65/music_demo/lib/music.inc aus der Vorlage examples/c64/music_demo/lib/music.inc.in.

Die Vorlage ist die C64-Fassung mit drei Stimmen; die MEGA65-Fassung mit vier SIDs (12 Stimmen) entsteht daraus durch Textersetzung. Die Frequenztabellen
kommen aus <c64/notes.inc> bzw. <mega65/notes.inc> (erzeugt von tools/gen_notes.py). Beide music.inc werden neu geschrieben (UTF-8; das vorhandene
Zeilenende CRLF/LF wird beibehalten). Der Arbeitsordner ist egal (alle Pfade gelten relativ zum Ort des Skripts).""",
    epilog="""Beispiele:
  python3 tools/gen_music.py
  cd tools && python3 gen_music.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und schreibt nichts.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()
root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
def fill(path, repl):
    t = open(path, encoding="utf-8").read()
    for k, v in repl.items(): t = t.replace(k, v)
    open(path, "w", encoding="utf-8", newline="").write(t)
def mega65_variant(t):
    """C64-Spielroutine → MEGA65 mit vier SIDs (12 Stimmen): Register $D400/$D420/$D440/$D460"""
    t = t.replace("music.inc – C64: Spielroutine für Tracker-Songs (SID, 3 Stimmen)", "music.inc – MEGA65: Spielroutine für Tracker-Songs (4 SIDs = 12 Stimmen, Profil mega65.sid4.song)")
    t = t.replace("mus_sid = Abbild der SID-Register ($D400-$D418)", "mus_sid = Abbild der SID-Register ($D400-$D47F, 4 SIDs)")
    t = t.replace("        sta mus_ins+1\n        sta mus_ins+2\n", "".join(f"        sta mus_ins+{i}\n" for i in range(1, 12)))
    t = t.replace("        ldx #24\n@clr:", "        ldx #127\n@clr:")
    t = t.replace("        sta mus_sid+24\n        sta $d418\n", "".join(f"        sta mus_sid+{o + 24}\n        sta ${0xD418 + o:04x}\n" for o in (0, 32, 64, 96)))
    t = t.replace("        sta $d404\n        sta $d40b\n        sta $d412\n        sta $d418\n", "".join(f"        sta ${0xD404 + o + v * 7:04x}\n" for o in (0, 32, 64, 96) for v in range(3)) + "".join(f"        sta ${0xD418 + o:04x}\n" for o in (0, 32, 64, 96)))
    t = t.replace("        cpx #3\n", "        cpx #12\n").replace("        adc #6\n", "        adc #24\n")
    t = t.replace("mus_ins:   .byte 0,0,0\n", "mus_ins:   .fill 12, 0\n").replace("mus_voff:  .byte 0,7,14\n", "mus_voff:  .byte 0,7,14,32,39,46,64,71,78,96,103,110\n").replace("mus_sid:   .fill 25, 0", "mus_sid:   .fill 128, 0")
    t = t.replace("(3 Stimmen * 2 Bytes)", "(12 Stimmen * 2 Bytes)").replace("Instrumente: höchstens 32 (SID-Datensatz 8 Bytes)", "Instrumente: höchstens 32 (SID-Datensatz 8 Bytes); Zeropage und Aufruf wie beim C64")
    t = t.replace("#include <c64/notes.inc>", "#include <mega65/notes.inc>").replace("<c64/notes.inc>", "<mega65/notes.inc>")
    return t
for src in ("c64", "mega65"):
    # UTF-8 explizit (unabhaengig von der Windows-Standardcodierung); Zeilenenden werden beim Lesen zu \n
    t = open(os.path.join(root, "examples/c64/music_demo/lib/music.inc.in"), encoding="utf-8").read()
    if src == "mega65": t = mega65_variant(t)
    out = os.path.join(root, f"examples/{src}/music_demo/lib/music.inc")
    crlf = os.path.exists(out) and b"\r\n" in open(out, "rb").read()   # Zeilenende der vorhandenen Datei beibehalten
    open(out, "w", encoding="utf-8", newline="\r\n" if crlf else "\n").write(t)
print("music.inc erzeugt")
