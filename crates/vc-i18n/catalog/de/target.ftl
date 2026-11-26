# SPDX-License-Identifier: GPL-3.0-or-later
# Oberflächentexte: vc-target (Adressierungsarten, Banking-Prüfung, Zielsystem)

# $id: Kennung des Zielsystems | Beispiel: vic20
target-missing = Zielsystem „{ $id }“ ist in diesem Release nicht enthalten

# $s: Zahl als Text | Beispiel: zwölf
target-bank-invalid-number = ungültige Zahl »{ $s }«
# $n: Wert | Beispiel: 70000
target-bank-out-of-range-u16 = Wert { $n } außerhalb 0..65535
# $n: Wert | Beispiel: 300
target-bank-out-of-range-u8 = Wert { $n } außerhalb 0..255
# $d: Kennung des Presets | Beispiel: basic
target-bank-default-preset-missing = Standard-Preset »{ $d }« fehlt
# $bit: Bitnummer | Beispiel: 9
target-bank-bit-out-of-range = Bit { $bit } außerhalb 0..7

target-mode-name-imp = Implizit
target-mode-name-acc = Akkumulator
target-mode-name-imm = Sofortwert
target-mode-name-imw = Sofortwert, 16 Bit
target-mode-name-zp = Zeropage
target-mode-name-zpx = Zeropage, X-indiziert
target-mode-name-zpy = Zeropage, Y-indiziert
target-mode-name-abs = Absolut
target-mode-name-abx = Absolut, X-indiziert
target-mode-name-aby = Absolut, Y-indiziert
target-mode-name-ind = Indirekt
target-mode-name-izx = Indirekt, X-vorindiziert
target-mode-name-izy = Indirekt, Y-nachindiziert
target-mode-name-rel = Relativ
target-mode-name-rl = Relativ, 16 Bit
target-mode-name-izz = Indirekt, Z-nachindiziert
target-mode-name-isy = Stack-relativ indirekt, Y-nachindiziert
target-mode-name-iax = Absolut indirekt, X-indiziert
target-mode-name-zpr = Zeropage + relativer Sprung

target-mode-explain-imp = kein Operand
target-mode-explain-acc = wirkt auf den Akkumulator
target-mode-explain-imm = der Wert steht direkt im Befehl
target-mode-explain-imw = 16-Bit-Wert steht direkt im Befehl
target-mode-explain-zp = Adresse in den ersten 256 Bytes
target-mode-explain-zpx = Zeropage-Adresse plus X
target-mode-explain-zpy = Zeropage-Adresse plus Y
target-mode-explain-abs = volle 16-Bit-Adresse
target-mode-explain-abx = 16-Bit-Adresse plus X
target-mode-explain-aby = 16-Bit-Adresse plus Y
target-mode-explain-ind = Adresse steht in einem Zeiger
target-mode-explain-izx = Zeiger in der Zeropage, X wird vorher zur Zeigeradresse addiert
target-mode-explain-izy = Zeiger in der Zeropage, Y wird danach zur Zieladresse addiert
target-mode-explain-rel = Sprungweite relativ zum Befehl
target-mode-explain-rl = 16-Bit-Sprungweite relativ zum Befehl
target-mode-explain-izz = Zeiger in der Zeropage, Z wird danach addiert
target-mode-explain-isy = Zeiger auf dem Stack, Y wird danach addiert
target-mode-explain-iax = Zeiger an 16-Bit-Adresse plus X
target-mode-explain-zpr = Zeropage-Adresse prüfen und relativ springen

# $i: Spaltennummer | Beispiel: 3
target-row-column-missing = Spalte { $i } fehlt
# $mode: Kürzel der Adressierungsart | Beispiel: xyz
target-row-unknown-mode = unbekannter Modus { $mode }
target-row-opcode-invalid = Opcode ungültig
# $want: gewünschter Ausbau | Beispiel: 99k
# $options: mögliche Ausbauten, kommagetrennt | Beispiel: 8k, 16k
target-unknown-memory = unbekannter Speicherausbau »{ $want }« (möglich: { $options })
target-json-id-missing = target.id fehlt
target-json-cpu-missing = target.cpu fehlt
target-chip-regs-missing = regs fehlt
target-chip-reg-name-missing = reg.name fehlt
target-chip-reg-addr-invalid = reg.addr ungültig
# $id: Kennung des Zielsystems | Beispiel: vic20
target-chip-inc-header = ; Automatisch erzeugt aus data/chips/ für Target '{ $id }' – nicht von Hand ändern.
# $what: Bezeichnung des Bereichs | Beispiel: base
# $start: Anfangsadresse (hex) | Beispiel: 2000
# $end: Endadresse (hex) | Beispiel: 1000
target-bank-start-after-end = { $what }: Anfang ${ $start } hinter Ende ${ $end }
# $id: Kennung des Presets | Beispiel: basic
target-bank-duplicate-preset = Preset-Kennung »{ $id }« doppelt
# $id: Kennung des Presets | Beispiel: basic
target-bank-preset-label = Preset { $id }

target-cbm2-entfernt = Zielsystem „{ $id }“ (CBM-II) gibt es nicht mehr: Es wurde aus VintageCoder entfernt. Das Projekt lässt sich nicht bauen; PET, C64 und die übrigen Zielsysteme bleiben.
