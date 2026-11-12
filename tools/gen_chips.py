#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/chips/*.chip.json (Registerlisten) für Atari 8-Bit und C64."""
import argparse, json, os
_ap = argparse.ArgumentParser(
    description="""Erzeugt data/chips/*.chip.json (Registerlisten der Chips) für Atari 8-Bit, C64, VC-20 u. a.: u. a. gtia, pokey, antic, pia, os, vic2, sid, vic1, via1, via2.

Liest nichts und schreibt je Chip eine Datei data/chips/<kennung>.chip.json (UTF-8; vorhandene werden überschrieben; der Ordner wird bei Bedarf angelegt und relativ
zum Ort des Skripts bestimmt, der Arbeitsordner ist egal). Die MEGA65-Chips erzeugt tools/gen_chips_mega65.py.""",
    epilog="""Beispiele:
  python3 tools/gen_chips.py
  cd tools && python3 gen_chips.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()
OUT = os.path.join(os.path.dirname(__file__), "..", "data", "chips")
os.makedirs(OUT, exist_ok=True)

def reg(name, addr, access, doc, bits=None):
    r = {"name": name, "addr": f"${addr:04X}", "access": access, "doc": doc}
    if bits: r["bits"] = [{"name": n, "bit": b, **({"width": w} if w else {}), "doc": d} for n, b, w, d in bits]
    return r

def write(cid, name, start, end, regs, extra=None):
    d = {"$schema": "https://vintagecoder.dev/schema/chip-1.json", "id": cid, "name": name,
         "window": {"space": "cpu16", "start": f"${start:04X}", "end": f"${end:04X}"}}
    if extra: d.update(extra)
    d["regs"] = regs
    text = "{\n" + ",\n".join(f'  {json.dumps(k)}: {json.dumps(v, ensure_ascii=False)}' for k, v in d.items() if k != "regs")
    text += ',\n  "regs": [\n' + ",\n".join("    " + json.dumps(r, ensure_ascii=False) for r in regs) + "\n  ]\n}\n"
    open(os.path.join(OUT, f"{cid}.chip.json"), "w", encoding="utf-8").write(text)
    names = [r["name"] for r in regs]; assert len(names) == len(set(names)), cid

# ---------------------------------------------------------------- Atari
gtia = [reg(f"hposp{i}", 0xD000 + i, "w", f"Horizontale Position Spieler {i}") for i in range(4)]
gtia += [reg(f"hposm{i}", 0xD004 + i, "w", f"Horizontale Position Missile {i}") for i in range(4)]
gtia += [reg(f"sizep{i}", 0xD008 + i, "w", f"Breite Spieler {i} (0=1x, 1=2x, 3=4x)") for i in range(4)]
gtia += [reg("sizem", 0xD00C, "w", "Breite der Missiles"), *[reg(f"grafp{i}", 0xD00D + i, "w", f"Grafikdaten Spieler {i}") for i in range(4)],
         reg("grafm", 0xD011, "w", "Grafikdaten der Missiles")]
gtia += [reg(f"colpm{i}", 0xD012 + i, "w", f"Farbe Spieler/Missile {i}") for i in range(4)]
gtia += [reg(f"colpf{i}", 0xD016 + i, "w", f"Farbe Playfield {i}") for i in range(4)]
gtia += [reg("colbk", 0xD01A, "w", "Hintergrundfarbe/Rahmen"),
         reg("prior", 0xD01B, "w", "Prioritäten und GTIA-Modi", [("PRI0-3", 0, 4, "Prioritätstabelle"), ("5TH", 4, None, "Missiles = 5. Spieler"), ("MULTI", 5, None, "Multicolor-Spieler"), ("GTIA", 6, 2, "GTIA-Modus 9/10/11")]),
         reg("vdelay", 0xD01C, "w", "Vertikale Verzögerung"),
         reg("gractl", 0xD01D, "w", "Grafik-Steuerung", [("MISSILE", 0, None, "Missile-DMA-Daten übernehmen"), ("PLAYER", 1, None, "Spieler-DMA-Daten übernehmen"), ("LATCH", 2, None, "Trigger-Eingänge zwischenspeichern")]),
         reg("hitclr", 0xD01E, "w", "Kollisionsregister löschen"), reg("consol", 0xD01F, "rw", "Konsolentasten / Lautsprecher")]
gtia += [reg(f"m{i}pf", 0xD000 + i, "r", f"Kollision Missile {i} mit Playfield") for i in range(4)]
gtia += [reg(f"p{i}pf", 0xD004 + i, "r", f"Kollision Spieler {i} mit Playfield") for i in range(4)]
gtia += [reg(f"m{i}pl", 0xD008 + i, "r", f"Kollision Missile {i} mit Spielern") for i in range(4)]
gtia += [reg(f"p{i}pl", 0xD00C + i, "r", f"Kollision Spieler {i} mit Spielern") for i in range(4)]
gtia += [reg(f"trig{i}", 0xD010 + i, "r", f"Joystick-Feuerknopf {i}") for i in range(4)] + [reg("pal", 0xD014, "r", "PAL-/NTSC-Kennung (Bits 1-3: 1=PAL, 15=NTSC)")]
write("gtia", "GTIA", 0xD000, 0xD01F, gtia)

pokey = []
for i in range(4): pokey += [reg(f"audf{i+1}", 0xD200 + 2 * i, "w", f"Frequenz Kanal {i+1}"), reg(f"audc{i+1}", 0xD201 + 2 * i, "w", f"Verzerrung/Lautstärke Kanal {i+1}")]
pokey += [reg("audctl", 0xD208, "w", "Audio-Steuerung", [("POLY9", 7, None, "9-Bit statt 17-Bit-Polynom"), ("CH1_179", 6, None, "Kanal 1 mit 1,79 MHz"), ("CH3_179", 5, None, "Kanal 3 mit 1,79 MHz"), ("J12", 4, None, "Kanäle 1+2 als 16 Bit"), ("J34", 3, None, "Kanäle 3+4 als 16 Bit"), ("HP1", 2, None, "Hochpass Kanal 1 mit 3"), ("HP2", 1, None, "Hochpass Kanal 2 mit 4"), ("CLK15", 0, None, "15 kHz statt 64 kHz")]),
          reg("stimer", 0xD209, "w", "Timer starten"), reg("skrest", 0xD20A, "w", "Serielle Statusbits löschen"), reg("potgo", 0xD20B, "w", "Paddle-Abtastung starten"),
          reg("serout", 0xD20D, "w", "Serielle Ausgabe"), reg("irqen", 0xD20E, "w", "IRQ-Freigabe"), reg("skctl", 0xD20F, "w", "Serielle Steuerung")]
pokey += [reg(f"pot{i}", 0xD200 + i, "r", f"Paddle {i}") for i in range(8)]
pokey += [reg("allpot", 0xD208, "r", "Paddle-Status"), reg("kbcode", 0xD209, "r", "Tastaturcode"), reg("random", 0xD20A, "r", "Zufallszahl"),
          reg("serin", 0xD20D, "r", "Serielle Eingabe"), reg("irqst", 0xD20E, "r", "IRQ-Status"), reg("skstat", 0xD20F, "r", "Serieller Status")]
write("pokey", "POKEY", 0xD200, 0xD20F, pokey)

antic = [reg("dmactl", 0xD400, "w", "DMA-Steuerung", [("PFW", 0, 2, "Playfield-Breite (0=aus, 1=schmal, 2=normal, 3=breit)"), ("MISSILE", 2, None, "Missile-DMA"), ("PLAYER", 3, None, "Spieler-DMA"), ("ONELINE", 4, None, "Einzeilige PM-Auflösung"), ("DLIST", 5, None, "Display-List-DMA")]),
         reg("chactl", 0xD401, "w", "Zeichensatz-Steuerung"), reg("dlistl", 0xD402, "w", "Display-List-Adresse (Low)"), reg("dlisth", 0xD403, "w", "Display-List-Adresse (High)"),
         reg("hscrol", 0xD404, "w", "Horizontales Scrollen"), reg("vscrol", 0xD405, "w", "Vertikales Scrollen"), reg("pmbase", 0xD407, "w", "PMG-Basisadresse (High-Byte)"),
         reg("chbase", 0xD409, "w", "Zeichensatz-Basisadresse (High-Byte)"), reg("wsync", 0xD40A, "w", "Auf Zeilenende warten"),
         reg("nmien", 0xD40E, "w", "NMI-Freigabe", [("VBI", 6, None, "Vertical-Blank-NMI"), ("DLI", 7, None, "Display-List-NMI")]),
         reg("nmires", 0xD40F, "w", "NMI-Status löschen"), reg("vcount", 0xD40B, "r", "Vertikaler Zeilenzähler (Zeile / 2)"),
         reg("penh", 0xD40C, "r", "Lichtgriffel horizontal"), reg("penv", 0xD40D, "r", "Lichtgriffel vertikal"), reg("nmist", 0xD40F, "r", "NMI-Status")]
write("antic", "ANTIC", 0xD400, 0xD40F, antic)
write("pia", "PIA 6520", 0xD300, 0xD303, [reg("porta", 0xD300, "rw", "Port A (Joysticks 1+2)"), reg("portb", 0xD301, "rw", "Port B (XL/XE: Speicher-Banking)"),
      reg("pactl", 0xD302, "rw", "Port-A-Steuerung"), reg("pbctl", 0xD303, "rw", "Port-B-Steuerung")])
os_ = [("vdslst", 0x0200, "DLI-Vektor"), ("vvblki", 0x0222, "Immediate-VBI-Vektor"), ("vvblkd", 0x0224, "Deferred-VBI-Vektor"), ("vimirq", 0x0216, "IRQ-Vektor"),
       ("sdmctl", 0x022F, "DMACTL-Schatten"), ("sdlstl", 0x0230, "Display-List-Schatten (Low)"), ("sdlsth", 0x0231, "Display-List-Schatten (High)"),
       ("gprior", 0x026F, "PRIOR-Schatten"), ("pcolr0", 0x02C0, "COLPM0-Schatten"), ("pcolr1", 0x02C1, "COLPM1-Schatten"), ("pcolr2", 0x02C2, "COLPM2-Schatten"),
       ("pcolr3", 0x02C3, "COLPM3-Schatten"), ("color0", 0x02C4, "COLPF0-Schatten"), ("color1", 0x02C5, "COLPF1-Schatten"), ("color2", 0x02C6, "COLPF2-Schatten"),
       ("color3", 0x02C7, "COLPF3-Schatten"), ("color4", 0x02C8, "COLBK-Schatten"), ("runad", 0x02E0, "Startadresse (XEX)"), ("initad", 0x02E2, "Init-Adresse (XEX)"),
       ("memtop", 0x02E5, "Obergrenze des freien Speichers"), ("ch", 0x02FC, "Letzter Tastencode"), ("rtclok", 0x0012, "Systemuhr (3 Byte, 50/60 Hz)"),
       ("atract", 0x004D, "Attract-Modus-Zähler"), ("coldst", 0x0244, "Kaltstart-Flag"), ("ramtop", 0x006A, "Obergrenze RAM (Seiten)")]
write("os", "Atari-OS-Variablen", 0x0000, 0x02FF, [reg(n, a, "rw", d) for n, a, d in os_])

# ---------------------------------------------------------------- C64
vic = []
for i in range(8): vic += [reg(f"sp{i}x", 0xD000 + 2 * i, "rw", f"Sprite {i} X (Bit 0-7)"), reg(f"sp{i}y", 0xD001 + 2 * i, "rw", f"Sprite {i} Y")]
vic += [reg("msigx", 0xD010, "rw", "Sprite X-Bit 8"),
        reg("ctrl1", 0xD011, "rw", "Steuerregister 1", [("RST8", 7, None, "Rasterzeile Bit 8"), ("ECM", 6, None, "Extended Color Mode"), ("BMM", 5, None, "Bitmap-Modus"), ("DEN", 4, None, "Bildschirm an"), ("RSEL", 3, None, "25/24 Zeilen"), ("YSCROLL", 0, 3, "Vertikales Scrollen")]),
        reg("raster", 0xD012, "rw", "Rasterzeile (Bit 0-7) / Vergleichswert"), reg("lpx", 0xD013, "r", "Lichtgriffel X"), reg("lpy", 0xD014, "r", "Lichtgriffel Y"),
        reg("spena", 0xD015, "rw", "Sprites aktivieren"),
        reg("ctrl2", 0xD016, "rw", "Steuerregister 2", [("RES", 5, None, "Reset (ungenutzt)"), ("MCM", 4, None, "Multicolor-Modus"), ("CSEL", 3, None, "40/38 Spalten"), ("XSCROLL", 0, 3, "Horizontales Scrollen")]),
        reg("spye", 0xD017, "rw", "Sprites Y-Expansion"), reg("memptr", 0xD018, "rw", "Speicherzeiger (Screen/Zeichensatz)"),
        reg("irq", 0xD019, "rw", "Interrupt-Status", [("IRQ", 7, None, "Interrupt aufgetreten"), ("ILP", 3, None, "Lichtgriffel"), ("IMMC", 2, None, "Sprite-Sprite-Kollision"), ("IMBC", 1, None, "Sprite-Hintergrund-Kollision"), ("IRST", 0, None, "Rasterinterrupt")]),
        reg("irqen", 0xD01A, "rw", "Interrupt-Freigabe"), reg("sppri", 0xD01B, "rw", "Sprite-Priorität"), reg("spmc", 0xD01C, "rw", "Sprites Multicolor"),
        reg("spxe", 0xD01D, "rw", "Sprites X-Expansion"), reg("spspc", 0xD01E, "r", "Sprite-Sprite-Kollision"), reg("spbgc", 0xD01F, "r", "Sprite-Hintergrund-Kollision"),
        reg("border", 0xD020, "rw", "Rahmenfarbe")]
vic += [reg(f"bg{i}", 0xD021 + i, "rw", f"Hintergrundfarbe {i}") for i in range(4)] + [reg("spmc0", 0xD025, "rw", "Sprite-Multicolor 0"), reg("spmc1", 0xD026, "rw", "Sprite-Multicolor 1")]
vic += [reg(f"sp{i}col", 0xD027 + i, "rw", f"Sprite {i} Farbe") for i in range(8)]
write("vic2", "VIC-II", 0xD000, 0xD3FF, vic, {"mirror": "$40"})
sid = []
for v in range(3):
    b = 0xD400 + 7 * v
    sid += [reg(f"v{v+1}_freq_lo", b, "w", f"Stimme {v+1} Frequenz Low"), reg(f"v{v+1}_freq_hi", b + 1, "w", f"Stimme {v+1} Frequenz High"),
            reg(f"v{v+1}_pw_lo", b + 2, "w", f"Stimme {v+1} Pulsbreite Low"), reg(f"v{v+1}_pw_hi", b + 3, "w", f"Stimme {v+1} Pulsbreite High"),
            reg(f"v{v+1}_ctrl", b + 4, "w", f"Stimme {v+1} Steuerung (Wellenform, Gate)"), reg(f"v{v+1}_ad", b + 5, "w", f"Stimme {v+1} Attack/Decay"), reg(f"v{v+1}_sr", b + 6, "w", f"Stimme {v+1} Sustain/Release")]
sid += [reg("fc_lo", 0xD415, "w", "Filter-Grenzfrequenz Low"), reg("fc_hi", 0xD416, "w", "Filter-Grenzfrequenz High"), reg("res_filt", 0xD417, "w", "Resonanz/Filterzuordnung"),
        reg("mode_vol", 0xD418, "w", "Filtermodus und Lautstärke"), reg("potx", 0xD419, "r", "Paddle X"), reg("poty", 0xD41A, "r", "Paddle Y"),
        reg("osc3", 0xD41B, "r", "Oszillator 3 (Zufall)"), reg("env3", 0xD41C, "r", "Hüllkurve 3")]
write("sid", "SID", 0xD400, 0xD41F, sid, {"mirror": "$20"})
for n, base in (("cia1", 0xDC00), ("cia2", 0xDD00)):
    write(n, "CIA 6526 " + n[-1], base, base + 0x0F, [reg(x, base + i, "rw", d) for i, (x, d) in enumerate(
        [("pra", "Port A"), ("prb", "Port B"), ("ddra", "Datenrichtung A"), ("ddrb", "Datenrichtung B"), ("ta_lo", "Timer A Low"), ("ta_hi", "Timer A High"),
         ("tb_lo", "Timer B Low"), ("tb_hi", "Timer B High"), ("tod_10ths", "Uhr: Zehntelsekunden"), ("tod_sec", "Uhr: Sekunden"), ("tod_min", "Uhr: Minuten"),
         ("tod_hr", "Uhr: Stunden"), ("sdr", "Serielles Datenregister"), ("icr", "Interrupt-Steuerung"), ("cra", "Steuerregister A"), ("crb", "Steuerregister B")])])

# ---------------------------------------------------------------- VC-20 (VIC-I 6560/6561, 2 × VIA 6522)
vic1 = [reg("origin_x", 0x9000, "rw", "Horizontaler Bildursprung (Bit 0-6), Bit 7 = Interlace", [("INTERLACE", 7, None, "Interlace-Modus"), ("ORIGIN_X", 0, 7, "Bildursprung X in 4-Pixel-Schritten")]),
        reg("origin_y", 0x9001, "rw", "Vertikaler Bildursprung (in Rasterzeilen/2)"),
        reg("cols", 0x9002, "rw", "Spaltenzahl (Bit 0-6), Bit 7 = Bildschirmadresse Bit 9", [("SCRN_A9", 7, None, "Bildschirmspeicher-Adresse Bit 9"), ("COLUMNS", 0, 7, "Zeichen pro Zeile (Standard 22)")]),
        reg("rows", 0x9003, "rw", "Zeilenzahl (Bit 1-6), Bit 0 = 8×16-Zeichen, Bit 7 = Rasterzeile Bit 0", [("RASTER0", 7, None, "Rasterzeile Bit 0 (nur lesen)"), ("ROWS", 1, 6, "Zeichenzeilen (Standard 23)"), ("CHAR16", 0, None, "Zeichen 8×16 statt 8×8")]),
        reg("raster", 0x9004, "r", "Rasterzeile (Bit 8-1)"),
        reg("memptr", 0x9005, "rw", "Speicherzeiger: Bit 4-7 Bildschirm (A13-A10), Bit 0-3 Zeichensatz (A13-A10)", [("SCRN", 4, 4, "Bildschirmspeicher (A13-A10; A9 in $9002)"), ("CHAR", 0, 4, "Zeichensatz (0=$8000 ROM, 8=$0000 RAM, 12=$1000 RAM …)")]),
        reg("lpx", 0x9006, "r", "Lichtgriffel X"), reg("lpy", 0x9007, "r", "Lichtgriffel Y"),
        reg("potx", 0x9008, "r", "Paddle X"), reg("poty", 0x9009, "r", "Paddle Y"),
        reg("bass", 0x900A, "rw", "Bassstimme: Tonhöhe (Bit 0-6), Bit 7 = ein", [("ON", 7, None, "Stimme eingeschaltet")]),
        reg("alto", 0x900B, "rw", "Altstimme: Tonhöhe (Bit 0-6), Bit 7 = ein", [("ON", 7, None, "Stimme eingeschaltet")]),
        reg("soprano", 0x900C, "rw", "Sopranstimme: Tonhöhe (Bit 0-6), Bit 7 = ein", [("ON", 7, None, "Stimme eingeschaltet")]),
        reg("noise", 0x900D, "rw", "Rauschen: Frequenz (Bit 0-6), Bit 7 = ein", [("ON", 7, None, "Rauschen eingeschaltet")]),
        reg("volume", 0x900E, "rw", "Lautstärke (Bit 0-3), Hilfsfarbe (Bit 4-7)", [("AUX", 4, 4, "Hilfsfarbe für Multicolor"), ("VOL", 0, 4, "Lautstärke 0-15")]),
        reg("color", 0x900F, "rw", "Hintergrund (Bit 4-7), Normal/Invers (Bit 3), Rahmen (Bit 0-2)", [("BG", 4, 4, "Hintergrundfarbe 0-15"), ("NORMAL", 3, None, "1 = normal, 0 = invers"), ("BORDER", 0, 3, "Rahmenfarbe 0-7")])]
write("vic1", "VIC 6560/6561 (VIC-I)", 0x9000, 0x900F, vic1)
via_names = [("orb", "Port B"), ("ora", "Port A"), ("ddrb", "Datenrichtung B"), ("ddra", "Datenrichtung A"), ("t1c_lo", "Timer 1 Zähler Low"), ("t1c_hi", "Timer 1 Zähler High"),
             ("t1l_lo", "Timer 1 Latch Low"), ("t1l_hi", "Timer 1 Latch High"), ("t2c_lo", "Timer 2 Zähler Low"), ("t2c_hi", "Timer 2 Zähler High"), ("sr", "Schieberegister"),
             ("acr", "Hilfssteuerregister"), ("pcr", "Peripherie-Steuerregister"), ("ifr", "Interrupt-Flags"), ("ier", "Interrupt-Freigabe"), ("ora_nh", "Port A (ohne Handshake)")]
via_doc = {0x9110: {"orb": "Port B: Userport", "ora": "Port A: Joystick, Kassettentaste, serielle Leitungen"},
           0x9120: {"orb": "Port B: Tastaturspalten, Joystick rechts", "ora": "Port A: Tastaturzeilen"}}
for n, base, d in (("via1", 0x9110, "VIA 6522 #1 (NMI, Userport, Joystick, serieller Bus)"), ("via2", 0x9120, "VIA 6522 #2 (IRQ-Timer, Tastatur)")):
    write(n, d, base, base + 0x0F, [reg(x, base + i, "rw", via_doc[base].get(x, t)) for i, (x, t) in enumerate(via_names)])
print("ok", sorted(os.listdir(OUT)))
