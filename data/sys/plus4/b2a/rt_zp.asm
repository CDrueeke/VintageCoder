// BASIC→Assembler (basic2asm), Laufzeit Plus/4 (BASIC 3.5): Nullseite, Variablen, Speichergrenzen und BASIC-ROM-Einsprünge.
// Einbinden mit: #include <plus4/b2a/rt_zp.asm> (am Anfang des Programms, vor allem Code)
//
// Das Programm ruft KERNAL-Routinen (CHROUT, CHRIN, GETIN, PLOT, RDTIM, SETTIM) und – nur bei Fließkomma (RT_FLOAT) – Rechenroutinen des BASIC-ROMs auf.
// Gemessen in VICE: KERNAL und Interrupt verändern die Nullseite von $02 bis $8F nicht. Der Block des Programms wird beim Start gesichert
// und bei Ende (und bei einem Fehler des ROMs, über den Fehlervektor $0300) zurückgeschrieben, damit das BASIC seine Zeiger wiederfindet.

.const RT_ZSAVE = 1

.ifdef RT_FLOAT
// ---- Stufe 2: die Rechenroutinen des ROMs benutzen $22-$29, $47, $49-$4A, $56, $5D-$72, $78 (gemessen aus dem ROM) – der Block liegt davor
.const RT_ZLO   = $30
.const RT_ZLEN  = 14
.const rt_t0   = $30     // Zeiger/Zwischenspeicher (2 Byte)
.const rt_ap   = $32     // Zeiger auf Feld-Kopf (2 Byte)
.const rt_dp   = $34     // DATA-Zeiger (2 Byte)
.const rt_s1   = $36     // Zeichenketten: Quell-/Hilfszeiger (2 Byte)
.const rt_s2   = $38     // Zeichenketten: Zielzeiger (2 Byte)
.const rt_gd   = $3A     // Speicherbereinigung: Beschreiber unter Prüfung (2 Byte)
.const rt_gq   = $3C     // Speicherbereinigung: Tabellenzeiger (2 Byte)
.else
// ---- Stufe 1 und 3: kein ROM-Aufruf, Block $60-$73
.const RT_ZLO   = $60
.const RT_ZLEN  = 20
.const rt_t0   = $60     // Zeiger/Zwischenspeicher (2 Byte)
.const rt_ap   = $62     // Zeiger auf Feld-Kopf (2 Byte)
.const rt_dp   = $64     // DATA-Zeiger (2 Byte)
.const rt_s1   = $66     // Zeichenketten: Quell-/Hilfszeiger (2 Byte)
.const rt_s2   = $68     // Zeichenketten: Zielzeiger (2 Byte)
.const rt_gd   = $6A     // Speicherbereinigung: Beschreiber unter Prüfung (2 Byte)
.const rt_gq   = $6C     // Speicherbereinigung: Tabellenzeiger (2 Byte)
.const MATH_ZP = $6E     // 6 Byte für mul16/div16 (common/math.asm): $6E-$73
.endif

// Variablen im Kassettenpuffer (frei, solange kein Band benutzt wird)
.const RT_VARS = $0333
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

// Obergrenze des Zeichenketten-Heaps und des Programms: oberhalb liegt das BASIC-ROM
.const RT_HEAPTOP = $8000

.ifdef RT_FLOAT
// ---- BASIC-ROM-Routinen für Stufe 2 (BASIC 3.5 (ROM ab $8000)). Rechenwerke: erst ARG aus dem Speicher (A/Y) laden (F_CONUPK), dann die "T"-Einsprünge.
.const F_FSUBT    = $9E87   // FAC = ARG - FAC
.const F_FADDT    = $9E9E   // FAC = ARG + FAC
.const F_FMULTT   = $A07B   // FAC = ARG * FAC
.const F_FDIVT    = $A197   // FAC = ARG / FAC
.const F_CONUPK   = $A0DC   // ARG = (A/Y)
.const F_FPWRT    = $A5EE   // FAC = ARG ^ FAC
.const F_MOVFM    = $A222   // FAC = (A/Y)
.const F_MOVMF    = $A259   // (X/Y) = FAC (gerundet)
.const F_FCOMP    = $A2E0   // vergleicht FAC mit (A/Y): 0 gleich, 1 FAC größer, $FF FAC kleiner
.const F_ROUND    = $A2A0
.const F_NEGOP    = $A627
.const F_ABS      = $A2DD
.const F_SGN      = $A2BE
.const F_INT      = $A358
.const F_SQR      = $A5E4
.const F_LOG      = $A01E
.const F_EXP      = $A660
.const F_SIN      = $AA77
.const F_COS      = $AA70
.const F_TAN      = $AAC0
.const F_ATN      = $AB1A
.const F_RND      = $A707
.const F_FOUT     = $A46F   // FAC -> Text, A/Y = Zeiger
.const F_FIN      = $A37F   // Text ab TXTPTR -> FAC (A = erstes Zeichen)
.const F_FACEXP  = $61     // Exponent des FAC (0 = Zahl ist 0)
.const F_CHRGOT  = $0479   // CHRGOT (Textzeichen holen, Flags wie CHRGET)
.const F_TXTPTR  = $3B     // BASIC-Textzeiger
.endif
