; tests_cycles.asm – Zyklenmessung auf der Ultimate 64 (vcc test --backend u64:HOST): Debug-Strom, Modus „6510 Only“
;
; Zweck: Prüfling für die Messung `"mode": "measure"` von `vcc test`. Gemessen werden die CPU-Takte von der ersten Abholung des Befehls an
;   der Marke `messen` bis zur ersten an der Marke `ende`, direkt auf der Hardware (Ultimate 64, Debug-Strom, Modus „6510 Only“; am Gerät
;   unter Data Streams -> Debug Stream Mode einzustellen). Die Testdatei tests_cycles.vctest legt das Budget fest.
;
; Bedienung: `vcc build examples/c64/tests_cycles`, dann `vcc test examples/c64/tests_cycles --backend u64:HOST --yes` (HOST = Adresse
;   der Ultimate). Ohne Gerät meldet `vcc test` nur die statische Größenprüfung (Programm höchstens $0100 Bytes). Das Programm startet
;   auch im Emulator und endet in einer Endlosschleife (nur RESET).
;
; Ablauf: Der Bildschirm wird ausgeschaltet (DEN = 0, keine Bad Lines) und der IRQ gesperrt, damit die Messung genau ist. Dann wartet das
;   Programm auf das Bildende (Rasterzeile $FA) – DEN wirkt erst im nächsten Bild, und das laufende hat bis dahin noch Bad Lines. Danach
;   läuft die gemessene Warteschleife (`dex`/`bne`, 256 Durchläufe) und die Endlosschleife an der Marke `ende`.
; Messstrecke messen..ende:
;   LDX (2) + 255 * (DEX 2 + BNE genommen 3) + (DEX 2 + BNE nicht genommen 2) + JMP (3) = 1284 Zyklen.
;   Die Testdatei prüft Budget 1284 (muss bestehen) und Budget 1283 mit Stufe „hint“ (muss knapp scheitern und zeigt so, dass die
;   Messung auf den Takt genau ist). 1284 Takte sind etwa 20 PAL-Rasterzeilen (je 63 Takte); sie bleiben im selben Bild (ab Zeile $FA).
; Weder `messen` noch `ende` steht unmittelbar hinter einem BNE: Eine genommene Verzweigung liest als Blindlesen schon die Adresse dahinter
;   (PC+2); das wäre ein „Zugriff“ auf die Marke und würde die Messung zu früh starten oder beenden.
;
; Hardware: VIC-II $D011 (Steuerregister 1) und $D012 (Rasterzeile). Speicherplan: $0801 BASIC-Zeile `10 SYS 2061`, Programm ab $080D
;   (39 Bytes mit Ladeadresse 41). Zeropage: unbenutzt.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei                     ; kein Interrupt darf in die Messung hineinlaufen
        lda #$0b                ; DEN = 0: Bildschirm aus, keine Bad Lines ab dem nächsten Bild (Wert $0B: RSEL = 1, YSCROLL = 3, Bit 4 = 0)
        sta $d011
bild:   lda $d012               ; auf das Bildende warten (Zeile $FA): das laufende Bild hat noch Bad Lines
        cmp #$fa
        bne bild
        jmp messen              ; nicht unmittelbar hinter dem BNE (Blindlesen, siehe oben)
messen: ldx #0                  ; Messbeginn: erste Abholung dieses Befehls
warte:  dex                     ; X läuft von 0 über 255 bis 0 zurück: 256 Durchläufe
        bne warte
        jmp ende                ; 3 Zyklen, Ziel ist der nächste Befehl
ende:   jmp ende                ; Endlosschleife: der erste Zugriff auf `ende` beendet die Messung
