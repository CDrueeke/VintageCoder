// BASIC→Assembler (basic2asm), Laufzeit Atari 8 Bit: Nullseite, Variablen und Speichergrenzen.
// Einbinden mit: #include <atari8bit/b2a/rt_zp.asm> (am Anfang des Programms, vor allem Code); danach #include <atari8bit/os.asm> ist bereits geschehen.
//
// Die Laufzeit nutzt keine Routine des Atari-BASIC-Moduls; Fließkomma (Stufe 2) rechnet mit dem Fließkommapaket des OS (FR0 $D4-$D9, FR1 $E0-$E5 und die Hilfszellen $DA-$DF, $E6-$FF). Die Nullseite von $80 an gehört
// dem BASIC-Modul; wird das Programm als XEX gestartet, braucht niemand diese Zellen mehr. Der Block wird trotzdem beim Start gesichert und
// bei Ende zurückgeschrieben (RT_ZSAVE), damit ein Rücksprung ins DOS oder BASIC die Zeiger wiederfindet.

.const B2A_ATARI = 1
.const RT_ZSAVE = 1
.const RT_ZLO   = $80
.const RT_ZLEN  = 20
.const rt_t0   = $80     // Zeiger/Zwischenspeicher (2 Byte)
.const rt_ap   = $82     // Zeiger auf Feld-Kopf (2 Byte)
.const rt_dp   = $84     // DATA-Zeiger (2 Byte)
.const rt_s1   = $86     // Zeichenketten: Quell-/Hilfszeiger (2 Byte)
.const rt_s2   = $88     // Zeichenketten: Zielzeiger (2 Byte)
.const rt_gd   = $8A     // Speicherbereinigung: Beschreiber unter Prüfung (2 Byte)
.const rt_gq   = $8C     // Speicherbereinigung: Tabellenzeiger (2 Byte)
.const RT_FS   = 6     // Bytes je Zahl (6-Byte-BCD des OS-Pakets)
.const RT_SGN  = 0     // Vorzeichenbit im Byte 0
.const F_FACEXP = FR0  // Byte 0 des FAC: 0 = Zahl ist 0
.const MATH_ZP = $8E     // 6 Byte für mul16/div16 (common/math.asm): $8E-$93

// Variablen in Seite 6 ($0600-$06FF ist für Anwenderprogramme frei)
.const RT_VARS = $0600
.const rt_t1   = RT_VARS+0    // Zwischenspeicher (2 Byte)
.const rt_fv   = RT_VARS+2    // Schleifenvariable (2 Byte)
.const rt_bp   = RT_VARS+4    // Sprungziel für FOR/GOSUB (2 Byte)
.const rt_v    = RT_VARS+6    // Zwischenspeicher (2 Byte)
.const rt_esp  = RT_VARS+8    // Ausdrucksstapel-Zeiger
.const rt_ssp  = RT_VARS+9    // Stapelzeiger für GOSUB/FOR
.const rt_sp0  = RT_VARS+10   // CPU-Stapelzeiger beim Start (für END)
.const rt_tmp  = RT_VARS+11   // Zwischenspeicher (1 Byte)
.const rt_line = RT_VARS+12   // aktuelle BASIC-Zeile (2 Byte, für Fehlermeldungen)
.const rt_txt  = RT_VARS+14   // Zwischenspeicher (2 Byte, VAL)

// Obergrenze des Programms und des Zeichenketten-Heaps: darüber liegen Anzeigeliste und Bildschirmspeicher (GRAPHICS 0 am Ende des RAMs)
.const RT_HEAPTOP = $9800
