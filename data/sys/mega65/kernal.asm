// MEGA65: KERNAL-Sprungtabelle (in allen ROM-Versionen an denselben Adressen)
// Konstanten für den Assembler: #include <mega65/kernal.asm>

// Die Sprungtabelle ist in allen MEGA65-ROMs (geschlossene 9xxxxx und OpenROMs) gleich; ROM-abhängige Adressen: rom_920418.asm, rom_open.asm.

// ---- Erweiterungen (C65/C128-artig)
.const SPIN_SPOUT              = $FF4D   // Schnelle serielle Ports
.const CLOSE_ALL               = $FF50   // Alle Dateien eines Geräts schließen
.const C64MODE                 = $FF53   // In den C64-Modus wechseln
.const MONITOR_CALL            = $FF56   // Monitor aufrufen
.const BOOTSYS                 = $FF59   // Bootsektor laden und starten
.const PHOENIX                 = $FF5C   // Funktionsmodule starten
.const LKUPLA                  = $FF5F   // Logische Dateinummer nachschlagen
.const LKUPSA                  = $FF62   // Sekundäradresse nachschlagen
.const SWAPPER                 = $FF65   // Bildschirmbreite umschalten
.const PFKEY                   = $FF68   // Funktionstaste belegen
.const SETBNK                  = $FF6B   // Bank für LOAD/SAVE setzen
.const JSR_FAR                 = $FF6E   // Unterprogramm in anderer Bank aufrufen
.const JMP_FAR                 = $FF71   // In andere Bank springen
.const LDA_FAR                 = $FF74   // Byte aus anderer Bank holen
.const STA_FAR                 = $FF77   // Byte in andere Bank schreiben
.const CMP_FAR                 = $FF7A   // Byte aus anderer Bank vergleichen
.const PRIMM                   = $FF7D   // Text hinter dem JSR ausgeben

// ---- C64-kompatibler Teil
.const CINT                    = $FF81   // Bildschirm-Editor initialisieren
.const IOINIT                  = $FF84   // E/A-Bausteine initialisieren
.const RAMTAS                  = $FF87   // RAM-Test und Zeiger setzen
.const RESTOR                  = $FF8A   // Vektoren auf Standard
.const VECTOR                  = $FF8D   // Vektoren lesen/setzen
.const SETMSG                  = $FF90   // Systemmeldungen steuern
.const SECOND                  = $FF93   // Sekundäradresse nach LISTEN
.const TKSA                    = $FF96   // Sekundäradresse nach TALK
.const MEMTOP                  = $FF99   // Speicherobergrenze lesen/setzen
.const MEMBOT                  = $FF9C   // Speicheruntergrenze lesen/setzen
.const SCNKEY                  = $FF9F   // Tastatur abtasten
.const SETTMO                  = $FFA2   // IEEE-Timeout
.const ACPTR                   = $FFA5   // Byte vom IEC-Bus holen
.const CIOUT                   = $FFA8   // Byte auf IEC-Bus senden
.const UNTLK                   = $FFAB   // UNTALK senden
.const UNLSN                   = $FFAE   // UNLISTEN senden
.const LISTEN                  = $FFB1   // Gerät als Hörer aufrufen
.const TALK                    = $FFB4   // Gerät als Sprecher aufrufen
.const READST                  = $FFB7   // Statuswort lesen
.const SETLFS                  = $FFBA   // Logische Datei, Gerät, Sekundäradresse setzen
.const SETNAM                  = $FFBD   // Dateinamen setzen
.const OPEN                    = $FFC0   // Datei öffnen
.const CLOSE                   = $FFC3   // Datei schließen
.const CHKIN                   = $FFC6   // Eingabekanal setzen
.const CHKOUT                  = $FFC9   // Ausgabekanal setzen
.const CLRCHN                  = $FFCC   // Kanäle zurücksetzen
.const CHRIN                   = $FFCF   // Zeichen holen
.const CHROUT                  = $FFD2   // Zeichen ausgeben
.const LOAD                    = $FFD5   // Datei laden
.const SAVE                    = $FFD8   // Speicher sichern
.const SETTIM                  = $FFDB   // Systemuhr setzen
.const RDTIM                   = $FFDE   // Systemuhr lesen
.const STOP                    = $FFE1   // STOP-Taste prüfen
.const GETIN                   = $FFE4   // Taste aus Puffer holen
.const CLALL                   = $FFE7   // Alle Dateien schließen
.const UDTIM                   = $FFEA   // Systemuhr weiterzählen
.const SCREEN                  = $FFED   // Bildschirmgröße holen
.const PLOT                    = $FFF0   // Cursorposition lesen/setzen
.const IOBASE                  = $FFF3   // Adresse der E/A-Bausteine (CIA 1)

// ---- Hardwarevektoren
.const VEC_NMI                 = $FFFA   // NMI-Vektor
.const VEC_RESET               = $FFFC   // RESET-Vektor
.const VEC_IRQ                 = $FFFE   // IRQ/BRK-Vektor
