# SPDX-License-Identifier: GPL-3.0-or-later
# Sprachserver vc-lsp: Meldungen, Hover-Texte, Hilfe (Etappe I2)

# vc-lsp --help: Titelzeile
# $version: Versionsnummer | Beispiel: 0.2.0
lsp-cli-help-title = vc-lsp { $version } - Language Server für VintageCoder-Assembler
lsp-cli-help-protocol = Spricht das Language Server Protocol über stdin und stdout (Nachrichten mit Content-Length).
lsp-cli-help-note = Wird normalerweise vom Editor gestartet, nicht von Hand: ohne Eingabe wartet er auf Nachrichten und gibt nichts aus.
lsp-cli-usage = Aufruf:  vc-lsp [--help] [--version]
lsp-cli-help-editor = Einbinden im Editor: Befehl "vc-lsp" ohne Argumente, Dateien mit der Endung .asm.
lsp-cli-help-exit = Rückgabe: 0 nach geordnetem Beenden (shutdown, exit), sonst 1.
# $option: unbekannte Option | Beispiel: --foo
lsp-cli-unknown-option = vc-lsp: unbekannte Option „{ $option }“

# LSP-Fehlerantwort auf eine nicht unterstützte Anfrage
# $method: Name der Anfrage | Beispiel: textDocument/rename
lsp-error-method-unsupported = nicht unterstützt: { $method }

# Fehler E000: Hardware-Datenbank für den Assemblerlauf nicht gefunden
diag-lsp-e000-no-data = Hardware-Datenbank (data/) nicht gefunden – initializationOptions.dataDir oder VC_DATA setzen

# Fehler E000: Zielsystem oder Prozessor konnte nicht geladen werden
# $detail: Fehlertext des Ladens | Beispiel: (Meldung)
diag-lsp-e000-load = { $detail }

# Zusatz bei Folgefehlern einer Datei, die nur im Kontext einer einbindenden Datei übersetzbar ist
# $message: ursprüngliche Meldung | Beispiel: (Meldung)
lsp-diag-context-hint = { $message } – vermutlich in der einbindenden Datei definiert

# Hover über einem Befehl: Zeile der Tabelle und Hinweise
lsp-hover-table-head =
    | Modus | Opcode | Bytes | Zyklen |
    |---|---|---|---|
# $flags: Flags mit Leerzeichen getrennt | Beispiel: N Z C
lsp-hover-flags = Flags: `{ $flags }`
lsp-hover-tag-unstable = ⚠ instabil
lsp-hover-tag-undocumented = ⚠ undok.
# $mode: Adressierungsart | Beispiel: Absolut (abs)
# $bytes: Bytes in Hex | Beispiel: AD 20 D0
# $cycles: Zahl der Zyklen | Beispiel: 4
lsp-hover-line-chosen = Diese Zeile: **{ $mode }** (vom Assembler gewählt) · Bytes `{ $bytes }` · { $cycles } Zyklen
# $why: Grund | Beispiel: noch kein Assemblerlauf
lsp-hover-line-open = Diese Zeile: Modus noch offen ({ $why })
lsp-hover-line-ambiguous = Diese Zeile: mehrere Modi (Makro oder Wiederholung), keiner hervorgehoben
lsp-hover-why-no-run = noch kein Assemblerlauf
lsp-hover-why-errors = die Datei enthält Fehler
lsp-hover-why-no-code = der Assembler hat für die Zeile keinen Code erzeugt
lsp-hover-why-ambiguous = Befehl in dieser Zeile nicht eindeutig zuzuordnen

# Hover über einem Chip-Register
# $chip: Chip | Beispiel: vic2
# $reg: Registername | Beispiel: border
# $addr: Adresse in Hex | Beispiel: $D020
# $access: Zugriff | Beispiel: lesen/schreiben
# $doc: Beschreibung aus der Hardware-Datenbank | Beispiel: Rahmenfarbe
# $name: Chipname | Beispiel: VIC-II
lsp-hover-register =
    { "" }**{ $chip }.{ $reg }** — `{ $addr }` · { $access }

    { $doc }

    { "" }*{ $name }*
