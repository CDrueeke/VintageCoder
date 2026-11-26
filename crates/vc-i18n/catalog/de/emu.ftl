# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: emu (aus crates/vc-emu/src/atari800.rs extrahiert, Wortlaut unverändert)

# Kontext: atari800.rs, ok_or_else
emu-atari800-nicht-gefunden-p1 = atari800 nicht gefunden ({ $p1 })

# Kontext: atari800.rs, Err
emu-atari800-monitor-meldet-sich-nicht = atari800-Monitor meldet sich nicht

# Kontext: atari800.rs, Err
emu-emulator-laeuft-zuerst-anhalten-haltepunkt-oder = Emulator läuft – zuerst anhalten (Haltepunkt oder F8 im Emulatorfenster)

# Kontext: atari800.rs, Err
emu-atari800-antwortet-nicht-auf-cmd = atari800 antwortet nicht auf '{ $cmd }'

# Kontext: atari800.rs, Err
emu-atari800-meldet-zeile-und-zyklus-nicht = atari800 meldet Zeile und Zyklus nicht

# Kontext: atari800.rs, Err
emu-speicher-nicht-lesbar = Speicher nicht lesbar

# Kontext: atari800.rs, Err
emu-atari800-laesst-sich-nicht-per-befehl = atari800 lässt sich nicht per Befehl anhalten – im Emulatorfenster F8 drücken

# Kontext: fujisan.rs, ok_or_else
emu-fujisan-nicht-gefunden-p1 = Fujisan nicht gefunden ({ $p1 })

# Kontext: fujisan.rs, Err
emu-fujisan-wurde-beendet-st-bevor-der = Fujisan wurde beendet ({ $st }), bevor der TCP-Server antwortete

# Kontext: fujisan.rs, Err
emu-fujisan-antwortet-auf-port-port-nicht = Fujisan antwortet auf Port { $port } nicht: { $e }

# Kontext: fujisan.rs, ok_or
emu-keine-adresse = keine Adresse

# Kontext: fujisan.rs, Err
emu-auf-dem-port-antwortet-kein-fujisan = auf dem Port antwortet kein Fujisan: { $hello }

# Kontext: fujisan.rs, Err
emu-fujisan-hat-die-verbindung-geschlossen = Fujisan hat die Verbindung geschlossen

# Kontext: fujisan.rs, Text
emu-antwort-von-fujisan-nicht-lesbar-e = Antwort von Fujisan nicht lesbar: { $e }

# Kontext: fujisan.rs, Err
emu-fujisan-antwortet-nicht-e = Fujisan antwortet nicht: { $e }

# Kontext: fujisan.rs, Err
emu-name-p2 = { $name }: { $p2 }

# Kontext: fujisan.rs, Err
emu-der-fujisan-debugger-laedt-nur-xex = Der Fujisan-Debugger lädt nur .xex-Dateien (Diskettenabbilder und Module bitte mit atari800 debuggen)

# Kontext: fujisan.rs, Err
emu-pcx-datei-nicht-lesbar-erwartet-8 = PCX-Datei nicht lesbar (erwartet: 8 Bit, eine Ebene, RLE)

# Kontext: fujisan.rs, Err
emu-pcx-datei-palette-fehlt = PCX-Datei: Palette fehlt

# Kontext: fujisan.rs, ok_or
emu-pcx-datei-abgeschnitten = PCX-Datei abgeschnitten

# Kontext: fujisan.rs, Text
emu-fujisan-von-https-github-com-pedgarcia = Fujisan von https://github.com/pedgarcia/fujisan/releases installieren oder mit „vcc emulator install fujisan“ laden

# Kontext: fujisan.rs, Err
emu-emulator-laeuft-zuerst-anhalten = Emulator läuft – zuerst anhalten

# Kontext: fujisan.rs, Err
emu-pc-liegt-zu-nah-am-speicherende = PC liegt zu nah am Speicherende für den Vorspann

# Kontext: fujisan.rs, Err
emu-register-setzen-fehlgeschlagen-pc-ist-p1 = Register setzen fehlgeschlagen: PC ist ${ $p1 } statt ${ $p2 } (Vorspann im ROM?)

# Kontext: fujisan.rs, ok_or
emu-screen-capture-nennt-keine-datei = screen.capture nennt keine Datei

