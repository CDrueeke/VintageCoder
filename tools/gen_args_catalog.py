#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/args/*.json (Katalog der Emulator-Kommandozeilenparameter).

Die erzeugten Dateien sind die Quelle der Wahrheit zur Laufzeit; dieses Skript hält nur die vielen gleichartigen VICE-Einträge
an einer Stelle zusammen. Die Parameter stammen aus den Hilfetexten der Programme (`x64sc -help`, `atari800 -help`, `xmega65 -h`),
geprüft mit tools/check_args_catalog.py. Aufruf: python3 tools/gen_args_catalog.py
"""
import argparse, json, os, sys
_ap = argparse.ArgumentParser(
    description="""Erzeugt data/args/*.json (Katalog der Emulator-Kommandozeilenparameter: VICE x64sc/x128/xplus4/xvic, atari800, xemu und der eigene Assembler vcc asm).

Die erzeugten Dateien sind die Quelle der Wahrheit zur Laufzeit; dieses Skript hält nur die vielen gleichartigen VICE-Einträge an einer Stelle zusammen. Es liest nichts
und schreibt je Programm eine Datei data/args/<programm>.json (UTF-8; vorhandene werden überschrieben; der Ordner wird relativ zum Ort des Skripts bestimmt, der
Arbeitsordner ist egal). Die Parameter stammen aus den Hilfetexten der Programme; Prüfung mit tools/check_args_catalog.py.""",
    epilog="""Beispiele:
  python3 tools/gen_args_catalog.py
  cd tools && python3 gen_args_catalog.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "args")


def sw(id, group, flag, label, hint, off=None, default=None, conflicts=(), requires=()):
    p = dict(id=id, group=group, type="switch", flag=flag, label=label, hint=hint)
    if off: p["off_flag"] = off
    if default is not None: p["default"] = default
    if conflicts: p["conflicts"] = list(conflicts)
    if requires: p["requires"] = list(requires)
    return p


def ch(id, group, flag, label, hint, choices, default=None, conflicts=(), requires=()):
    p = dict(id=id, group=group, type="choice", flag=flag, label=label, hint=hint,
             choices=[dict(value=v, label=l) for v, l in choices])
    if default is not None: p["default"] = default
    if conflicts: p["conflicts"] = list(conflicts)
    if requires: p["requires"] = list(requires)
    return p


def tx(id, group, flag, label, hint, default=None, example=None, conflicts=(), requires=(), type="text", min=None, max=None):
    p = dict(id=id, group=group, type=type, flag=flag, label=label, hint=hint)
    if default is not None: p["default"] = default
    if example: p["example"] = example
    if min is not None: p["min"] = min
    if max is not None: p["max"] = max
    if conflicts: p["conflicts"] = list(conflicts)
    if requires: p["requires"] = list(requires)
    return p


def nu(id, group, flag, label, hint, **kw): return tx(id, group, flag, label, hint, type="number", **kw)
def fi(id, group, flag, label, hint, **kw): return tx(id, group, flag, label, hint, type="file", **kw)


# Schalter, die der Debug-Start selbst setzt (beim Debuggen fallen Angaben des Benutzers dazu weg); " <value>" = Schalter mit Wert
DEBUG_RESERVED = {
    "vice": ["-binarymonitor", "+binarymonitor", "-binarymonitoraddress <value>", "-confirmonexit", "+confirmonexit", "-autostart <value>", "-autoload <value>", "-cartcrt <value>"],
    "atari800": ["-monitor", "-no-autosave-config", "-autosave-config", "-run <value>", "-cart <value>"],
    "xemu": ["-uartmon <value>", "-besure", "-rom <value>"],
}


def write(name, doc):
    key = "vice" if name.startswith("vice-") else name
    if key in DEBUG_RESERVED: doc["debug_reserved"] = DEBUG_RESERVED[key]
    path = os.path.join(OUT, name + ".json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(doc, f, ensure_ascii=False, indent=1); f.write("\n")
    print(path)


VICE_GROUPS = [("run", "Ablauf"), ("video", "Bild"), ("sound", "Ton"), ("drives", "Laufwerk und Start von Datei"), ("input", "Eingabe"), ("debug", "Monitor und Fehlersuche"), ("logging", "Protokoll und Einstellungen")]
VICE_VERIFIED = "x64sc/x128/xplus4/xvic -help, VICE 3.10 (gtk3, macOS arm64)"
VICE_RESERVED = ["-autostart", "-cartcrt", "-cartA", "-cart2", "-cart4", "-cart6", "-cartB", "-memory", "-8", "-binarymonitor", "-binarymonitoraddress", "+confirmonexit"]


def vice_common(video):
    """gemeinsame Parameter; `video` = Präfix des Videochips (VICII, TED, VIC)"""
    p = []
    p += [
        nu("speed", "run", "-speed", "Geschwindigkeit in Prozent", "Begrenzt die Emulation auf diesen Wert (100 = Originaltempo). Eine negative Zahl begrenzt auf so viele Bilder pro Sekunde.", default="100", example="200", conflicts=["warp"]),
        sw("warp", "run", "-warp", "Warp-Modus beim Start", "Startet im Warp-Modus: die Emulation läuft so schnell wie möglich.", off="+warp", default="aus", conflicts=["speed"]),
        sw("autostart_warp", "run", "-autostart-warp", "Warp beim Autostart", "Schaltet den Warp-Modus nur während des Autostarts ein, bis das Programm geladen ist.", off="+autostart-warp"),
        sw("pauseonsettings", "run", "-pauseonsettings", "Pause im Einstellungsdialog", "Hält die Emulation an, solange der Einstellungsdialog von VICE offen ist.", off="+pauseonsettings"),
        sw("confirmonexit", "run", "-confirmonexit", "Beenden bestätigen", "Fragt vor dem Beenden von VICE nach.", off="+confirmonexit"),
        sw("maximized", "run", "-maximized", "Maximiert starten", "Öffnet das Fenster maximiert.", off="+maximized"),
        sw("minimized", "run", "-minimized", "Minimiert starten", "Öffnet das Fenster minimiert.", off="+minimized"),
        ch("jamaction", "run", "-jamaction", "Aktion bei CPU-JAM", "Was VICE tut, wenn die CPU an einem ungültigen Opcode hängen bleibt.", [("0", "Nachfragen"), ("1", "Weiterlaufen"), ("2", "Monitor öffnen"), ("3", "Reset"), ("4", "Neustart (Power Cycle)"), ("5", "Emulator beenden")], default="0"),
        nu("limitcycles", "run", "-limitcycles", "Zyklenlimit", "Beendet VICE mit Fehler, wenn nach so vielen Zyklen noch nicht Schluss ist (für automatische Tests).", min=1),
        nu("ramstart", "run", "-raminitstartvalue", "Anfangswert des RAM", "Wert der allerersten RAM-Adresse nach dem Einschalten; der Rest folgt dem Muster von VICE.", min=0, max=255, example="0"),
        nu("seed", "run", "-seed", "Zufallszahl-Startwert", "Fester Startwert des Zufallsgenerators, damit Läufe reproduzierbar sind (zur Fehlersuche).", min=0),
        sw("saveres", "logging", "-saveres", "Einstellungen beim Beenden speichern", "VICE schreibt seine Einstellungen beim Beenden in die Datei vicerc.", off="+saveres"),
        fi("config", "logging", "-config", "Konfigurationsdatei", "Verwendet diese Konfigurationsdatei von VICE statt der Standarddatei.", example="{project_dir}/vice.cfg"),
        fi("addconfig", "logging", "-addconfig", "Zusatz-Konfiguration", "Lädt diese Datei zusätzlich zur Konfiguration (nur Ressourcen, die dort stehen).", example="{project_dir}/extra.cfg"),
        sw("default", "logging", "-default", "Standardeinstellungen laden", "Setzt alle Einstellungen von VICE auf die Werkseinstellung zurück, bevor weitere Parameter wirken."),
        sw("silent", "logging", "-silent", "Keine Ausgabe", "Unterdrückt alle Meldungen außer Fehlern.", conflicts=["verbose", "debug"]),
        sw("verbose", "logging", "-verbose", "Ausführliche Ausgabe", "Gibt mehr Meldungen aus.", conflicts=["silent"]),
        sw("debug", "logging", "-debug", "Debug-Ausgabe", "Gibt ausführliche Fehlersuch-Meldungen aus.", conflicts=["silent"]),
        fi("logfile", "logging", "-logfile", "Protokolldatei", "Name der Protokolldatei; mit „In Datei protokollieren“ wird sie geschrieben.", example="{project_dir}/vice.log"),
        sw("logtofile", "logging", "-logtofile", "In Datei protokollieren", "Schreibt das Protokoll in die Protokolldatei.", off="+logtofile"),
        sw("logtostdout", "logging", "-logtostdout", "Protokoll auf die Standardausgabe", "Gibt das Protokoll auf stdout aus.", off="+logtostdout"),
        fi("exitscreenshot", "logging", "-exitscreenshot", "Bildschirmfoto beim Beenden", "Speichert beim Beenden ein Bildschirmfoto unter diesem Namen (nützlich für automatische Tests).", example="{project_dir}/build/{project}.png"),
    ]
    p += [
        sw("sound", "sound", "-sound", "Ton an", "Schaltet die Tonausgabe ein oder aus.", off="+sound", default="an"),
        nu("soundrate", "sound", "-soundrate", "Abtastrate (Hz)", "Abtastrate der Tonausgabe in Hertz.", example="48000", min=8000, max=192000),
        nu("soundbufsize", "sound", "-soundbufsize", "Tonpuffer (ms)", "Größe des Tonpuffers in Millisekunden; größer vermeidet Knacken, kleiner verkürzt die Verzögerung.", example="100", min=1),
        ch("soundoutput", "sound", "-soundoutput", "Ausgabeart", "Mono oder Stereo.", [("0", "System entscheidet"), ("1", "immer Mono"), ("2", "immer Stereo")]),
        ch("sidengine", "sound", "-sidengine", "SID-Engine", "Wie der SID berechnet wird: FastSID ist schnell, ReSID genau.", [("0", "FastSID"), ("1", "ReSID")], conflicts=["sidenginemodel"]),
        ch("sidenginemodel", "sound", "-sidenginemodel", "SID-Engine und Modell", "Engine und Chipmodell in einem Schritt.",
           [("0", "FastSID 6581"), ("1", "FastSID 8580"), ("256", "ReSID 6581"), ("257", "ReSID 8580"), ("258", "ReSID 8580 mit Digiboost")], conflicts=["sidengine"]),
        sw("drivesound", "drives", "-drivesound", "Laufwerksgeräusche", "Spielt die Geräusche des Diskettenlaufwerks ab.", off="+drivesound"),
    ]
    p += [
        sw("trapdevice8", "drives", "-trapdevice8", "Schnelles Laden (Traps)", "Beschleunigt Laden und Speichern, indem die Kernal-Routinen abgefangen werden (virtuelles Laufwerk).", off="+trapdevice8"),
        sw("basicload", "drives", "-basicload", "Autostart: an den BASIC-Anfang laden", "Lädt Dateien von Diskette beim Autostart an den BASIC-Anfang (ohne „,1“).", off="+basicload"),
        sw("autostartwithcolon", "drives", "-autostartwithcolon", "Autostart mit „RUN:“", "Verwendet beim Autostart „RUN:“ mit Doppelpunkt statt „RUN“.", off="+autostartwithcolon"),
        sw("autostart_tde", "drives", "-autostart-handle-tde", "Autostart: Laufwerkemulation anpassen", "Schaltet beim Autostart die genaue Laufwerkemulation (True Drive Emulation) passend zum Programm.", off="+autostart-handle-tde"),
        ch("autostartprgmode", "drives", "-autostartprgmode", "Autostart-Art für PRG", "Wie ein PRG beim Autostart in den Speicher kommt.", [("0", "virtuelles Dateisystem"), ("1", "direkt einspielen"), ("2", "Diskimage")]),
        nu("autostart_delay", "drives", "-autostart-delay", "Autostart-Verzögerung (s)", "Wartezeit vor dem Autostart in Sekunden; 0 = Standard.", min=0),
        fi("disk9", "drives", "-9", "Diskimage in Laufwerk 9", "Legt dieses Image in Laufwerk 9 ein.", example="{disk}"),
        fi("tape", "drives", "-1", "Band (Datasette)", "Legt dieses Bandimage in die Datasette ein."),
    ]
    p += [
        ch("joydev1", "input", "-joydev1", "Joystick, Port 1" if video != "VIC" else "Joystick", "Welches Gerät den Joystickport bedient.", [("0", "keines"), ("1", "Ziffernblock"), ("2", "Tastensatz 1"), ("3", "Tastensatz 2")]),
        ch("keymap", "input", "-keymap", "Tastaturbelegung", "Welche Tastaturtabelle VICE benutzt.", [("0", "symbolisch"), ("1", "positionsgetreu"), ("2", "symbolisch (eigene)"), ("3", "positionsgetreu (eigene)")]),
        sw("mouse", "input", "-mouse", "Maus einfangen", "Fängt die Maus im Fenster ein.", off="+mouse"),
        tx("keybuf", "input", "-keybuf", "Tastaturpuffer füllen", "Schreibt diese Zeichen beim Start in den Tastaturpuffer, z. B. einen Befehl, der das Programm startet.", example='RUN:\\n'),
    ]
    p += [
        fi("moncommands", "debug", "-moncommands", "Monitor-Befehle aus Datei", "Führt beim Start die Befehle dieser Datei im VICE-Monitor aus (z. B. Haltepunkte oder geladene Labels).", example="{symbols}"),
        tx("initbreak", "debug", "-initbreak", "Anfänglicher Haltepunkt", "Hält den Monitor an: bei einer Adresse, bei „ready“ (BASIC-Eingabe) oder bei „reset“.", example="0x{start}", requires=[]),
        sw("remotemonitor", "debug", "-remotemonitor", "Textmonitor im Netz", "Öffnet den Monitor von VICE als Netzwerkdienst (Standard 127.0.0.1:6510).", off="+remotemonitor"),
        tx("remotemonitoraddress", "debug", "-remotemonitoraddress", "Adresse des Textmonitors", "Lokale Adresse, an die der Textmonitor gebunden wird.", example="ip4://127.0.0.1:6510", requires=["remotemonitor"]),
        sw("nativemonitor", "debug", "-nativemonitor", "Monitor im Terminal", "Verwendet den Monitor im Betriebssystem-Terminal statt im Fenster.", off="+nativemonitor"),
    ]
    return p


def vice(prog, kind, label, video, extra_params):
    params = vice_common(video)
    groups = list(VICE_GROUPS)
    params = extra_params + params
    cat = dict(program=prog, label=label, kinds=[kind], verified=VICE_VERIFIED, groups=[dict(id=i, label=l) for i, l in groups], reserved=VICE_RESERVED, params=params)
    return cat


def vic_filter(flag_prefix, chip):
    return [
        ch("filter", "video", "-%sfilter" % flag_prefix, "Darstellungsfilter", "Wie das Bild skaliert wird: ohne Filter, mit CRT-Nachbildung oder Scale2x.", [("0", "keiner"), ("1", "CRT-Emulation"), ("2", "Scale2x")]),
        ch("borders", "video", "-%sborders" % flag_prefix, "Rahmen", "Wie viel vom Bildschirmrand sichtbar ist.", [("0", "normal"), ("1", "voll"), ("2", "Debug (Raster sichtbar)"), ("3", "ohne Rahmen")]),
    ]


def c64_family_video():
    return vic_filter("VICII", "VIC-II") + [
        sw("dsize", "video", "-VICIIdsize", "Doppelte Größe", "Zeichnet das Bild doppelt so groß.", off="+VICIIdsize"),
        sw("dscan", "video", "-VICIIdscan", "Doppelte Zeilen (Scanlines)", "Zeichnet jede Rasterzeile doppelt.", off="+VICIIdscan"),
        sw("vsync", "video", "-VICIIvsync", "Vertikale Synchronisation", "Gleicht die Ausgabe mit dem Bildwechsel des Monitors ab und verhindert Zerreißen.", off="+VICIIvsync"),
        ch("aspectmode", "video", "-VICIIaspectmode", "Seitenverhältnis", "Wie das Seitenverhältnis des Bildes bestimmt wird.", [("0", "aus"), ("1", "eigenes"), ("2", "wahrheitsgetreu")]),
        nu("aspect", "video", "-VICIIaspect", "Eigenes Seitenverhältnis", "Verhältnis zwischen 0,5 und 2,0 (nur mit „eigenes“).", example="1.0", min=0.5, max=2.0, requires=["aspectmode"]),
        sw("audioleak", "sound", "-VICIIaudioleak", "Audio-Übersprechen", "Bildet das Übersprechen des VIC-II auf den Ton nach.", off="+VICIIaudioleak"),
    ]


def sid_extra():
    return [
        ch("sidextra", "sound", "-sidextra", "Zusätzliche SIDs", "Anzahl weiterer SID-Chips (0 bis 3).", [("0", "keine"), ("1", "1"), ("2", "2"), ("3", "3")]),
        tx("sid2address", "sound", "-sid2address", "Adresse des 2. SID", "Basisadresse des zweiten SID, z. B. 0xD420.", example="0xD420", requires=["sidextra"]),
        tx("sid3address", "sound", "-sid3address", "Adresse des 3. SID", "Basisadresse des dritten SID, z. B. 0xD440.", example="0xD440", requires=["sidextra"]),
    ]


def reu():
    return [
        sw("reu", "drives", "-reu", "RAM-Erweiterung (REU)", "Schaltet die RAM Expansion Unit ein.", off="+reu"),
        ch("reusize", "drives", "-reusize", "REU-Größe", "Größe der RAM-Erweiterung.", [(str(k), "%d KB" % k) for k in (128, 256, 512, 1024, 2048, 4096, 8192, 16384)], requires=["reu"]),
        fi("cartcrt", "drives", "-cartcrt", "Modul (CRT)", "Steckt ein Modulabbild im CRT-Format ein.", example="{output}"),
    ]


def drive_param():
    return ch("drive8type", "drives", "-drive8type", "Laufwerkstyp (Unit 8)", "Welches Diskettenlaufwerk als Unit 8 nachgebildet wird.",
              [("0", "kein Laufwerk"), ("1540", "CBM 1540"), ("1541", "CBM 1541"), ("1542", "CBM 1541-II"), ("1570", "CBM 1570"), ("1571", "CBM 1571"), ("1581", "CBM 1581")])


def norm_switch():
    return [
        sw("pal", "video", "-pal", "PAL-Takt", "Verwendet den PAL-Synchronfaktor.", conflicts=["ntsc"]),
        sw("ntsc", "video", "-ntsc", "NTSC-Takt", "Verwendet den NTSC-Synchronfaktor.", conflicts=["pal"]),
    ]


# --- VICE ---
x64sc = vice("vice-x64sc", "x64sc", "VICE x64sc (C64)", "VICII",
             c64_family_video() + norm_switch() + sid_extra() + reu() + [drive_param(),
             ch("model", "run", "-model", "C64-Modell", "Welches C64-Modell nachgebildet wird (Videonorm, VIC-II und SID gehören dazu).",
                [("c64", "C64 (PAL, alt)"), ("c64c", "C64C (PAL)"), ("c64old", "C64 alt (PAL)"), ("ntsc", "C64 (NTSC)"), ("newntsc", "C64C (NTSC)"), ("oldntsc", "C64 alt (NTSC)"), ("drean", "Drean (PAL-N)"), ("jap", "Japan"), ("c64gs", "C64 GS"), ("pet64", "PET 64"), ("ultimax", "Ultimax")]),
             ch("viciimodel", "video", "-VICIImodel", "VIC-II-Modell", "Chipmodell des VIC-II.", [("6569", "6569 (PAL)"), ("6569r1", "6569 R1 (PAL)"), ("8565", "8565 (PAL, neu)"), ("6567", "6567 (NTSC)"), ("8562", "8562 (NTSC, neu)"), ("6567r56a", "6567 R56A (NTSC, alt)"), ("6572", "6572 (PAL-N)")]),
             ])
x128 = vice("vice-x128", "x128", "VICE x128 (C128)", "VICII",
            c64_family_video() + norm_switch() + sid_extra() + reu() + [drive_param(),
            ch("model", "run", "-model", "C128-Modell", "Welches C128-Modell nachgebildet wird.", [("c128", "C128 (PAL)"), ("c128dcr", "C128DCR (PAL)"), ("pal", "PAL"), ("ntsc", "NTSC")]),
            ch("machinetype", "run", "-machinetype", "Ländervariante", "Tastatur und Zeichensatz der Ländervariante.", [("0", "International"), ("1", "Finnisch"), ("2", "Französisch"), ("3", "Deutsch"), ("4", "Italienisch"), ("5", "Norwegisch"), ("6", "Schwedisch"), ("7", "Schweizer")]),
            sw("go64", "run", "-go64", "Immer im C64-Modus starten", "Wechselt nach jedem Reset in den C64-Modus.", off="+go64"),
            sw("c128fullbanks", "run", "-c128fullbanks", "RAM-Bänke 2 und 3", "Schaltet die RAM-Bänke 2 und 3 ein.", off="+c128fullbanks"),
            sw("col40", "video", "-40col", "40 Spalten", "Startet mit der 40-Zeichen-Anzeige.", conflicts=["col80"]),
            sw("col80", "video", "-80col", "80 Spalten", "Startet mit der 80-Zeichen-Anzeige (VDC).", conflicts=["col40"]),
            ])
xplus4 = vice("vice-xplus4", "xplus4", "VICE xplus4 (Plus/4)", "TED",
              vic_filter("TED", "TED") + norm_switch() + [drive_param(),
              ch("model", "run", "-model", "Plus/4-Modell", "Welches Modell der 264er-Reihe nachgebildet wird.", [("c16", "C16"), ("c16pal", "C16 (PAL)"), ("c16ntsc", "C16 (NTSC)"), ("plus4", "Plus/4"), ("plus4pal", "Plus/4 (PAL)"), ("plus4ntsc", "Plus/4 (NTSC)"), ("v364", "V364"), ("cv364", "CV364"), ("c232", "C232")]),
              fi("cartcrt", "drives", "-cartcrt", "Modul (CRT)", "Steckt ein Modulabbild im CRT-Format ein.", example="{output}"),
              ])
xvic = vice("vice-xvic", "xvic", "VICE xvic (VC-20)", "VIC",
            vic_filter("VIC", "VIC") + norm_switch() + [drive_param(),
            ch("model", "run", "-model", "VC-20-Modell", "Welches Modell nachgebildet wird.", [("vic20", "VC-20"), ("vic20pal", "VC-20 (PAL)"), ("vic20ntsc", "VC-20 (NTSC)"), ("vic21", "VIC-21")]),
            fi("cartA", "drives", "-cartA", "Modulabbild bei $A000", "4 oder 8 KB ROM-Abbild im Bereich $A000.", example="{output}"),
            fi("cart2", "drives", "-cart2", "Modulabbild bei $2000", "4, 8 oder 16 KB ROM-Abbild im Bereich $2000.", example="{output}"),
            fi("cartcrt", "drives", "-cartcrt", "Modul (CRT)", "Steckt ein Modulabbild im CRT-Format ein.", example="{output}"),
            ])
for c in (x64sc, x128, xplus4, xvic):
    # Plus/4 und VC-20 haben keinen SID-Chip mit Auswahl; der VC-20-Speicherausbau kommt aus den Projekteinstellungen (-memory)
    if c["program"] in ("vice-xplus4", "vice-xvic"):
        c["params"] = [p for p in c["params"] if p["id"] not in ("sidengine", "sidenginemodel", "sidextra", "sid2address", "sid3address", "audioleak")]
    # Doppelte Kennungen (z. B. cartcrt) entfernen, das erste gilt
    seen, out = set(), []
    for p in c["params"]:
        if p["id"] in seen: continue
        seen.add(p["id"]); out.append(p)
    c["params"] = out
    write(c["program"], c)

# --- atari800 ---
A8 = [("run", "Ablauf"), ("system", "Maschine und Speicher"), ("video", "Bild"), ("sound", "Ton"), ("input", "Eingabe"), ("debug", "Monitor und Fehlersuche"), ("drives", "Dateien und Laufwerke")]
atari800 = dict(program="atari800", label="atari800 (Atari 8 Bit)", kinds=["atari800"], verified="atari800 -help, Version 7.2.1 (Homebrew, macOS)",
    groups=[dict(id=i, label=l) for i, l in A8], reserved=["-xl", "-pal", "-ntsc", "-nobasic", "-basic", "-run", "-cart", "-xlxe_rom", "-no-autosave-config", "-monitor"], params=[])
atari800["params"] = [
    sw("turbo", "run", "-turbo", "Turbo", "Lässt den Atari so schnell wie möglich laufen."),
    nu("refresh", "run", "-refresh", "Bildwiederholrate", "Zeigt nur jedes n-te Bild: 1 = jedes Bild, 2 = jedes zweite usw.", min=1, default="1"),
    sw("autosave", "run", "-autosave-config", "Einstellungen beim Beenden speichern", "Schreibt die Einstellungen beim Beenden in die Konfigurationsdatei.", off="-no-autosave-config"),
    fi("config", "run", "-config", "Konfigurationsdatei", "Verwendet diese Konfigurationsdatei statt ~/.atari800.cfg.", example="{project_dir}/atari800.cfg"),
    sw("mapram", "system", "-mapram", "MapRAM (XL/XE)", "Schaltet den RAM unter dem ROM-Bereich (MapRAM) für XL/XE ein.", off="-no-mapram"),
    sw("stereo", "sound", "-stereo", "Zwei POKEYs (Stereo)", "Bildet zwei POKEY-Chips nach.", off="-nostereo"),
    sw("sound", "sound", "-sound", "Ton an", "Schaltet die Tonausgabe ein oder aus.", off="-nosound", default="an"),
    nu("volume", "sound", "-volume", "Lautstärke (0 bis 100)", "Lautstärke der Tonausgabe.", example="80", min=0, max=100),
    nu("dsprate", "sound", "-dsprate", "Abtastrate (Hz)", "Abtastrate der Tonausgabe in Hertz.", example="44100", min=8000),
    nu("snddelay", "sound", "-snddelay", "Tonverzögerung (ms)", "Zielverzögerung des Tons in Millisekunden.", example="40", min=0),
    sw("fullscreen", "video", "-fullscreen", "Vollbild", "Startet im Vollbild.", off="-windowed", conflicts=[]),
    ch("artif", "video", "-artif", "Artefaktfarben", "Modus der Farbartefakte in Hochauflösung (0 = aus).", [("0", "aus"), ("1", "1"), ("2", "2"), ("3", "3"), ("4", "4")]),
    ch("ntsc_artif", "video", "-ntsc-artif", "NTSC-Artefakte", "Art der Videoartefakte bei NTSC.", [("none", "keine"), ("ntsc-old", "alt"), ("ntsc-new", "neu"), ("ntsc-full", "voll")]),
    ch("pal_artif", "video", "-pal-artif", "PAL-Artefakte", "Art der Videoartefakte bei PAL.", [("none", "keine"), ("pal-simple", "einfach"), ("pal-blend", "Mischung")]),
    ch("colors_preset", "video", "-colors-preset", "Farbvoreinstellung", "Vorgegebene Farbanpassung.", [("standard", "Standard"), ("deep-black", "Tiefschwarz"), ("vibrant", "Lebhaft")]),
    ch("stretch", "video", "-stretch", "Skalierung", "Wie das Bild gestreckt wird.", [("none", "keine"), ("integral", "ganzzahlig"), ("full", "voll")]),
    ch("image_aspect", "video", "-image-aspect", "Seitenverhältnis", "Seitenverhältnis des Bildes.", [("none", "keines"), ("square-pixels", "quadratische Pixel"), ("real", "wie am Fernseher")]),
    sw("vsync", "video", "-vsync", "Vertikale Synchronisation", "Gleicht die Ausgabe mit dem Bildwechsel des Monitors ab.", off="-no-vsync"),
    sw("showspeed", "video", "-showspeed", "Geschwindigkeit anzeigen", "Zeigt die tatsächliche Geschwindigkeit in Prozent an."),
    nu("win_width", "video", "-win-width", "Fensterbreite", "Breite des Fensters in Pixeln.", min=1),
    nu("win_height", "video", "-win-height", "Fensterhöhe", "Höhe des Fensters in Pixeln.", min=1),
    ch("mouse", "input", "-mouse", "Maus", "Welches Gerät die Maus nachbildet.", [("off", "keine"), ("pad", "Paddles"), ("touch", "Touch Tablet"), ("koala", "Koala Pad"), ("pen", "Lichtgriffel"), ("gun", "Lightgun"), ("amiga", "Amiga-Maus"), ("st", "Atari-ST-Maus"), ("trak", "Trak-Ball"), ("joy", "Joystick")]),
    sw("nojoystick", "input", "-nojoystick", "Joystick aus", "Schaltet die Joystickunterstützung ab.", conflicts=["kbdjoy0", "kbdjoy1"]),
    sw("kbdjoy0", "input", "-kbdjoy0", "Tastatur als Joystick 0", "Die Tastatur bedient Joystickport 0.", off="-no-kbdjoy0"),
    sw("kbdjoy1", "input", "-kbdjoy1", "Tastatur als Joystick 1", "Die Tastatur bedient Joystickport 1.", off="-no-kbdjoy1"),
    sw("grabmouse", "input", "-grabmouse", "Maus einfangen", "Hält den Mauszeiger im Fenster."),
    sw("bbrk", "debug", "-bbrk", "Halt bei BRK", "Hält den Monitor an, wenn ein BRK-Befehl ausgeführt wird."),
    tx("bpc", "debug", "-bpc", "Haltepunkt bei Adresse", "Hält den Monitor an, wenn der Programmzähler diese Adresse (hexadezimal) erreicht.", example="{start}"),
    fi("label_file", "debug", "-label-file", "Label-Datei für den Monitor", "Lädt Labels aus dieser Datei in den Monitor des atari800.", example="{symbols}"),
    sw("monitor", "debug", "-monitor", "Im Monitor starten", "Startet den Emulator direkt im Monitor (Textkonsole).", conflicts=[]),
    fi("record", "debug", "-record", "Eingaben aufzeichnen", "Schreibt alle Eingaben in diese Datei.", example="{project_dir}/build/{project}.rec"),
    fi("playback", "debug", "-playback", "Eingaben abspielen", "Spielt zuvor aufgezeichnete Eingaben aus dieser Datei ab.", example="{project_dir}/build/{project}.rec"),
    fi("screenshots", "debug", "-screenshots", "Muster für Bildschirmfotos", "Dateiname bzw. Muster für Bildschirmfotos.", example="{project_dir}/build/{project}"),
    fi("tape", "drives", "-tape", "Kassettenimage", "Legt dieses Kassettenimage ein.", example=""),
    fi("boottape", "drives", "-boottape", "Kassette einlegen und booten", "Legt das Kassettenimage ein und bootet davon."),
    fi("h1", "drives", "-H1", "Ordner für H1:", "Ordner des Hosts, der als Gerät H1: erscheint.", example="{project_dir}"),
    fi("hpath", "drives", "-Hpath", "Ordner für H:-Programme", "Ordner für Atari-Programme auf dem Host-Gerät H:.", example="{project_dir}/build"),
    sw("hreadonly", "drives", "-hreadonly", "H: schreibgeschützt", "Das Host-Gerät H: ist schreibgeschützt.", off="-hreadwrite"),
]
for p in atari800["params"]:
    if p["id"] == "tape": p.pop("example", None)
write("atari800", atari800)

# --- Altirra (Windows) ---
# Quelle: Altirra 4.01.200, `Altirra64.exe /?` (Windows-Sitzung). Schalter beginnen mit „/“; Werte stehen teils im selben Wort (`/kernel:xl`, join=":"),
# teils im nächsten Argument (`/cartmapper 1`). Funktional ausprobiert (Start über F5) sind bisher nur /run, /disk, /pal, /tempprofile und /defprofile:xl,
# also die von VintageCoder selbst gesetzten; alle übrigen Einträge stehen nur in der Hilfe von Altirra.
# Nicht aufgenommen, weil VintageCoder sie selbst setzt: /pal /ntsc /nobasic /basic /run /runbas /cart /disk /tempprofile /defprofile.
# Windows-Prüfung (Altirra 4.01, Start mit /portabletemp /skipsetup, Titelleiste und Fehlerfenster ausgewertet, dazu die Altirra-Hilfe):
# Nicht aufgenommen, weil sie mit Schaltern kollidieren, die VintageCoder selbst setzt:
#   /secam /ntsc50 /pal60 -- zusammen mit /pal oder /ntsc (jede Reihenfolge) zeigt Altirra „Altirra Error“; nur eines der fünf Bildnormschalter ist erlaubt.
#   /autoprofile /launch -- überschreiben /defprofile und /pal bzw. /ntsc: `/defprofile:800 /autoprofile /run x.xex` startet als „XL … NTSC“ statt „800 … PAL“.
#   /profile:<Name> -- wählt ein Profil und hebt damit /defprofile auf; ein unbekannter Name wird ohne Meldung ignoriert.
# Nicht aufgenommen, weil sie die Installation oder die Ausgabe verändern bzw. keine sichtbare Wirkung haben: /portable (legt Altirra.ini im Programmordner an),
#   /portablealt (Wirkung nicht bestätigt: auch nach geordnetem Beenden entstand die INI-Datei nicht), /startuplog (Ausgabe nur auf einer Konsole),
#   /fullheapdump /crash /macwin64hack (Absturzdiagnose), /siopatchsafe (Unterschied zu /siopatch in der Altirra-Hilfe nicht beschrieben).
# Aufgenommen (Wirkung in der Hilfe beschrieben, Syntax ohne Fehlerfenster gestartet): /baseline /rawkeys /bootrw /bootvrw /casautoboot /cheats /pclink /hdpathrw
#   /hostcpu /advconfig /portabletemp /reset /resetall.
AL = [("video", "Bild"), ("hardware", "Hardware"), ("drives", "Programme und Datenträger"), ("acceleration", "Beschleunigung"), ("debug", "Debugger"), ("operation", "Betrieb")]


def aj(fn, *a, **kw):
    p = fn(*a, **kw); p["join"] = ":"; return p


altirra = dict(program="altirra", label="Altirra (Atari 8 Bit, Windows)", kinds=["altirra"],
    verified="Altirra 4.01.200, Altirra64.exe /? (Windows). Schalter, Werte und Argumentart aus der Hilfeliste; die Wirkung ist mit der Altirra-Hilfe abgeglichen (Beschleunigung, Disketten, Hardware, Profile, PCLink), die Syntax aller Einträge ist beim Start ohne Fehlerfenster getestet. Über F5 benutzt werden nur /run, /disk, /pal, /tempprofile, /defprofile:xl. Die Beschreibungstexte sind eigene Formulierungen, nicht von Altirra übernommen.",
    groups=[dict(id=i, label=l) for i, l in AL], reserved=["/tempprofile", "/defprofile:xl", "/pal", "/ntsc", "/nobasic", "/basic", "/run", "/runbas", "/cart", "/disk"],
    params=[
        sw("windowed", "video", "/w", "Im Fenster", "Startet Altirra im Fenster.", conflicts=["fullscreen"]),
        sw("fullscreen", "video", "/f", "Vollbild", "Startet Altirra im Vollbild.", conflicts=["windowed"]),
        sw("borderless", "video", "/borderless", "Rahmenloser Vollbildmodus", "Schaltet den rahmenlosen Vollbildmodus ein (mit /noborderless aus); laut Altirra wirkt er nur im Vollbild.", off="/noborderless"),
        aj(ch, "artifact", "video", "/artifact", "Artefaktfarben", "Simuliert die Farben, die aus abwechselnd hellen und dunklen Pixeln entstehen (Artefaktfarben); NTSC oder PAL passend zur gewählten Norm, „hohe Qualität“ rechnet genauer.", [("none", "keine"), ("ntsc", "NTSC"), ("ntschi", "NTSC (hohe Qualität)"), ("pal", "PAL"), ("palhi", "PAL (hohe Qualität)")]),
        sw("gdi", "video", "/gdi", "Anzeige über GDI", "Verwendet GDI als Anzeigetreiber.", conflicts=["d3d9", "d3d11"]),
        sw("d3d9", "video", "/d3d9", "Anzeige über Direct3D 9", "Verwendet Direct3D 9 als Anzeigetreiber.", conflicts=["gdi", "d3d11"]),
        sw("d3d11", "video", "/d3d11", "Anzeige über Direct3D 11", "Verwendet Direct3D 11 als Anzeigetreiber.", conflicts=["gdi", "d3d9"]),
        sw("vsync", "video", "/vsync", "Vertikale Synchronisation", "Gleicht die Ausgabe mit dem Bildwechsel des Monitors ab.", off="/novsync"),
        aj(ch, "kernel", "hardware", "/kernel", "Betriebssystem (Kernel)", "Welches Betriebssystem-ROM Altirra benutzt.", [("default", "Standard"), ("osa", "OS-A"), ("osb", "OS-B"), ("xl", "XL/XE"), ("lle", "LLE"), ("llexl", "LLE XL/XE"), ("5200", "5200"), ("5200lle", "5200 LLE")]),
        aj(ch, "hardware", "hardware", "/hardware", "Maschine", "Welche Maschine nachgebildet wird.", [("800", "Atari 800"), ("800xl", "Atari 800XL"), ("5200", "Atari 5200")]),
        aj(ch, "memsize", "hardware", "/memsize", "Speichergröße", "Größe des Arbeitsspeichers.", [(v, v) for v in ("8K", "16K", "24K", "32K", "48K", "52K", "64K", "128K", "320K", "320KCOMPY", "576K", "576KCOMPY", "1088K")]),
        aj(ch, "axlonmemsize", "hardware", "/axlonmemsize", "Axlon-Speicher", "Größe der Axlon-Speichererweiterung.", [(v, v) for v in ("none", "64K", "128K", "256K", "512K", "1024K", "2048K", "4096K")]),
        aj(ch, "highbanks", "hardware", "/highbanks", "Hohe Speicherbänke", "Anzahl der hohen (linearen) Speicherbänke für eine 65C816-CPU; „na“ = keine. Ohne 65C816-Prozessor im Profil ohne Wirkung.", [(v, v) for v in ("na", "0", "1", "3", "15", "63")]),
        sw("stereo", "hardware", "/stereo", "Zwei POKEYs (Stereo)", "Bildet zwei POKEY-Chips nach.", off="/nostereo"),
        aj(ch, "soundboard", "hardware", "/soundboard", "Soundboard", "Adresse der Soundboard-Erweiterung.", [("d2c0", "$D2C0"), ("d500", "$D500"), ("d600", "$D600")]),
        sw("covox", "hardware", "/covox", "Covox", "Schaltet die Covox-Tonausgabe ein.", off="/nocovox"),
        sw("baseline", "hardware", "/baseline", "Hardware auf Standard", "Setzt die Hardware-Einstellungen (Maschine, Speicher, Kernel usw.) auf Altirras Standardwerte zurück. Mit /tempprofile, das VintageCoder setzt, wird nichts gespeichert."),
        sw("rawkeys", "hardware", "/rawkeys", "Rohe Tastaturdrücke", "Schaltet die Weitergabe roher Tastaturdrücke ein (mit /norawkeys aus); Altirra nennt sie „raw keyboard presses“. Sichtbare Wirkung nicht geprüft.", off="/norawkeys"),
        tx("cartmapper", "drives", "/cartmapper", "Modultyp (Mapper)", "Typ des Moduls bei Modulabbildern ohne Kopf (Zahl oder Name des Mappers laut Altirra). Wirkt zusammen mit /cart, das VintageCoder für Modulabbilder selbst setzt.", example="1"),
        fi("tape", "drives", "/tape", "Kassettenimage", "Legt dieses Kassettenimage ein."),
        tx("tapepos", "drives", "/tapepos", "Bandposition", "Startposition auf der Kassette als [[hh:]mm:]ss.sss.", example="0:30.000", requires=["tape"]),
        fi("hdpath", "drives", "/hdpath", "Ordner als Festplatte (H:)", "Bindet einen Ordner des Rechners als Laufwerk H: ein, nur lesend (Schreiben erlaubt /hdpathrw).", example="{project_dir}", conflicts=["hdpathrw"]),
        tx("type", "drives", "/type", "Tasten eintippen", "Tippt diese Zeichen nach dem Start ein; ~ ist Enter, ` ist ein Anführungszeichen.", example="RUN~"),
        fi("hdpathrw", "drives", "/hdpathrw", "Ordner als Festplatte (H:), schreibbar", "Bindet einen Ordner des Rechners als Laufwerk H: ein, mit Schreibzugriff: Programme können dort Dateien anlegen und ändern.", example="{project_dir}", conflicts=["hdpath"]),
        aj(tx, "pclink", "drives", "/pclink", "PCLink-Ordner", "Stellt einen Ordner des Rechners über PCLink bereit (Zugriff vom Atari über SIO, z. B. mit SpartaDOS X). Wert: ro,<Ordner> nur lesend oder rw,<Ordner> mit Schreibzugriff; ein anderer Modus führt zu einem Fehlerfenster von Altirra.", example="rw,{project_dir}"),
        sw("bootrw", "drives", "/bootrw", "Diskette: Lesen/Schreiben", "Startet Diskettenabbilder im Modus Lesen/Schreiben: Änderungen des Programms landen im Abbild selbst (bei Projektdisketten also in der Datei).", conflicts=["bootvrw"]),
        sw("bootvrw", "drives", "/bootvrw", "Diskette: virtuell beschreibbar", "Startet Diskettenabbilder virtuell beschreibbar: Schreiben ist erlaubt, die Änderungen bleiben aber im Speicher und das Abbild unverändert (Speichern über den Dialog Disk Drives von Altirra).", conflicts=["bootrw"]),
        sw("casautoboot", "drives", "/casautoboot", "Kassette automatisch starten", "Hält beim Start die START-Taste und löst das Laden der Kassette aus (mit /nocasautoboot aus). Funktioniert nur bei Kassetten mit Maschinenprogramm; BASIC-Kassetten lädt man mit CLOAD.", off="/nocasautoboot"),
        fi("cheats", "drives", "/cheats", "Cheat-Datei", "Lädt eine Cheat-Datei im Format von Altirra. Fehlt die Datei, zeigt Altirra ein Fehlerfenster."),
        sw("burstio", "acceleration", "/burstio", "Burst-I/O", "Verkürzt die Pause zwischen den Bytes auf dem SIO-Bus; beschleunigt eigene Diskettenlader, die der SIO-Patch nicht abfangen kann. Selten zeigen Programme dabei Fehler oder erkennen es als Kopierschutz-Umgehung, dann abschalten.", off="/noburstio"),
        sw("siopatch", "acceleration", "/siopatch", "SIO-Patch", "Beschleunigt Disketten- und Kassettenzugriffe, indem Altirra die SIO-Aufrufe des Betriebssystems direkt bedient, sodass Laden fast sofort geht.", off="/nosiopatch"),
        sw("fastboot", "acceleration", "/fastboot", "Schneller Start", "Verkürzt Startverzögerung, ROM-Prüfsumme und Geräteabfrage des XL/XE-Betriebssystems; spart beim Kaltstart einige Sekunden.", off="/nofastboot"),
        sw("accuratedisk", "acceleration", "/accuratedisk", "Genaue Laufwerkszeiten", "Bildet Drehverzögerung und Kopfbewegung des Laufwerks nach: Laden dauert länger, ist aber für manche kopiergeschützten Disketten nötig. Mit SIO-Patch bleibt nur die Drehposition erfasst.", off="/noaccuratedisk"),
        sw("debug", "debug", "/debug", "Debugger öffnen", "Öffnet beim Start den Debugger von Altirra."),
        sw("debugbrkrun", "debug", "/debugbrkrun", "Halt beim Programmstart", "Hält im Debugger an der Startadresse des mit /run geladenen Programms an.", off="/nodebugbrkrun", requires=["debug"]),
        aj(tx, "debugcmd", "debug", "/debugcmd", "Debugger-Befehl", "Führt diesen Befehl beim Start im Debugger von Altirra aus.", example="bp 2000", requires=["debug"]),
        sw("singleinstance", "operation", "/singleinstance", "Nur eine Instanz", "Verwendet eine bereits laufende Altirra-Instanz weiter, statt eine neue zu öffnen. Bei F5 meist unerwünscht, weil dann ein altes Fenster das Programm übernimmt.", off="/nosingleinstance"),
        aj(ch, "hostcpu", "operation", "/hostcpu", "Prozessorerkennung des Rechners", "Überschreibt, welche Befehlssätze Altirra beim Rechner annimmt (Fehlersuche bei Problemen mit SSE/AVX-Code). Unbekannte Werte nimmt Altirra ohne Meldung an.", [(v, v) for v in ("none", "mmx", "sse", "sse2", "sse3", "ssse3", "sse41", "sse42", "avx", "avx2")]),
        sw("advconfig", "operation", "/advconfig", "Erweiterte Einstellungen zeigen", "Öffnet vor dem Start den Dialog „Advanced Configuration“ und wartet, bis er geschlossen ist; für automatische Starts ungeeignet."),
        sw("portabletemp", "operation", "/portabletemp", "Einstellungen nur im Speicher", "Altirra lädt und speichert keine Einstellungen (Registry im Arbeitsspeicher). Es gelten die Altirra-Grundwerte statt der eigenen Einstellungen, z. B. interne Firmware statt der eingetragenen ROMs und 320K-Rambo-Speicher."),
        aj(tx, "reset", "operation", "/reset", "Einstellungsgruppen zurücksetzen", "ACHTUNG, dauerhaft: setzt die genannten Gruppen von Altirra auf Standard zurück. Werte: display, windowlayouts, advconfig, devices (mehrere durch Komma).", example="display,windowlayouts"),
        sw("resetall", "operation", "/resetall", "Alle Einstellungen zurücksetzen", "ACHTUNG, dauerhaft: setzt ALLE Einstellungen von Altirra auf Standard zurück (außer mit /portabletemp, dann nur im Arbeitsspeicher)."),
        sw("skipsetup", "operation", "/skipsetup", "Einrichtung überspringen", "Überspringt den Einrichtungsdialog beim ersten Start."),
    ])
write("altirra", altirra)

# --- Xemu (MEGA65) ---
XG = [("run", "Ablauf"), ("video", "Bild"), ("sound", "Ton"), ("system", "Maschine"), ("input", "Eingabe"), ("drives", "Laufwerke und Karte"), ("debug", "Fehlersuche")]
xemu = dict(program="xemu", label="Xemu (MEGA65)", kinds=["xemu"], verified="xmega65 -h, Xemu master 40dfef0d (20260129, macOS)",
    groups=[dict(id=i, label=l) for i, l in XG], reserved=["-besure", "-rom", "-uartmon", "-prg", "-prgmode"],
    params=[
        sw("fullscreen", "video", "-fullscreen", "Vollbild", "Startet im Vollbild."),
        sw("fullborders", "video", "-fullborders", "Voller Bildrand", "Zeigt den Bildrand ungeschnitten."),
        sw("driveled", "video", "-driveled", "Laufwerk-LED", "Zeichnet die Laufwerk-LED oben rechts ins Bild."),
        sw("showscanlines", "video", "-showscanlines", "Scanlines in V200-Modi", "Zeigt Scanlines in den V200-Modi."),
        ch("videostd", "video", "-videostd", "Videonorm", "Videonorm beim Start und Reset.", [("0", "PAL"), ("1", "NTSC"), ("-1", "Vorgabe des Hyppo")]),
        ch("coloureffect", "video", "-coloureffect", "Farbeffekt", "Farbeffekt der Ausgabe.", [("0", "keiner"), ("1", "Graustufen"), ("2", "grüner Monitor")]),
        nu("sdlrenderquality", "video", "-sdlrenderquality", "Skalierungsqualität (SDL)", "Qualität der Skalierung: 0, 1 oder 2.", min=0, max=2),
        tx("winpos", "video", "-winpos", "Fensterposition", "Position des Fensters als x,y.", example="100,100"),
        sw("nosound", "sound", "-nosound", "Ton aus", "Schaltet die Tonausgabe ab.", conflicts=["mastervolume"]),
        nu("mastervolume", "sound", "-mastervolume", "Gesamtlautstärke", "Endlautstärke (100 = voll, 0 = stumm).", example="80", min=0, max=100, conflicts=["nosound"]),
        sw("noopl3", "sound", "-noopl3", "OPL3 aus", "Schaltet die OPL3-Nachbildung ab."),
        nu("model", "system", "-model", "MEGA65-Modell", "Kennung des nachgebildeten MEGA65-Modells.", min=0),
        nu("fastclock", "system", "-fastclock", "Takt im Schnellmodus (MHz)", "Takt des MEGA65 im schnellen Modus.", example="40", min=1),
        sw("fastboot", "system", "-fastboot", "Schneller Start", "Versucht beim Hochfahren den schlaflosen (schnellsten) Emulationsmodus."),
        sw("sleepless", "system", "-sleepless", "Höchste Geschwindigkeit", "Maximale Emulationsgeschwindigkeit (zum Testen).", conflicts=[]),
        sw("go64", "system", "-go64", "In den C64-Modus", "Wechselt nach dem Start in den C64-Modus (mit Eintippen, kombinierbar mit „Autoload“)."),
        sw("autoload", "system", "-autoload", "Erstes Programm der Diskette starten", "Lädt und startet das erste Programm von der Diskette (mit Eintippen)."),
        ch("joyport", "input", "-joyport", "Joystickport", "Welcher Joystickport standardmäßig nachgebildet wird.", [("1", "Port 1"), ("2", "Port 2")]),
        sw("curskeyjoy", "input", "-curskeyjoy", "Cursortasten als Joystick", "Die Cursortasten ersetzen den Joystick; im BASIC lässt sich dann der Cursor nicht mehr bewegen."),
        sw("nomouseemu", "input", "-nomouseemu", "Keine Mausnachbildung", "Schaltet die Mausnachbildung ab."),
        sw("allowmousegrab", "input", "-allowmousegrab", "Maus mit Linksklick einfangen", "Erlaubt das Einfangen der Maus per Linksklick."),
        fi("d81", "drives", "-8", "D81-Image in Laufwerk 8", "Externes D81-Image (nicht auf der SD-Karte) in Laufwerk 8.", example="{disk}"),
        fi("d81_9", "drives", "-9", "D81-Image in Laufwerk 9", "Externes D81-Image in Laufwerk 9."),
        fi("sdimg", "drives", "-sdimg", "SD-Karten-Image", "Pfad des SD-Karten-Images, das statt des Standards verwendet wird."),
        fi("cart", "drives", "-cart", "Modul", "Lädt eine Moduldatei.", example="{output}"),
        fi("initattic", "drives", "-initattic", "Attic-RAM vorbelegen", "Füllt das Attic-RAM mit dem Inhalt einer Datei."),
        sw("headless", "debug", "-headless", "Ohne Fenster", "Läuft ohne Fenster (für Tests)."),
        sw("besure", "debug", "-besure", "Rückfragen unterdrücken", "Überspringt die Rückfrage „Sicher?“ bei Reset und Beenden.", conflicts=[]),
        fi("screenshot", "debug", "-screenshot", "Bildschirmfoto beim Beenden", "Speichert beim Beenden ein Bild als PNG.", example="{project_dir}/build/{project}.png"),
        fi("dumpmem", "debug", "-dumpmem", "Speicher beim Beenden sichern", "Schreibt beim Beenden den Speicherinhalt in diese Datei."),
        fi("dumpscreen", "debug", "-dumpscreen", "Bildschirmtext beim Beenden sichern", "Schreibt beim Beenden den Bildschirminhalt als Text in diese Datei."),
        fi("hyperserialfile", "debug", "-hyperserialfile", "Serielle Hypervisor-Ausgabe in Datei", "Schreibt die serielle Ausgabe des Hypervisors in diese Datei."),
        sw("hyperdebug", "debug", "-hyperdebug", "Hypervisor-Debug", "Sehr langsamer, gesprächiger Debugmodus des Hypervisors."),
        sw("matrixstart", "debug", "-matrixstart", "Matrix-Modus beim Start", "Startet mit aktiviertem Matrix-Modus (Monitor)."),
    ])
write("xemu", xemu)

# --- Assembler (vcc asm / Build der IDE) ---
# Nur Schalter, die der Build in der IDE wirklich auswertet (Einstellungen -> Toolchain); Ausgabedateien (--listing, --labels, --debug,
# --format, --cart, --target, --cpu, --memory) setzt der Build je Projekt selbst. Unbekanntes ist hier ein Fehler (strict).
vcc_asm = dict(program="vcc-asm", label="Assembler (vc-asm)", kinds=["vcc-asm"], strict=True,
    verified="vcc --help (vcc asm): --illegals, -I und -D; Verhalten in vc_ide::args::apply_asm und den Tests",
    groups=[dict(id="asm", label="Assembler")], reserved=[],
    params=[
        ch("illegals", "asm", "--illegals", "Illegale Opcodes", "Wie der Assembler mit illegalen (undokumentierten) 6502-Befehlen umgeht.",
           [("deny", "verbieten (Fehler)"), ("warn", "warnen"), ("allow", "erlauben")], default="warn"),
        tx("include", "asm", "-I", "Include-Ordner", "Zusätzlicher Suchordner für .include und .incbin; nach dem Projektordner, vor den Systemdateien. Weitere Ordner im freien Feld mit je einem eigenen -I.", example="{project_dir}/inc"),
        tx("define", "asm", "-D", "Konstante", "Legt eine Konstante NAME=Zahl für alle Quellen an (z. B. DEBUG=1 für .if DEBUG). Weitere im freien Feld mit je einem eigenen -D.", example="DEBUG=1"),
    ])
write("vcc-asm", vcc_asm)
