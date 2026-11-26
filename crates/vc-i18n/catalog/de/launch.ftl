# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: launch (aus crates/vc-ide/src/launch.rs extrahiert, Wortlaut unverändert)

# Kontext: launch.rs, Text
launch-cmd-e = { $cmd }: { $e }

# Kontext: launch.rs, Text
launch-rueckgabewert-c = , Rückgabewert { $c }

# Kontext: launch.rs, Text
launch-der-emulator-hat-sich-sofort-beendet = Der Emulator hat sich sofort beendet{ $code }. Fehlende ROM-Dateien? Siehe Einstellungen ▸ ROMs.

# Kontext: launch.rs, Text
launch-ausgabe-des-emulators = Ausgabe des Emulators:

# Kontext: launch.rs, ok_or_else
launch-atari800-nicht-gefunden-p1-oder-pfad = atari800 nicht gefunden ({ $p1 } oder Pfad unter Einstellungen eintragen)

# Kontext: launch.rs, Text
launch-rel-e = { $rel }: { $e }

# Kontext: launch.rs, ok_or
launch-mega65-rom-nicht-gefunden = MEGA65-ROM nicht gefunden

# Kontext: launch.rs, Text
launch-mega65-start-fehlgeschlagen-e = MEGA65-Start fehlgeschlagen: { $e }

# Kontext: launch.rs, Text
launch-xemu-programm-wird-mit-prg-im = Xemu (Programm wird mit -prg im C65-Modus geladen und gestartet)

# Kontext: launch.rs, Text
launch-xemu-programm-wird-ueber-den-monitor = Xemu (Programm wird über den Monitor geladen, Start bei ${ $entry }; dauert ca. 15 s)

# Kontext: launch.rs, ok_or
launch-prog-vice-nicht-gefunden = { $prog } (VICE) nicht gefunden

# Kontext: launch.rs, Text
launch-cmd-p2 = { $cmd } { $p2 }

# Kontext: launch.rs, Err
launch-mega65-einlegen-im-emulator-ist-noch = MEGA65: Einlegen im Emulator ist noch nicht umgesetzt

# Kontext: launch.rs, ok_or
launch-p1-vice-nicht-gefunden = { $p1 } (VICE) nicht gefunden

# Kontext: launch.rs, Err
launch-die-ultimate-hardware-spielt-nur-c64 = Die Ultimate-Hardware spielt nur C64-Programme

# Kontext: launch.rs, Err
launch-auf-der-ultimate-lassen-sich-nur = Auf der Ultimate lassen sich nur .prg-Programme und .crt-Module starten (keine Diskette)

# Kontext: launch.rs, Text
launch-who-modul-gestartet-p2-bytes = { $who }: Modul gestartet ({ $p2 } Bytes)

# Kontext: launch.rs, Text
launch-who-programm-gestartet-p2-bytes = { $who }: Programm gestartet ({ $p2 } Bytes)

# Kontext: launch.rs, Err
launch-die-mega65-hardware-spielt-nur-mega65 = Die MEGA65-Hardware spielt nur MEGA65-Programme

# Kontext: launch.rs, Text
launch-p1-programm-gestartet-start-bei-entry = { $p1 }: Programm gestartet, Start bei ${ $entry }

# Kontext: launch.rs, Err
launch-unbekannter-maschinentyp-k = unbekannter Maschinentyp '{ $k }'

# Kontext: launch.rs, Err
launch-diskette-auf-hardware-starten-gibt-es = Diskette auf Hardware starten gibt es nur für den MEGA65

# Kontext: launch.rs, Err
launch-eine-diskette-laesst-sich-nur-ueber = Eine Diskette lässt sich nur über Ethernet auf den MEGA65 bringen (Einstellungen → Hardware, Verbindung „Starten“)