# Kontext: fujisan.rs, ok_or_else
emu-register-k-nicht-lesbar = Register { $k } nicht lesbar

# Kontext: fujisan.rs, Err
emu-schreiben-ueber-ffff-hinaus = Schreiben über $FFFF hinaus

# Kontext: install.rs, ok_or
emu-konfigurationsordner-nicht-ermittelbar = Konfigurationsordner nicht ermittelbar

# Kontext: install.rs, Err
emu-pruefsumme-stimmt-nicht-erwartet-p1-erhalten = Prüfsumme stimmt nicht (erwartet { $p1 }, erhalten { $got }); die Datei wurde gelöscht

# Kontext: install.rs, Text
emu-tar-nicht-ausfuehrbar-e = tar nicht ausführbar: { $e }

# Kontext: install.rs, Err
emu-fuer-dieses-paket-ist-keine-pruefsumme = Für dieses Paket ist keine Prüfsumme hinterlegt; es wird nicht geladen

# Kontext: install.rs, Text
emu-p1-p2-ist-bereits-installiert-p3 = { $p1 } { $p2 } ist bereits installiert: { $p3 }

# Kontext: install.rs, Text
emu-curl-nicht-ausfuehrbar-e = curl nicht ausführbar: { $e }

# Kontext: install.rs, Text
emu-pruefe-sha-256 = Prüfe SHA-256 …

# Kontext: install.rs, Text
emu-packe-aus = Packe aus …

# Kontext: install.rs, Text
emu-fertig-p1 = Fertig: { $p1 }

# Kontext: lib.rs, ok_or
emu-mega65-rom-nicht-gefunden = MEGA65-ROM nicht gefunden

# Kontext: lib.rs, Err
emu-diese-anbindung-hat-keinen-debugger-und = Diese Anbindung hat keinen Debugger- und Test-Adapter

# Kontext: lib.rs, Err
emu-adapter-other-ist-unbekannt = Adapter „{ $other }“ ist unbekannt

# Kontext: lib.rs, Err
emu-datenhaltepunkte-werden-von-diesem-adapter-nicht = Datenhaltepunkte werden von diesem Adapter nicht unterstützt

# Kontext: lib.rs, Err
emu-register-setzen-wird-von-diesem-adapter = Register setzen wird von diesem Adapter nicht unterstützt

# Kontext: lib.rs, Err
emu-zyklenmessung-wird-von-diesem-adapter-nicht = Zyklenmessung wird von diesem Adapter nicht unterstützt

# Kontext: lib.rs, Err
emu-das-bild-des-emulators-ist-mit = Das Bild des Emulators ist mit diesem Adapter nicht abrufbar

# Kontext: lib.rs, Err
emu-texteingabe-wird-von-diesem-adapter-nicht = Texteingabe wird von diesem Adapter nicht unterstützt

# Kontext: lib.rs, Text
emu-paket-pkg-der-distribution-installieren-z = Paket „{ $pkg }“ der Distribution installieren, z. B. sudo apt install { $pkg } / sudo dnf install { $pkg } / sudo zypper install { $pkg }

# Kontext: mega65.rs, Err
emu-p-unix-sockets-gibt-es-unter = { $p }: Unix-Sockets gibt es unter Windows nicht – tcp:127.0.0.1:Port verwenden

# Kontext: mega65.rs, Err
emu-dev-schnittstelle-laesst-sich-nicht-einstellen = { $dev }: Schnittstelle lässt sich nicht einstellen

# Kontext: mega65.rs, Text
emu-keine-antwort-des-monitors-e = keine Antwort des Monitors: { $e }

# Kontext: mega65.rs, ok_or_else
emu-sd-dir-keine-mega65-rom-gefunden = { $sd_dir }: keine MEGA65.ROM gefunden

# Kontext: mega65.rs, ok_or
emu-xmega65-xemu-nicht-gefunden-vc-xemu = xmega65 (Xemu) nicht gefunden – VC_XEMU setzen

# Kontext: mega65.rs, Err
emu-keine-antwort-auf-cmd = keine Antwort auf '{ $cmd }'

# Kontext: mega65.rs, Err
emu-prg-zu-kurz = PRG zu kurz

