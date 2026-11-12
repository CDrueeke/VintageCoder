// VC-20: KERNAL-Sprungtabelle (wie C64, aber ohne CINT/IOINIT/RAMTAS), Hardwarevektoren, Systemvariablen und RAM-Vektoren
// Konstanten für den Assembler: #include <vic20/kernal.asm>

#include <vic20/hardware.asm>

// ---- Offizielle KERNAL-Sprungtabelle (beim VC-20 erst ab RESTOR; CINT, IOINIT und RAMTAS fehlen dort, siehe KERNAL-ROM-Routinen)
.const RESTOR                = $FF8A   // Vektoren auf Standard
.const VECTOR                = $FF8D   // Vektoren lesen/setzen
.const SETMSG                = $FF90   // Systemmeldungen steuern
.const SECOND                = $FF93   // Sekundäradresse nach LISTEN
.const TKSA                  = $FF96   // Sekundäradresse nach TALK
.const MEMTOP                = $FF99   // Speicherobergrenze lesen/setzen
.const MEMBOT                = $FF9C   // Speicheruntergrenze lesen/setzen
.const SCNKEY                = $FF9F   // Tastatur abtasten
.const SETTMO                = $FFA2   // IEEE-Timeout
.const ACPTR                 = $FFA5   // Byte vom IEC-Bus holen
.const CIOUT                 = $FFA8   // Byte auf IEC-Bus senden
.const UNTLK                 = $FFAB   // UNTALK senden
.const UNLSN                 = $FFAE   // UNLISTEN senden
.const LISTEN                = $FFB1   // Gerät als Hörer aufrufen
.const TALK                  = $FFB4   // Gerät als Sprecher aufrufen
.const READST                = $FFB7   // Statuswort lesen
.const SETLFS                = $FFBA   // Logische Datei, Gerät, Sekundäradresse setzen
.const SETNAM                = $FFBD   // Dateinamen setzen
.const OPEN                  = $FFC0   // Datei öffnen
.const CLOSE                 = $FFC3   // Datei schließen
.const CHKIN                 = $FFC6   // Eingabekanal setzen
.const CHKOUT                = $FFC9   // Ausgabekanal setzen
.const CLRCHN                = $FFCC   // Kanäle zurücksetzen
.const CHRIN                 = $FFCF   // Zeichen holen
.const CHROUT                = $FFD2   // Zeichen ausgeben
.const LOAD                  = $FFD5   // Datei laden
.const SAVE                  = $FFD8   // Speicher sichern
.const SETTIM                = $FFDB   // Systemuhr setzen
.const RDTIM                 = $FFDE   // Systemuhr lesen
.const STOP                  = $FFE1   // STOP-Taste prüfen
.const GETIN                 = $FFE4   // Taste aus Puffer holen
.const CLALL                 = $FFE7   // Alle Dateien schließen
.const UDTIM                 = $FFEA   // Systemuhr weiterzählen
.const SCREEN                = $FFED   // Bildschirmgröße holen
.const PLOT                  = $FFF0   // Cursorposition lesen/setzen
.const IOBASE                = $FFF3   // Adresse der E/A-Bausteine (VIA 1)

// ---- Hardwarevektoren
.const VEC_NMI               = $FFFA   // NMI-Vektor
.const VEC_RESET             = $FFFC   // RESET-Vektor
.const VEC_IRQ               = $FFFE   // IRQ/BRK-Vektor

// ---- KERNAL-Nullseite
.const STATUS                = $0090   // Statuswort (ST)
.const STKEY                 = $0091   // STOP-Taste (Flag)
.const LDTND                 = $0098   // Anzahl offener Dateien
.const DFLTN                 = $0099   // Eingabegerät
.const DFLTO                 = $009A   // Ausgabegerät
.const TIME                  = $00A0   // Systemuhr (3 Byte, 1/60 s)
.const FNLEN                 = $00B7   // Länge des Dateinamens
.const LA                    = $00B8   // Aktuelle logische Dateinummer
.const SA                    = $00B9   // Aktuelle Sekundäradresse
.const FA                    = $00BA   // Aktuelle Gerätenummer
.const FNADR                 = $00BB   // Zeiger auf Dateinamen
.const LSTX                  = $00C5   // Zuletzt gedrückte Taste
.const NDX                   = $00C6   // Anzahl Zeichen im Tastaturpuffer
.const RVS                   = $00C7   // Revers-Flag
.const SFDX                  = $00CB   // Aktuell gedrückte Taste
.const BLNSW                 = $00CC   // Cursorblinken aus (0 = blinkt)
.const PNT                   = $00D1   // Zeiger auf aktuelle Bildschirmzeile
.const PNTR                  = $00D3   // Cursor-Spalte
.const QTSW                  = $00D4   // Anführungszeichen-Modus
.const TBLX                  = $00D6   // Cursor-Zeile
.const USER                  = $00F3   // Zeiger auf aktuelle Farb-RAM-Zeile

// ---- KERNAL-RAM
.const BUF                   = $0200   // BASIC-Eingabepuffer (89 Byte)
.const KEYD                  = $0277   // Tastaturpuffer (10 Byte)
.const MEMSTR                = $0281   // Start des RAM für das Betriebssystem
.const MEMSIZ                = $0283   // Ende des RAM für das Betriebssystem
.const COLOR                 = $0286   // Aktuelle Zeichenfarbe
.const GDCOL                 = $0287   // Farbe unter dem Cursor
.const HIBASE                = $0288   // Bildschirmspeicher (High-Byte)
.const XMAX                  = $0289   // Größe des Tastaturpuffers
.const RPTFLAG               = $028A   // Tastenwiederholung
.const SHFLAG                = $028D   // SHIFT/CTRL/C=-Flags
.const MODE                  = $0291   // Zeichensatzumschaltung erlaubt
.const TBUFFR                = $033C   // Kassettenpuffer (192 Byte)

// ---- RAM-Vektoren (umlenkbar)
.const CINV                  = $0314   // IRQ-Vektor
.const CBINV                 = $0316   // BRK-Vektor
.const NMINV                 = $0318   // NMI-Vektor
.const IOPEN                 = $031A   // OPEN
.const ICLOSE                = $031C   // CLOSE
.const ICHKIN                = $031E   // CHKIN
.const ICKOUT                = $0320   // CHKOUT
.const ICLRCH                = $0322   // CLRCHN
.const IBASIN                = $0324   // CHRIN
.const IBSOUT                = $0326   // CHROUT
.const ISTOP                 = $0328   // STOP
.const IGETIN                = $032A   // GETIN
.const ICLALL                = $032C   // CLALL
.const ILOAD                 = $0330   // LOAD
.const ISAVE                 = $0332   // SAVE

// ---- KERNAL-ROM-Routinen (Adressen der KERNAL-Versionen 901486-06 und -07)
.const RAMTAS                = $FD8D   // RAM prüfen, Zeiger und Zeropage setzen (nicht in der Sprungtabelle)
.const IOINIT                = $FDF9   // VIC-I und VIA initialisieren (nicht in der Sprungtabelle)
.const CINT                  = $E518   // Bildschirm-Editor initialisieren (nicht in der Sprungtabelle)
.const CLRSCR                = $E55F   // Bildschirm löschen
.const HOME                  = $E581   // Cursor links oben
.const IRQ_STD               = $EABF   // Standard-IRQ-Routine
.const IRQ_ACK               = $EB15   // IRQ beenden (VIA-2-Timer quittieren, Register zurück, RTI)
.const IRQ_EXIT              = $EB18   // IRQ beenden (Register zurück, RTI)