lsp-hover-register-short = Register **{ $chip }.{ $reg }** – { $doc }
lsp-hover-access-read = lesen
lsp-hover-access-write = schreiben
lsp-hover-access-rw = lesen/schreiben
# $file: Dateiname | Beispiel: lib.asm
# $line: Zeilennummer | Beispiel: 12
lsp-hover-defined-in = Definiert in `{ $file }`:{ $line }

# Hover: alte Schreibweisen einer Direktive
# $names: alte Namen, kommagetrennt | Beispiel: `.pseudopc`
lsp-hover-also = Auch: { $names }

# Completion
lsp-completion-directive = Direktive
# Completion: alter Name einer Direktive
# $name: kanonischer Name | Beispiel: .relocate
lsp-completion-alias = Alter Name von { $name }

# Quick-Fix: Direktivennamen vereinheitlichen
# $old: bisheriger Name | Beispiel: .pseudopc
# $new: kanonischer Text | Beispiel: .relocate
lsp-codeaction-canonical = Kanonischen Namen verwenden: { $old } → { $new }

# $count: Anzahl der Stellen | Beispiel: 3
lsp-codeaction-canonical-all = Alle Direktivennamen der Datei vereinheitlichen ({ $count })
# $line: Zeilennummer | Beispiel: 12
lsp-completion-line = Zeile { $line }
# $params: Parameter, mit Komma getrennt | Beispiel: a, b
lsp-completion-macro = Makro({ $params })

# Hover über einem Zahlenwert: Tabelle der Schreibweisen
lsp-value-table-head =
    | Format | Wert |
    |---|---|
# $v: Wert dezimal | Beispiel: 53280
lsp-value-row-dec = | Dezimal | { $v } |
# $v: Wert hexadezimal | Beispiel: $D020
lsp-value-row-hex = | Hex | `{ $v }` |
# $v: Wert binär | Beispiel: %1101 0000 0010 0000
lsp-value-row-bin = | Binär | `{ $v }` |
# $v: Wert mit Vorzeichen | Beispiel: -56
lsp-value-row-signed8 = | Mit Vorzeichen (8 Bit) | { $v } |
# $v: Wert mit Vorzeichen | Beispiel: -1
lsp-value-row-signed16 = | Mit Vorzeichen (16 Bit) | { $v } |
# $v: Wert ohne Vorzeichen | Beispiel: 255
# $hex: derselbe Wert hexadezimal | Beispiel: $FF
lsp-value-row-unsigned8 = | 8 Bit ohne Vorzeichen | { $v } (`{ $hex }`) |
# $v: Wert ohne Vorzeichen | Beispiel: 65535
lsp-value-row-unsigned16 = | 16 Bit ohne Vorzeichen | { $v } |
# $v: Zeichen | Beispiel: A
lsp-value-row-ascii = | Zeichen (ASCII) | `'{ $v }'` |
# $hex: Low-Byte hexadezimal | Beispiel: $20
# $dec: Low-Byte dezimal | Beispiel: 32
lsp-value-row-lowbyte = | Low-Byte (`<`) | `{ $hex }` = { $dec } |
# $hex: High-Byte hexadezimal | Beispiel: $D0
# $dec: High-Byte dezimal | Beispiel: 208
lsp-value-row-highbyte = | High-Byte (`>`) | `{ $hex }` = { $dec } |

# Hover- und Completion-Texte des Sprachservers: Mnemonics und Direktiven (Namen bleiben unübersetzt)