# Kontext: mega65.rs, Err
emu-mega65-monitor-nur-schreib-haltepunkte-wertaenderung = MEGA65-Monitor: nur Schreib-Haltepunkte (Wertänderung) per Einzelschritt-Beobachtung

# Kontext: mega65.rs, Err
emu-bereich-hoechstens-256-bytes = Bereich höchstens 256 Bytes

# Kontext: mega65.rs, Err
emu-hoechstens-4-beobachtete-bereiche = höchstens 4 beobachtete Bereiche

# Kontext: ultimate.rs, Text
emu-debug-register-antwort-nicht-lesbar-p1 = Debug-Register: Antwort nicht lesbar: { $p1 }

# Kontext: ultimate.rs, Text
emu-ultimate-64-unter-host-e = Ultimate 64 unter { $host }: { $e }

# Kontext: ultimate.rs, Err
emu-keine-geraeteinformation-in-der-antwort = keine Geräteinformation in der Antwort

# Kontext: ultimate.rs, Text
emu-addr-e = { $addr }: { $e }

# Kontext: ultimate.rs, ok_or
emu-adresse-nicht-aufloesbar = Adresse nicht auflösbar

# Kontext: ultimate.rs, Text
emu-keine-verbindung-e = keine Verbindung: { $e }

# Kontext: ultimate.rs, Err
emu-netzwerkpasswort-der-ultimate-fehlt-oder-ist = Netzwerkpasswort der Ultimate fehlt oder ist falsch (403)

# Kontext: ultimate.rs, Err
emu-datenstroeme-gibt-es-nur-am-ultimate = Datenströme gibt es nur am Ultimate 64 (501)

# Kontext: ultimate.rs, Text
emu-http-status = HTTP { $status }

# Kontext: ultimate.rs, Text
emu-kein-lan-link-die-ultimate-hat = Kein LAN-Link: Die Ultimate hat kein LAN-Kabel. Die Ströme laufen nur über den LAN-Anschluss, nicht über WLAN (Meldung der Ultimate: „No Operational Network Interface“).

# Kontext: ultimate.rs, Text
emu-strom-p1-laesst-sich-nicht-what = Strom „{ $p1 }“ lässt sich nicht { $what }: { $raw }

# Kontext: ultimate.rs, Err
emu-403-netzwerkpasswort-fehlt-oder-falsch = 403: Netzwerkpasswort fehlt oder falsch

# Kontext: ultimate.rs, Err
emu-501-funktion-nur-am-ultimate-64 = 501: Funktion nur am Ultimate 64

# Kontext: ultimate.rs, Err
emu-http-c = HTTP { $c }

# Kontext: ultimate.rs, ok_or
emu-ungueltige-http-antwort = ungültige HTTP-Antwort

# Kontext: ultimate.rs, ok_or
emu-ungueltige-statuszeile = ungültige Statuszeile

# Kontext: ultimate.rs, ok_or
emu-chunked-abgeschnitten = chunked: abgeschnitten

# Kontext: ultimate.rs, ok_or
emu-chunked-laenge-fehlt = chunked: Länge fehlt

# Kontext: ultimate.rs, Text
emu-chunked-ungueltige-laenge = chunked: ungültige Länge

# Kontext: ultimate.rs, Err
emu-die-ultimate-hardware-liefert-keine-cpu = Die Ultimate-Hardware liefert keine CPU-Register

# Kontext: ultimate.rs, Err
emu-schreibzugriff-ueber-ffff-hinaus = Schreibzugriff über $FFFF hinaus

# Kontext: ultimate.rs, Err
emu-einzelschritt-gibt-es-auf-der-ultimate = Einzelschritt gibt es auf der Ultimate-Hardware nicht

# Kontext: ultimate.rs, Err
emu-haltepunkte-gibt-es-auf-der-ultimate = Haltepunkte gibt es auf der Ultimate-Hardware nicht

# Kontext: vice.rs, ok_or
emu-name-vice-nicht-gefunden = { $name } (VICE) nicht gefunden

# Kontext: vice.rs, Err
emu-vice-monitor-nicht-erreichbar = VICE-Monitor nicht erreichbar

# Kontext: vice.rs, Text
emu-zeitueberschreitung-bei-vice-befehl-0x-code = Zeitüberschreitung bei VICE-Befehl 0x{ $code }

