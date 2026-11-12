// C64: BASIC-Nullseite, Vektoren und ROM-Routinen
// Konstanten für den Assembler: #include <c64/basic.asm>

// ---- Zeiger
.const TXTTAB                = $002B   // Start des BASIC-Programms
.const VARTAB                = $002D   // Start der einfachen Variablen
.const ARYTAB                = $002F   // Start der Felder
.const STREND                = $0031   // Ende der Felder
.const FRETOP                = $0033   // Untergrenze des Stringbereichs
.const FRESPC                = $0035   // Zeiger für neue Strings
.const MEMSIZ_B              = $0037   // Obergrenze des BASIC-Speichers
.const CURLIN                = $0039   // Aktuelle Zeilennummer
.const OLDLIN                = $003B   // Zeile bei STOP/END
.const OLDTXT                = $003D   // Textzeiger bei STOP/END
.const DATLIN                = $003F   // Zeile des aktuellen DATA
.const DATPTR                = $0041   // Zeiger auf aktuelles DATA
.const TXTPTR                = $007A   // BASIC-Textzeiger (CHRGET)

// ---- Gleitkomma-Akkumulatoren
.const FACEXP                = $0061   // FAC1: Exponent
.const FACHO                 = $0062   // FAC1: Mantisse (4 Byte)
.const FACSGN                = $0066   // FAC1: Vorzeichen
.const ARGEXP                = $0069   // FAC2: Exponent
.const ARGSGN                = $006E   // FAC2: Vorzeichen
.const LINNUM                = $0014   // Ganzzahl-Zwischenspeicher (Zeilennummer, 2 Byte)

// ---- Textzeiger-Routinen im RAM
.const CHRGET                = $0073   // Nächstes Zeichen holen
.const CHRGOT                = $0079   // Aktuelles Zeichen holen

// ---- BASIC-Vektoren
.const IERROR                = $0300   // Fehlermeldung
.const IMAIN                 = $0302   // Hauptschleife (Eingabe)
.const ICRNCH                = $0304   // Tokenisieren
.const IQPLOP                = $0306   // Zeilen auflisten
.const IGONE                 = $0308   // Befehl ausführen
.const IEVAL                 = $030A   // Ausdruck auswerten
.const SAREG                 = $030C   // A-Register für SYS
.const SXREG                 = $030D   // X-Register für SYS
.const SYREG                 = $030E   // Y-Register für SYS
.const SPREG                 = $030F   // Statusregister für SYS

// ---- BASIC-ROM-Routinen
.const STROUT                = $AB1E   // String ausgeben (Adresse in A/Y, mit 0 beendet)
.const LINPRT                = $BDCD   // Ganzzahl in X/A ausgeben
.const GIVAYF                = $B391   // Ganzzahl in Y/A nach FAC1
.const FACINX                = $B1AA   // FAC1 nach Ganzzahl in Y/A
.const MOVMF                 = $BBD4   // FAC1 nach (X/Y) speichern
.const MOVFM                 = $BBA2   // FAC1 aus (A/Y) laden
.const FOUT                  = $BDDD   // FAC1 in Text wandeln
