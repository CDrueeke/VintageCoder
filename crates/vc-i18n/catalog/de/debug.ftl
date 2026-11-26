# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: debug (aus crates/vc-gui/src/debug.rs extrahiert, Wortlaut unverändert)

# Kontext: debug.rs, Text
debug-meinst-du-t = Meinst du ${ $t }?

# Kontext: debug.rs, Text
debug-t-ist-weder-register-noch-symbol = „{ $t }“ ist weder Register noch Symbol.{ $rat } Beispiele: $d020 (Byte an dieser Adresse), a, x, y, sp, pc, start (Symbol), [$d020] + 1.

# Kontext: debug.rs, Text
debug-t-ist-kein-gueltiger-ausdruck-roh = „{ $t }“ ist kein gültiger Ausdruck ({ $roh }). Beispiele: $d020, a, sp, start, [$d020] + 1.

# Kontext: debug.rs, Err
debug-t-ist-keine-adresse-beispiele-d020 = „{ $t }“ ist keine Adresse. Beispiele: D020, $d020, 0x0400 oder ein Symbol wie start.













# Kontext: debug.rs, Text
debug-die-verbindung-zum-emulator-ist-verloren = Die Verbindung zum Emulator ist verloren (er wurde beendet oder ist abgestürzt). Die Debug-Sitzung wird beendet.

# Kontext: debug.rs, Text
debug-der-emulator-antwortet-nicht-zeitueberschreitung-versuche = Der Emulator antwortet nicht (Zeitüberschreitung). Versuche es noch einmal oder beende die Sitzung.

# Kontext: debug.rs, Text
debug-der-emulator-meldet-einen-systemfehler-p1 = Der Emulator meldet einen Systemfehler ({ $p1 }).

# Kontext: debug.rs, Text
debug-momentaufnahme-gespeichert-x = Momentaufnahme gespeichert: { $x }

# Kontext: debug.rs, Text
debug-momentaufnahme-e = Momentaufnahme: { $e }