lsp-mnemonic-adc = Addition mit Übertrag
lsp-mnemonic-and = UND mit Akkumulator
lsp-mnemonic-asl = Arithmetisch links schieben
lsp-mnemonic-bcc = Verzweigen, wenn Übertrag gelöscht
lsp-mnemonic-bcs = Verzweigen, wenn Übertrag gesetzt
lsp-mnemonic-beq = Verzweigen, wenn gleich (Z=1)
lsp-mnemonic-bit = Bits testen
lsp-mnemonic-bmi = Verzweigen, wenn negativ
lsp-mnemonic-bne = Verzweigen, wenn ungleich (Z=0)
lsp-mnemonic-bpl = Verzweigen, wenn positiv
lsp-mnemonic-brk = Software-Interrupt
lsp-mnemonic-bvc = Verzweigen, wenn kein Überlauf
lsp-mnemonic-bvs = Verzweigen bei Überlauf
lsp-mnemonic-clc = Übertrag löschen
lsp-mnemonic-cld = Dezimalmodus aus
lsp-mnemonic-cli = Interrupts freigeben
lsp-mnemonic-clv = Überlauf löschen
lsp-mnemonic-cmp = Akkumulator vergleichen
lsp-mnemonic-cpx = X-Register vergleichen
lsp-mnemonic-cpy = Y-Register vergleichen
lsp-mnemonic-dec = Speicher dekrementieren
lsp-mnemonic-dex = X dekrementieren
lsp-mnemonic-dey = Y dekrementieren
lsp-mnemonic-eor = Exklusiv-ODER mit Akkumulator
lsp-mnemonic-inc = Speicher inkrementieren
lsp-mnemonic-inx = X inkrementieren
lsp-mnemonic-iny = Y inkrementieren
lsp-mnemonic-jmp = Sprung
lsp-mnemonic-jsr = Unterprogramm aufrufen
lsp-mnemonic-lda = Akkumulator laden
lsp-mnemonic-ldx = X-Register laden
lsp-mnemonic-ldy = Y-Register laden
lsp-mnemonic-lsr = Logisch rechts schieben
lsp-mnemonic-nop = Keine Operation
lsp-mnemonic-ora = ODER mit Akkumulator
lsp-mnemonic-pha = Akkumulator auf den Stack
lsp-mnemonic-php = Statusregister auf den Stack
lsp-mnemonic-pla = Akkumulator vom Stack
lsp-mnemonic-plp = Statusregister vom Stack
lsp-mnemonic-rol = Rotieren links
lsp-mnemonic-ror = Rotieren rechts
lsp-mnemonic-rti = Rückkehr vom Interrupt
lsp-mnemonic-rts = Rückkehr vom Unterprogramm
lsp-mnemonic-sbc = Subtraktion mit Borgen
lsp-mnemonic-sec = Übertrag setzen
lsp-mnemonic-sed = Dezimalmodus an
lsp-mnemonic-sei = Interrupts sperren
lsp-mnemonic-sta = Akkumulator speichern
lsp-mnemonic-stx = X-Register speichern
lsp-mnemonic-sty = Y-Register speichern
lsp-mnemonic-tax = A nach X
lsp-mnemonic-tay = A nach Y
lsp-mnemonic-tsx = Stackzeiger nach X
lsp-mnemonic-txa = X nach A
lsp-mnemonic-txs = X nach Stackzeiger
lsp-mnemonic-tya = Y nach A
lsp-mnemonic-asr = Arithmetisch rechts schieben, Vorzeichen bleibt (45GS02)
lsp-mnemonic-asw = 16-Bit-Wert im Speicher links schieben (45GS02)
lsp-mnemonic-row = 16-Bit-Wert im Speicher links rotieren (45GS02)
lsp-mnemonic-inw = 16-Bit-Wert in der Zeropage inkrementieren (45GS02)
lsp-mnemonic-dew = 16-Bit-Wert in der Zeropage dekrementieren (45GS02)
lsp-mnemonic-phw = 16-Bit-Wert (Konstante oder Speicher) auf den Stack (45GS02)
lsp-mnemonic-bbr0 = Verzweigen, wenn Bit 0 des Zeropage-Bytes gelöscht (45GS02; BBR1 bis BBR7 entsprechend)
lsp-mnemonic-bbr1 = Verzweigen, wenn Bit 1 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbr2 = Verzweigen, wenn Bit 2 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbr3 = Verzweigen, wenn Bit 3 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbr4 = Verzweigen, wenn Bit 4 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbr5 = Verzweigen, wenn Bit 5 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbr6 = Verzweigen, wenn Bit 6 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbr7 = Verzweigen, wenn Bit 7 des Zeropage-Bytes gelöscht (45GS02)
lsp-mnemonic-bbs0 = Verzweigen, wenn Bit 0 des Zeropage-Bytes gesetzt (45GS02; BBS1 bis BBS7 entsprechend)
lsp-mnemonic-bbs1 = Verzweigen, wenn Bit 1 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-bbs2 = Verzweigen, wenn Bit 2 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-bbs3 = Verzweigen, wenn Bit 3 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-bbs4 = Verzweigen, wenn Bit 4 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-bbs5 = Verzweigen, wenn Bit 5 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-bbs6 = Verzweigen, wenn Bit 6 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-bbs7 = Verzweigen, wenn Bit 7 des Zeropage-Bytes gesetzt (45GS02)
lsp-mnemonic-rmb0 = Bit 0 im Zeropage-Byte löschen (45GS02; RMB1 bis RMB7 entsprechend)
lsp-mnemonic-rmb1 = Bit 1 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-rmb2 = Bit 2 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-rmb3 = Bit 3 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-rmb4 = Bit 4 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-rmb5 = Bit 5 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-rmb6 = Bit 6 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-rmb7 = Bit 7 im Zeropage-Byte löschen (45GS02)
lsp-mnemonic-smb0 = Bit 0 im Zeropage-Byte setzen (45GS02; SMB1 bis SMB7 entsprechend)
lsp-mnemonic-smb1 = Bit 1 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-smb2 = Bit 2 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-smb3 = Bit 3 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-smb4 = Bit 4 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-smb5 = Bit 5 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-smb6 = Bit 6 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-smb7 = Bit 7 im Zeropage-Byte setzen (45GS02)
lsp-mnemonic-bra = Immer verzweigen (45GS02)
lsp-mnemonic-bsr = Unterprogramm per 16-Bit-Sprung relativ aufrufen (45GS02)
lsp-mnemonic-lbcc = Lange Verzweigung (16 Bit), wenn Übertrag gelöscht (45GS02)
lsp-mnemonic-lbcs = Lange Verzweigung (16 Bit), wenn Übertrag gesetzt (45GS02)
lsp-mnemonic-lbeq = Lange Verzweigung (16 Bit), wenn gleich (45GS02)
lsp-mnemonic-lbne = Lange Verzweigung (16 Bit), wenn ungleich (45GS02)
lsp-mnemonic-lbmi = Lange Verzweigung (16 Bit), wenn negativ (45GS02)
lsp-mnemonic-lbpl = Lange Verzweigung (16 Bit), wenn positiv (45GS02)
lsp-mnemonic-lbvc = Lange Verzweigung (16 Bit), wenn kein Überlauf (45GS02)
lsp-mnemonic-lbvs = Lange Verzweigung (16 Bit), bei Überlauf (45GS02)
lsp-mnemonic-lbra = Lange Verzweigung (16 Bit), immer (45GS02)
lsp-mnemonic-cle = Erweitertes Stack-Flag löschen: 16-Bit-Stackzeiger (45GS02)
lsp-mnemonic-see = Erweitertes Stack-Flag setzen: 8-Bit-Stackzeiger (45GS02)
lsp-mnemonic-cpz = Z-Register vergleichen (45GS02)
lsp-mnemonic-ldz = Z-Register laden (45GS02)
lsp-mnemonic-stz = Z-Register speichern (45GS02)
lsp-mnemonic-inz = Z inkrementieren (45GS02)
lsp-mnemonic-dez = Z dekrementieren (45GS02)
lsp-mnemonic-phx = X auf den Stack (45GS02)
lsp-mnemonic-phy = Y auf den Stack (45GS02)
lsp-mnemonic-phz = Z auf den Stack (45GS02)
lsp-mnemonic-plx = X vom Stack (45GS02)
lsp-mnemonic-ply = Y vom Stack (45GS02)
lsp-mnemonic-plz = Z vom Stack (45GS02)
lsp-mnemonic-taz = A nach Z (45GS02)
lsp-mnemonic-tza = Z nach A (45GS02)
lsp-mnemonic-tab = A nach B, dem Basisseitenregister (45GS02)
lsp-mnemonic-tba = B (Basisseitenregister) nach A (45GS02)
lsp-mnemonic-tsy = High-Byte des Stackzeigers nach Y (45GS02)
lsp-mnemonic-tys = Y in das High-Byte des Stackzeigers (45GS02)
lsp-mnemonic-trb = Bits des Akkumulators im Speicher löschen und testen (45GS02)
lsp-mnemonic-tsb = Bits des Akkumulators im Speicher setzen und testen (45GS02)
lsp-mnemonic-neg = Akkumulator negieren, Zweierkomplement (45GS02)
lsp-mnemonic-eom = Keine Operation ($EA); auch Präfix für 32-Bit-Zeiger (45GS02)
lsp-mnemonic-map = Speicherabbild aus A, X, Y, Z setzen (45GS02)
lsp-mnemonic-ldq = 32-Bit-Wert Q (Z:Y:X:A) laden (45GS02)
lsp-mnemonic-stq = 32-Bit-Wert Q speichern (45GS02)
lsp-mnemonic-adcq = Q mit Übertrag addieren (45GS02)
lsp-mnemonic-sbcq = Q mit Borgen subtrahieren (45GS02)
lsp-mnemonic-andq = Q UND Speicher (45GS02)
lsp-mnemonic-orq = Q ODER Speicher (45GS02)
lsp-mnemonic-eorq = Q Exklusiv-ODER Speicher (45GS02)
lsp-mnemonic-cpq = Q vergleichen (45GS02)
lsp-mnemonic-bitq = Bits von Q testen (45GS02)
lsp-mnemonic-aslq = Q links schieben (45GS02)
lsp-mnemonic-lsrq = Q logisch rechts schieben (45GS02)
lsp-mnemonic-rolq = Q links rotieren (45GS02)
lsp-mnemonic-rorq = Q rechts rotieren (45GS02)
lsp-mnemonic-asrq = Q arithmetisch rechts schieben (45GS02)
lsp-mnemonic-inq = Q inkrementieren (45GS02)
lsp-mnemonic-deq = Q dekrementieren (45GS02)
lsp-mnemonic-negq = Q negieren (45GS02)
lsp-mnemonic-lax = A und X laden (undokumentiert)
lsp-mnemonic-sax = A AND X speichern (undokumentiert)
lsp-mnemonic-dcp = DEC, dann CMP (undokumentiert)
lsp-mnemonic-isc = INC, dann SBC (undokumentiert)
lsp-mnemonic-slo = ASL, dann ORA (undokumentiert)
lsp-mnemonic-rla = ROL, dann AND (undokumentiert)
lsp-mnemonic-sre = LSR, dann EOR (undokumentiert)
lsp-mnemonic-rra = ROR, dann ADC (undokumentiert)
lsp-mnemonic-anc = AND, Bit 7 nach Übertrag (undokumentiert)
lsp-mnemonic-alr = AND, dann LSR (undokumentiert)
lsp-mnemonic-arr = AND, dann ROR (undokumentiert)
lsp-mnemonic-axs = (A AND X) minus Operand nach X (undokumentiert)
lsp-mnemonic-las = Speicher AND Stackzeiger nach A, X, SP (undokumentiert)
lsp-mnemonic-xaa = instabil: X nach A, AND Operand (undokumentiert)
lsp-mnemonic-lxa = instabil: A und X laden (undokumentiert)
lsp-mnemonic-sha = instabil: A AND X AND (H+1) speichern (undokumentiert)
lsp-mnemonic-shx = instabil: X AND (H+1) speichern (undokumentiert)
lsp-mnemonic-shy = instabil: Y AND (H+1) speichern (undokumentiert)
lsp-mnemonic-tas = instabil: A AND X nach SP (undokumentiert)
lsp-mnemonic-jam = Prozessor bleibt stehen (undokumentiert)

