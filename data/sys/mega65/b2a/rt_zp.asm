// BASIC→Assembler (basic2asm), Laufzeit MEGA65 (BASIC 65, Programm bei $2001): Nullseite, Variablen und Speichergrenzen. Nur Stufe 1 und 3 (Ganzzahl und
// Zeichenketten): das ROM bietet keine verwendbaren Fließkommaroutinen (siehe Konzept, Abschnitt 10.4 und rom-disasm).
// Einbinden mit: #include <mega65/b2a/rt_zp.asm> (am Anfang des Programms, vor allem Code)
//
// In Xemu (Windows) geprüft, am Gerät NICHT: Der Test `probe_zeropage` in crates/vc-ide/tests/b2a_mega65_live.rs misst, dass KERNAL und Interrupt im Lauf nur
// $C4/$C5, $DA, $E0/$E1 und $EA/$EB ändern; der gewählte Block ($40-$53, wie im Beispiel hello.asm) und die Variablen ($1F00, unterhalb des BASIC-Textes) sind frei. Der Block des Programms wird beim Start gesichert und am Ende zurückgeschrieben (wie bei C128 und Plus/4).

.const RT_ZSAVE = 1
.const RT_ZLO   = $40
.const RT_ZLEN  = 20
.const rt_t0   = $40     // Zeiger/Zwischenspeicher (2 Byte)
.const rt_ap   = $42     // Zeiger auf Feld-Kopf (2 Byte)
.const rt_dp   = $44     // DATA-Zeiger (2 Byte)
.const rt_s1   = $46     // Zeichenketten: Quell-/Hilfszeiger (2 Byte)
.const rt_s2   = $48     // Zeichenketten: Zielzeiger (2 Byte)
.const rt_gd   = $4A     // Speicherbereinigung: Beschreiber unter Prüfung (2 Byte)
.const rt_gq   = $4C     // Speicherbereinigung: Tabellenzeiger (2 Byte)
.const MATH_ZP = $4E     // 6 Byte für mul16/div16 (common/math.asm): $4E-$53

// Variablen im RAM unterhalb des BASIC-Textes ($2001)
.const RT_VARS = $1F00
.const rt_t1   = RT_VARS+0    // Zwischenspeicher (2 Byte)
.const rt_fv   = RT_VARS+2    // Schleifenvariable (2 Byte)
.const rt_bp   = RT_VARS+4    // Sprungziel für FOR/GOSUB (2 Byte)
.const rt_v    = RT_VARS+6    // Zwischenspeicher (2 Byte)
.const rt_esp  = RT_VARS+8    // Ausdrucksstapel-Zeiger
.const rt_ssp  = RT_VARS+9    // Stapelzeiger für GOSUB/FOR
.const rt_sp0  = RT_VARS+10   // CPU-Stapelzeiger beim Start (für END)
.const rt_tmp  = RT_VARS+11   // Zwischenspeicher (1 Byte)
.const rt_line = RT_VARS+12   // aktuelle BASIC-Zeile (2 Byte, für Fehlermeldungen)
.const rt_txt  = RT_VARS+14   // gesicherter BASIC-Textzeiger (2 Byte, INPUT/VAL)

// Obergrenze des Zeichenketten-Heaps: unterhalb der E/A ($D000) und des ROM-Fensters
.const RT_HEAPTOP = $7F00