# Kontext: vice.rs, Err
emu-ressource-name-ist-keine-zahl = Ressource { $name } ist keine Zahl

# Kontext: vice.rs, Err
emu-ressource-name-antwort-zu-kurz = Ressource { $name }: Antwort zu kurz

# Kontext: vice.rs, ok_or_else
emu-vice-kennt-das-register-n-nicht = VICE kennt das Register { $n } nicht

# Kontext: vice.rs, Err
emu-zyklenmessung-ist-nur-fuer-x64sc-geprueft = Zyklenmessung ist nur für x64sc geprüft, nicht für { $p1 }

# Kontext: vice.rs, Err
emu-zyklenmessung-nur-fuer-pal-gemessen-videonorm = Zyklenmessung nur für PAL gemessen (Videonorm { $x })

# Kontext: vice.rs, Err
emu-vice-meldet-lin-cyc-nicht = VICE meldet LIN/CYC nicht

# Kontext: vice.rs, Err
emu-vice-palette-antwort-zu-kurz = VICE-Palette: Antwort zu kurz

# Kontext: vice.rs, Err
emu-vice-palette-eintrag-unvollstaendig = VICE-Palette: Eintrag unvollständig

# Kontext: vice.rs, Err
emu-vice-bild-antwort-zu-kurz = VICE-Bild: Antwort zu kurz

# Kontext: vice.rs, Err
emu-vice-bild-kopflaenge-info-unbekannt = VICE-Bild: Kopflänge { $info } unbekannt

# Kontext: vice.rs, Err
emu-vice-bild-bpp-bit-je-punkt = VICE-Bild: { $bpp } Bit je Punkt werden nicht unterstützt

# Kontext: vice.rs, Err
emu-vice-bild-groesse-w-h-unplausibel = VICE-Bild: Größe { $w }×{ $h } unplausibel

# Kontext: vice.rs, Err
emu-vice-palette-ist-leer = VICE-Palette ist leer

# Kontext: vice.rs, ok_or
emu-vice-bild-puffer-fehlt = VICE-Bild: Puffer fehlt

# Kontext: vice.rs, Err
emu-vice-bild-puffer-zu-kurz-p1 = VICE-Bild: Puffer zu kurz ({ $p1 } von { $p2 } Bytes)

# Kontext: vice.rs, Text
emu-vice-a-b-stuerzt-unter-windows = VICE { $a }.{ $b } stürzt unter Windows beim Einzelschritt ab – bitte VICE 3.10 oder neuer verwenden

# Kontext: receiver.rs, Text
emu-host-e = { $host }: { $e }

# Kontext: receiver.rs, ok_or_else
emu-host-keine-ipv4-adresse = { $host }: keine IPv4-Adresse

# Kontext: receiver.rs, Text
emu-der-rechner-erreicht-die-ultimate-nicht = Der Rechner erreicht die Ultimate nicht (keine Route).

# Kontext: receiver.rs, Text
emu-port-p1-ist-belegt = Port { $p1 } ist belegt.

# Kontext: receiver.rs, Text
emu-keine-berechtigung-udp-port-p1-zu = Keine Berechtigung, UDP-Port { $p1 } zu öffnen (Rechte oder Firewall).

# Kontext: receiver.rs, Text
emu-udp-empfaenger-addr-e = UDP-Empfänger { $addr }: { $e }

# Kontext: receiver.rs, Text
emu-multicast-gruppe-g-auf-iface-e = Multicast-Gruppe { $g } auf { $iface }: { $e }

# Kontext: receiver.rs, Err
emu-keine-ipv4-adresse = keine IPv4-Adresse

# Kontext: trace.rs, Text
emu-hardware-messung-v-zyklen-von-p2 = Hardware-Messung: { $v } Zyklen von ${ $p2 } bis ${ $p3 } (Hinweis, kein Urteil)

# Kontext: trace.rs, Text
emu-hardware-messung-v-zyklen-von-p2-2 = Hardware-Messung: { $v } Zyklen von ${ $p2 } bis ${ $p3 } (Hinweis, kein Urteil; { $n } Messung(en) ungültig wegen Paketverlust)

# Kontext: trace.rs, Text
emu-hardware-messung-p1-bis-p2-noch = Hardware-Messung: ${ $p1 } bis ${ $p2 } noch nicht durchlaufen