lsp-directive-bank =
    ROM-Bank für das Modulformat wählen (0–1023). Jede Bank hat einen eigenen Adressraum.

    Schreibweise:

    - `.bank 1`, danach `.org $8000` setzt die Adresse in dieser Bank
lsp-directive-segment =
    Segment benennen und optional den erlaubten Bereich prüfen. Schreibweisen:

    - `.segment "CODE", $8000, $9fff` – benennen und Bereich prüfen

    - `.segment "CODE"` – nur benennen

    - `.segment` – Namen und Bereich aufheben
lsp-directive-cartridge =
    Modul statt Programm ausgeben: `.cartridge "typ", "NAME"`. Typen:

    - C64: `normal`, `ocean`, `magicdesk`, `easyflash`, `actionreplay`, `ultimax`

    - Atari: `8k`, `16k`, `xegs`

    - VC-20: `a0`, `20`, `40`, `60` (Speicherblock; rohes Abbild mit Endung .a0/.20/.40/.60)
lsp-directive-org =
    Programmzähler setzen und neues Segment beginnen. Schreibweisen:

    - `.org $0801`

    - `*= $0801` – gleichbedeutend

    - `.org next(1024)` – zur nächsten durch 1024 teilbaren Adresse
lsp-directive-byte = Bytes ausgeben: `.byte 1, $ff, "text"`
lsp-directive-word = 16-Bit-Werte (Little Endian) ausgeben: `.word label`
lsp-directive-text = ASCII-Text ausgeben: `.text "hallo"`
lsp-directive-petscii =
    Text als PETSCII ausgeben (C64). Sonderzeichen stehen in geschweiften Klammern in der Zeichenkette:

    - `{ "{" }clr{ "}" }` – Name, `{ "{" }rvs on{ "}" }` – Name mit Leerzeichen

    - `{ "{" }down*2{ "}" }` – Wiederholung

    - `{ "{" }$c1{ "}" }` oder `{ "{" }147{ "}" }` – Code hexadezimal oder dezimal
