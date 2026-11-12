// PET/CBM: KERNAL-Sprungtabelle (BASIC 2 und 4)
// Konstanten für den Assembler: #include <pet/kernal.asm>

#include <pet/hardware.asm>

.const OPEN                  = $FFC0   // Datei öffnen
.const CLOSE                 = $FFC3   // Datei schließen
.const CHKIN                 = $FFC6   // Eingabekanal setzen
.const CHKOUT                = $FFC9   // Ausgabekanal setzen
.const CLRCHN                = $FFCC   // Kanäle zurücksetzen
.const CHRIN                 = $FFCF   // Zeichen holen
.const CHROUT                = $FFD2   // Zeichen ausgeben
.const LOAD                  = $FFD5   // Datei laden
.const SAVE                  = $FFD8   // Speicher sichern
.const VERIFY                = $FFDB   // Vergleichen
.const SYS                   = $FFDE   // SYS-Einsprung
.const STOP                  = $FFE1   // STOP-Taste prüfen
.const GETIN                 = $FFE4   // Taste aus Puffer holen
.const CLALL                 = $FFE7   // Alle Dateien schließen
.const UDTIM                 = $FFEA   // Systemuhr weiterzählen