# Kontext: trace.rs, Text
emu-hardware-messung-ungueltig-paketverlust-n-messung = Hardware-Messung: ungültig (Paketverlust), { $n } Messung(en) verworfen

# Kontext: trace.rs, Text
emu-der-debug-strom-enthaelt-zyklen-mit = Der Debug-Strom enthält Zyklen mit PHI2 = 0: Er läuft nicht im Modus „6510 Only“. Modi mit VIC- oder 1541-Zyklen werden nicht ausgewertet; bitte in der Ultimate unter Data Streams → Debug Stream Mode „6510 Only“ wählen.

# Kontext: video.rs, Err
emu-header-pixel-je-zeile = Pixel je Zeile

# Kontext: video.rs, Err
emu-header-zeilen-je-paket = Zeilen je Paket

# Kontext: video.rs, Err
emu-header-zeile-ausserhalb-des-bilds = Zeile außerhalb des Bilds

# Kontext: wav.rs, Err
emu-wav-datei-voll-4-gib = WAV-Datei voll (4 GiB)

# Kontext: wav.rs, Err
emu-keine-wav-datei = keine WAV-Datei

# Kontext: wav.rs, ok_or
emu-fmt-block-fehlt = fmt-Block fehlt

# Kontext: wav.rs, ok_or
emu-data-block-fehlt = data-Block fehlt

# Kontext: wav.rs, Err
emu-nur-pcm-mit-16-bit-und = nur PCM mit 16 Bit und 1 oder 2 Kanälen (hier Format { $tag }, { $bits } Bit, { $ch } Kanäle)

# Kontext: atari800.rs, mega65.rs
emu-register-nicht-lesbar-roh = Register nicht lesbar: { $raw }

# Kontext: fujisan.rs
emu-fehler = Fehler

# Kontext: ultimate.rs (Verb in „Strom … lässt sich nicht …“)
emu-stream-starten = starten

# Kontext: ultimate.rs
emu-stream-stoppen = stoppen

# Kontext: streams/mod.rs
emu-paketlaenge-got-statt-want = Paketlänge { $got } statt { $want }

# Kontext: streams/mod.rs
emu-paketkopf-unplausibel-w = Paketkopf unplausibel: { $w }

# Kontext: streams/video.rs (Paketkopf-Felder)
emu-header-bits-je-pixel = Bits je Pixel

# Kontext: streams/video.rs
emu-header-kodierung = Kodierung

# Bildstrom (K1): VICE hat das Fortsetzen der Maschine (Monitorbefehl „exit“) auch nach Wiederholung nicht bestätigt
diag-emu-vice-resume-lost = VICE hat das Fortsetzen nicht bestätigt (Emulator reagiert nicht mehr?)

# Bildstrom (K1): dieser Adapter kann kein Bild liefern
diag-emu-stream-no-frames = Dieser Emulator liefert kein Bild für einen Bildstrom

# Bildstrom (K1): Bildrate außerhalb des erlaubten Bereichs
# Kontext: rate = angeforderte Bilder pro Sekunde, min/max = Grenzen
diag-emu-stream-bad-rate = Bildrate { $rate } außerhalb des erlaubten Bereichs ({ $min } bis { $max } Bilder pro Sekunde)

# Bildstrom (K1): der Strom endet, weil der Bildabruf mehrfach hintereinander fehlschlug
# Kontext: detail = Fehlertext des Emulators
diag-emu-stream-grab-failed = Bildstrom beendet, der Bildabruf schlug fehl: { $detail }

# Bildstrom (K1): der Arbeitsfaden des Stroms ließ sich nicht starten
# Kontext: detail = Fehlertext des Systems
diag-emu-stream-thread = Bildstrom konnte nicht gestartet werden: { $detail }

# I3b
# $got: gelieferte Byte | Beispiel: 10
# $n: erwartete Byte | Beispiel: 16
# $addr: Adresse (hex) | Beispiel: 0400
emu-fujisan-short-read = Fujisan lieferte { $got } statt { $n } Byte ab ${ $addr }
# $got: gelieferte Byte | Beispiel: 10
# $n: erwartete Byte | Beispiel: 16
emu-ultimate-short-read = readmem lieferte { $got } statt { $n } Bytes