lsp-directive-screencode = Text als Bildschirmcodes des Zielsystems ausgeben (C64-Familie: VIC/TED, Atari: ANTIC)
lsp-directive-atascii =
    Text als ATASCII ausgeben (Atari; Zeilenende wird $9B). Sonderzeichen stehen in geschweiften Klammern in der Zeichenkette:

    - `{ "{" }clr{ "}" }`, `{ "{" }eol{ "}" }` – Name

    - `{ "{" }$14{ "}" }` oder `{ "{" }20{ "}" }` – Code hexadezimal oder dezimal
lsp-directive-encoding =
    Textkodierung für `.text`, `.textnull`, `.textlen`, `.textshift`. Namen: `ascii`, `petscii`, `screen`, `atascii` oder eine 256-Byte-Tabelle. Schreibweisen:

    - `.encoding "petscii"` – gilt bis zur nächsten Umschaltung

    - `.encoding "petscii" { "{" }` … `{ "}" }` – nur im Block
lsp-directive-textnull = Text mit abschließendem Nullbyte: `.textnull "hallo"` (auch `.null`)
lsp-directive-textlen = Text mit vorangestelltem Längenbyte: `.textlen "hallo"` (auch `.ptext`)
lsp-directive-textshift = Text, dessen letztes Zeichen Bit 7 gesetzt hat: `.textshift "end"` (auch `.shift`)
lsp-directive-retaddr = Wort mit Wert − 1 (für Sprünge per RTS): `.retaddr routine` (auch `.rta`)
lsp-directive-be16 = 16-Bit-Wert, High-Byte zuerst: `.be16 $1234` (auch `.dbyt`)
lsp-directive-be32 = 32-Bit-Wert, High-Byte zuerst: `.be32 $12345678`
lsp-directive-dword = 32-Bit-Wert, Low-Byte zuerst: `.dword $12345678` (auch `.long`; `.long` legt hier vier Byte ab)
lsp-directive-hex = Bytes aus Hexziffern: `.hex "aa bb0c"`
lsp-directive-var =
    Veränderbare Variable anlegen: `.var x = 0` (darf Kommazahlen und Listen wie `[1, 2]` halten). Ändern:

    - `x = x + 1`

    - `x += 2` (auch `-=`, `*=` …)

    - `.set x = …`
