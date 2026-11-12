// BASIC→Assembler (basic2asm), Laufzeit C64: Nullseite und Konstanten.
// Das Ergebnis ruft kein BASIC-ROM auf; die Zeiger des BASIC ($2B-$38, $73-$8A) bleiben unberührt, damit "END" sauber ins BASIC zurückkehrt.
// Einbinden mit: #include <c64/b2a/rt_zp.asm> (am Anfang des Programms, vor allem Code)

.ifdef RT_FLOAT
// ---- Stufe 2 (Fließkomma): das BASIC-ROM rechnet mit $22-$2A und $57-$70 (FAC, ARG, Hilfsfelder) – diese Bereiche bleiben frei.
// Nur Zeiger, die indirekt benutzt werden, liegen in der Nullseite ($F7-$FE); alles andere im Kassettenpuffer ab $0334.
.const rt_t0   = $FB     // Zeiger/Zwischenspeicher (2 Byte, Nullseite)
.const rt_ap   = $FD     // Zeiger auf Feld-Kopf (2 Byte, Nullseite)
.const rt_dp   = $F7     // DATA-Zeiger (2 Byte, Nullseite)
.const rt_line = $39     // = CURLIN des BASIC: Fehlermeldungen des ROMs nennen so die richtige Zeile
.const rt_t1   = $0334   // Zwischenspeicher (2 Byte)
.const rt_fv   = $0336   // Zeiger auf Schleifenvariable (2 Byte)
.const rt_bp   = $0338   // Sprungziel für FOR/GOSUB (2 Byte)
.const rt_v    = $033A   // Zwischenspeicher (2 Byte)
.const rt_esp  = $033C   // Ausdrucksstapel-Zeiger (Bytes)
.const rt_ssp  = $033D   // Stapelzeiger für GOSUB/FOR
.const rt_sp0  = $033E   // CPU-Stapelzeiger beim Start (für END)
.const rt_tmp  = $033F   // Zwischenspeicher (1 Byte)
.const rt_txt  = $0340   // gesicherter BASIC-Textzeiger (2 Byte, INPUT)
.else
.const rt_t0   = $22     // Zwischenspeicher 0 (2 Byte): linker Operand, Zeiger
.const rt_t1   = $24     // Zwischenspeicher 1 (2 Byte): rechter Operand
.const rt_ap   = $26     // Zeiger auf Feld-Kopf (2 Byte)
.const rt_fv   = $28     // Zeiger auf Schleifenvariable (2 Byte)
.const MATH_ZP = $57     // 6 Byte für mul16/div16 (common/math.asm): $57-$5C
.const rt_esp  = $5D     // Ausdrucksstapel-Zeiger
.const rt_ssp  = $5E     // Stapelzeiger für GOSUB/FOR
.const rt_sp0  = $5F     // CPU-Stapelzeiger beim Start (für END)
.const rt_tmp  = $60     // Zwischenspeicher (1 Byte)
.const rt_line = $F7     // aktuelle BASIC-Zeile (2 Byte, für Fehlermeldungen)
.const rt_v    = $F9     // Zwischenspeicher (2 Byte)
.const rt_bp   = $FB     // Sprungziel für FOR/GOSUB (2 Byte)
.const rt_dp   = $FD     // DATA-Zeiger (2 Byte)

.endif

// Zeichenketten (Stufe 3, rt_str.asm): Zeiger aus dem BASIC-Bereich der Nullseite, den weder Stufe 1 noch das Fließkomma-ROM braucht
.const rt_s1   = $3F     // Quell-/Hilfszeiger (2 Byte)
.const rt_s2   = $41     // Zielzeiger (2 Byte)
.const rt_gd   = $43     // Speicherbereinigung: Beschreiber unter Prüfung (2 Byte)
.const rt_gq   = $45     // Speicherbereinigung: Tabellenzeiger (2 Byte)

// Obergrenze des Zeichenketten-Heaps (Stufe 3) und Grenze, bis zu der das Programm reichen darf: der Heap wächst von hier abwärts
.const RT_HEAPTOP = $A000

// ---- BASIC-ROM-Routinen für Stufe 2 (Fließkomma, FAC = Gleitkomma-Akkumulator 1 bei $61)
// Rechenwerke: erst ARG aus dem Speicher (A/Y) laden (F_CONUPK), dann die "T"-Einsprünge: FAC = ARG op FAC. Auf dem C128 lesen
// die Speicherformen (FADD usw.) aus Bank 1; CONUPK liest aus dem aktuellen Speicher, daher gilt dieses Muster auf allen Zielen.
.const F_FSUBT   = $B853   // FAC = ARG - FAC
.const F_FADDT   = $B86A   // FAC = ARG + FAC
.const F_FMULTT  = $BA2B   // FAC = ARG * FAC
.const F_FDIVT   = $BB12   // FAC = ARG / FAC
.const F_CONUPK  = $BA8C   // ARG = (A/Y)
.const F_FPWRT   = $BF7B   // FAC = ARG ^ FAC
.const F_MOVFM   = $BBA2   // FAC = (A/Y)
.const F_MOVMF   = $BBD4   // (X/Y) = FAC (gerundet)
.const F_FCOMP   = $BC5B   // vergleicht FAC mit (A/Y): 0 gleich, 1 FAC größer, $FF FAC kleiner
.const F_ROUND   = $BC1B
.const F_NEGOP   = $BFB4
.const F_ABS     = $BC58
.const F_SGN     = $BC39
.const F_INT     = $BCCC
.const F_SQR     = $BF71
.const F_LOG     = $B9EA
.const F_EXP     = $BFED
.const F_SIN     = $E26B
.const F_COS     = $E264
.const F_TAN     = $E2B4
.const F_ATN     = $E30E
.const F_RND     = $E097
.const F_FACEXP  = $61     // Exponent des FAC (0 = Zahl ist 0)
.const F_FOUT    = $BDDD   // FAC -> Text bei $0100, A/Y = Zeiger
.const F_FIN     = $BCF3   // Text ab TXTPTR -> FAC (A = erstes Zeichen)
.const F_CHRGOT  = $0079
.const F_TXTPTR  = $007A
