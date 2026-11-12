// C128: KERNAL-Sprungtabelle (C128-Erweiterungen und C64-Teil), Vektoren, Bank-Hilfen
// Konstanten für den Assembler: #include <c128/kernal.asm>

#include <c128/hardware.asm>

// ---- C128-Erweiterungen der Sprungtabelle
.const SPIN_SPOUT              = $FF47   // Schnelle serielle Ports für Ein-/Ausgabe einrichten
.const CLOSE_ALL               = $FF4A   // Alle Dateien eines Geräts schließen
.const C64MODE                 = $FF4D   // In den C64-Modus wechseln
.const DMA_CALL                = $FF50   // DMA-Dienst aufrufen
.const BOOT_CALL               = $FF53   // Bootsektor laden und starten
.const PHOENIX                 = $FF56   // Funktionsmodule starten
.const LKUPLA                  = $FF59   // Logische Dateinummer nachschlagen
.const LKUPSA                  = $FF5C   // Sekundäradresse nachschlagen
.const SWAPPER                 = $FF5F   // Bildschirm 40/80 umschalten
.const DLCHR                   = $FF62   // Zeichensatz in den VDC laden
.const PFKEY                   = $FF65   // Funktionstaste belegen
.const SETBNK                  = $FF68   // Bank für LOAD/SAVE/OPEN setzen
.const GETCFG                  = $FF6B   // Konfigurationswert zu einer Bank holen
.const JSRFAR                  = $FF6E   // Unterprogramm in anderer Bank aufrufen
.const JMPFAR                  = $FF71   // In andere Bank springen
.const INDFET                  = $FF74   // Byte aus anderer Bank holen (LDA)
.const INDSTA                  = $FF77   // Byte in andere Bank schreiben (STA)
.const INDCMP                  = $FF7A   // Byte aus anderer Bank vergleichen (CMP)
.const PRIMM                   = $FF7D   // Text hinter dem JSR ausgeben

// ---- C64-kompatibler Teil der Sprungtabelle
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

// ---- Systemvariablen
.const STATUS                  = $0090   // Statuswort (ST)
.const FA                      = $00BA   // Aktuelle Gerätenummer
.const MODE                    = $00D7   // Bildschirmmodus (Bit 7: 80 Spalten)

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

// ---- Routinen für Zugriffe auf andere Bänke (im gemeinsamen RAM)
.const FETCH                   = $02A2   // Byte holen
.const FETVEC                  = $02AA   // Adresszeiger für FETCH
.const STASH                   = $02AF   // Byte schreiben
.const STAVEC                  = $02B9   // Adresszeiger für STASH
.const CMPARE                  = $02BE   // Byte vergleichen
.const CMPVEC                  = $02C8   // Adresszeiger für CMPARE

// ---- Makros
// Konfiguration direkt über das immer sichtbare Register $FF00 setzen (z. B. mmu_config MMUCFG_RAM0_IO)
.macro mmu_config(cfg)
 lda #cfg
 sta MMU_CR_FF00
.endmacro

// Byte aus anderer Bank holen: ptr = Nullseitenadresse des 16-Bit-Zeigers, bank = Bankwert; Index in Y, Ergebnis in A
.macro fetch_far(ptr, bank)
 lda #ptr
 ldx #bank
 jsr INDFET
.endmacro