lsp-directive-set = Variable ändern: `.set x = x + 1` (auch `x = …`, `x += …`)
lsp-directive-for =
    Schleife, jeweils mit `.endfor` am Ende, auch `.next` (höchstens 65536 Durchläufe). Schreibweisen:

    - `.for i = 0, i < 8, i += 1` – Startwert, Bedingung, Schritt

    - `.for i in range(n)` – 0 bis n−1

    - `.for x in [1, 2, 3]` – Werte einer Liste
lsp-directive-endfor = Ende von `.for` (auch `.next`)
lsp-directive-break = Verlässt die innerste Schleife (`.for` oder `.repeat`)
lsp-directive-continue = Springt zum nächsten Durchlauf der innersten Schleife
lsp-directive-function = Einzeilige Funktion für Ausdrücke: `.function lerp(a, b, t) = a + (b - a) * t / 255`; gilt ab dieser Zeile, keine Rekursion
lsp-directive-exitmacro = Beendet das laufende Makro vorzeitig
lsp-directive-fill =
    Bytes füllen. Schreibweisen:

    - `.fill 4, $ea` – vier Bytes mit festem Wert

    - `.fill 256, i * 2` – der Ausdruck darf die Laufvariable `i` (0-basiert) benutzen

    - `.fill 8, k + 10, k` – drittes Argument: anderer Name der Laufvariablen
