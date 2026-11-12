// Plus/4 und C16: KERNAL-Sprungtabelle (C64-kompatibler Teil), Vektoren, RAM-Vektoren
// Konstanten für den Assembler: #include <plus4/kernal.asm>

#include <plus4/hardware.asm>

// ---- KERNAL-Sprungtabelle
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

// ---- RAM-Vektoren
.const CINV                    = $0314   // IRQ-Vektor
.const CBINV                   = $0316   // BRK-Vektor
.const NMINV                   = $0318   // NMI-Vektor
.const IOPEN                   = $031A   // OPEN
.const ICLOSE                  = $031C   // CLOSE
.const ICHKIN                  = $031E   // CHKIN
.const ICKOUT                  = $0320   // CHKOUT
.const ICLRCH                  = $0322   // CLRCHN
.const IBASIN                  = $0324   // CHRIN
.const IBSOUT                  = $0326   // CHROUT
.const ISTOP                   = $0328   // STOP
.const IGETIN                  = $032A   // GETIN
.const ICLALL                  = $032C   // CLALL
.const ILOAD                   = $0330   // LOAD
.const ISAVE                   = $0332   // SAVE