lsp-directive-lohitable = Tabelle aus 16-Bit-Werten, erst alle Low-Bytes, dann alle High-Bytes: `.lohitable 25, $0400 + 40 * i` (auch `.lohifill`)
lsp-directive-branchlong =
    Zu weite bedingte Sprünge automatisch verlängern (invertierter Branch + JMP, auf dem 45GS02 LBxx; Warnung W203). Schreibweisen:

    - `.branchlong auto` – einschalten

    - `.branchlong off` – ausschalten (Standard)
lsp-directive-struct =
    Struktur mit Feldern der Form `name typ [anzahl]`; definiert `Sprite.feld` (Offset) und `Sprite.size`. Feldtypen:

    - `.byte`, `.word`, `.dword`

    - eine früher definierte Struktur

    Aufbau: `.struct Sprite` … `.endstruct`
lsp-directive-endstruct = Ende von `.struct`
lsp-directive-enum =
    Aufzählung: `.enum Farbe` … `.endenum`; definiert `Farbe.name`. Einträge:

    - `name` – zählt ab 0 oder dem letzten Wert hoch

    - `name = wert` – setzt den Wert
lsp-directive-endenum = Ende von `.enum`
lsp-directive-align =
    Auf ein Vielfaches ausrichten. Schreibweisen:

    - `.align 256` – mit Nullbytes auffüllen

    - `.align 4, $ea` – mit einem Füllbyte auffüllen (hier NOP)

    - `.align 256, skip` – ohne Füllbytes springen und ein neues Segment beginnen (früher `.page`)
lsp-directive-const = Konstante definieren: `.const NAME = ausdruck`
lsp-directive-include = Datei einbinden: `.include "lib/vic2.inc"`
lsp-directive-incbin =
    Binärdatei einbinden. Schreibweisen:

    - `.incbin "datei.bin"` – ganze Datei

    - `.incbin "datei.bin", offset` – ab Offset

    - `.incbin "datei.bin", offset, länge` – Ausschnitt
lsp-directive-if = Bedingte Assemblierung: `.if ausdruck` … `.elseif` … `.else` … `.endif`
lsp-directive-ifdef = Nur assemblieren, wenn das Symbol definiert ist
lsp-directive-ifndef = Nur assemblieren, wenn das Symbol nicht definiert ist
lsp-directive-else = Alternativzweig von `.if`
lsp-directive-elseif = Weitere Bedingung von `.if`
lsp-directive-endif = Ende von `.if`
lsp-directive-scope = Namensraum öffnen: `.scope` … `.endscope` oder `{ "{" }` … `{ "}" }`
lsp-directive-endscope = Ende von `.scope` und `.namespace` (auch `{ "}" }`)
lsp-directive-namespace = Benannten Namensraum öffnen: `.namespace name { "{" }` … `{ "}" }`
lsp-directive-macro =
    Makro definieren, Ende mit `.endmacro`; `@label` wird je Aufruf eindeutig. Schreibweisen:

    - `.macro name(param, …)`

    - `.macro name(param, opt=wert)` – Parameter mit Standardwert (hinter den Pflichtparametern)

    Aufruf: `name wert, wert` oder `name(wert, wert)`
lsp-directive-endmacro = Ende einer Makrodefinition
lsp-directive-repeat =
    Block wiederholen, Ende mit `.endrepeat`. Schreibweisen:

    - `.repeat 8` – achtmal

    - `.repeat 8, i` – mit Laufvariable `i` (ab 0)
lsp-directive-endrepeat = Ende von `.repeat`
lsp-directive-relocate = Code für eine andere Laufzeitadresse assemblieren (bis `.endrelocate`); früher `.pseudopc`
lsp-directive-endrelocate = Ende von `.relocate` (auch `.realpc`, `.endpseudopc`)
lsp-directive-cycles = Zyklenbudget-Block: `.cycles name max=40` … `.endcycles`; Warnung W201 bei Überschreitung
lsp-directive-endcycles = Ende von `.cycles`
lsp-directive-basicstub =
    BASIC-Zeile `SYS <ziel>` voranstellen. Schreibweisen:

    - `.basicstub start` – Zeile 10

    - `.basicstub start, 1000` – eigene Zeilennummer 0–63999

    - `.basicstub start, 1000, "text"` – hängt `:REM TEXT` an (Kleinbuchstaben werden PETSCII-Großbuchstaben, `{ "{" }name{ "}" }`-Sonderzeichen erlaubt)

    Ohne vorheriges `.org` beginnt der Stub an der BASIC-Startadresse des Zielsystems (C64 $0801, C128 $1C01, Plus/4 und VC-20 $1001, MEGA65 $2001); mit `.org` auf einer anderen Adresse warnt W204
lsp-directive-run = Startadresse für XEX-Dateien (Atari): `.run start`
lsp-directive-assert = Bedingung prüfen: `.assert ausdruck, "meldung"`
lsp-directive-error = Fehler ausgeben
lsp-directive-warning = Warnung ausgeben
lsp-directive-target = Zielsystem im Quelltext vormerken (`.target c64` oder `.target atari8`); der Assembler ignoriert die Zeile, die IDE liest sie bei „automatisch“.
lsp-directive-cpu =
    Umgang mit undokumentierten Befehlen: `.cpu 6510 { "{" } illegals = warn { "}" }`. Werte:

    - `deny` – Fehler

    - `warn` – Warnung

    - `allow` – erlaubt
diag-export-wird-beim-bauen-erzeugt = Die Datei '{ $name }' entsteht beim Bauen aus '{ $quelle }'.
