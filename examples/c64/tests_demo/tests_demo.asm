; tests_demo.asm – Beispiel für `vcc test`: statische Prüfungen und Tests im Emulator (Größe, Zeropage, Überlappung, Zyklenspanne)
;
; Zweck: zeigt, wofür die Testdatei tests_demo.vctest gebraucht wird. Das Programm selbst ist nur ein Prüfling; es enthält drei kleine
;   Unterprogramme, an denen sich verschiedene Prüfungen vorführen lassen:
;     irq_handler  geradliniger Code (PHA, LDA, STA, PLA, RTI): die Zyklen sind genau bestimmbar, im Quelltext zusätzlich mit einem
;                  `.cycles`-Block mit Budget 24 Takte versehen (tatsächlich 3 + 3 + 4 + 4 + 6 = 20 Takte)
;     next_color   Verzweigung nach vorn: die Zyklenspanne reicht vom kürzesten bis zum längsten Weg (hier 18 bis 19 Takte)
;     clear_text   Schleife: die Prüfung meldet „nicht bestimmbar“ statt zu raten; gemessen im Emulator etwa 3079 Takte
;   Das Hauptprogramm `start` ruft clear_text auf und springt dann wie ein Interrupt in irq_handler (siehe „Grenzen“); next_color
;   wird nur von den Tests benutzt. Danach endet das Programm in einer Endlosschleife.
;
; Bedienung (im Repository-Wurzelverzeichnis):
;   vcc build examples/c64/tests_demo              ; baut tests_demo.prg und tests_mailbox.prg
;   vcc test examples/c64/tests_demo               ; nur die statischen Prüfungen (ohne Emulator)
;   vcc test examples/c64/tests_demo --backend vice   ; zusätzlich die Tests im Emulator (VICE, nur ein Emulator zugleich)
; Die Prüfungen stehen in tests_demo.vctest: size (Segment und Programm gesamt, mit Endadresse), zeropage (verboten: $02-$1F), overlap,
; cycles (Zyklenblock `irq`, Marken irq_handler, next_color, clear_text, start; das Budget 18 für next_color ist knapp bemessen und
; zeigt den Hinweis „Budget überschritten“ – der längere Weg braucht 19 Takte; für clear_text und start meldet die Prüfung „nicht
; bestimmbar“ wegen Schleife bzw. JSR), expr (Programmgröße unter $2000) sowie vier Tests im Emulator (next_color mit 15 und 3,
; irq_handler mit Rückkehr per RTI, clear_text). Der zweite Prüfling im Projekt, tests_mailbox.asm, hat seine eigene Testdatei.
;
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061`, Programm ab $080D (64 Bytes mit Ladeadresse). Zeropage: $FB = COLOR (aktuelle Farbe).
;   Die Testdatei verbietet $02-$1F; $FB ist erlaubt. Bildschirmspeicher $0400-$04FF wird von clear_text beschrieben, Rahmenfarbe $D020
;   von irq_handler.
; Grenzen: irq_handler endet mit RTI und darf daher nicht per JSR aufgerufen werden; `start` legt deshalb Rücksprungadresse und Status
;   selbst auf den Stapel und springt mit JMP hin. Der Emulator-Test ruft irq_handler über die Testfalle mit `"return": "rti"` auf.
;   next_color wird vom Programm selbst nie aufgerufen, nur von den Tests.
.target c64
.cpu 6510 { illegals = deny }
.const BORDER = $d020           ; VIC-II: Rahmenfarbe
.const COLOR  = $fb             ; Zeropage: aktuelle Farbe (frei laut Testdatei)

.org $0801
.basicstub start

start:  sei
        jsr clear_text          ; Schleife: Zyklen nicht bestimmbar
        lda #>back              ; Interrupt von Hand nachbilden, damit das RTI von irq_handler richtig zurückkehrt:
        pha                     ; Rücksprungadresse (high, dann low) und Statusregister auf den Stapel legen (wie ein echter IRQ),
        lda #<back              ; RTI holt sie in der Reihenfolge P, low, high zurück und landet genau bei `back`
        pha
        php
        jmp irq_handler
back:   jmp *

; irq_handler – Rasterunterbrechung, geradliniger Code: genau bestimmbar.
;   Eingabe: COLOR ($FB) = Farbe. Ausgabe: Rahmenfarbe $D020 = COLOR. A bleibt erhalten (PHA/PLA), kein X/Y.
;   Endet mit RTI, ist also als Interruptroutine gedacht (Rücksprung mit Status). Ohne Budgetüberschreitung: 20 Takte, Budget 24.
irq_handler:
.cycles irq max=24              ; der Assembler addiert die Takte bis .endcycles und warnt (W201) bei mehr als 24 im ungünstigsten Fall
        pha
        lda COLOR
        sta BORDER
        pla
        rti
.endcycles

; next_color – Verzweigung nach vorn: die Spanne reicht vom kürzesten bis zum längsten Weg.
;   Eingabe: COLOR ($FB) = aktuelle Farbe. Ausgabe: COLOR = nächste Farbe 0..15 (von 15 zurück auf 0). Verändert A und die Flags.
;   Weg 1 (Farbe = 15): 18 Takte; Weg 2 (sonst): 19 Takte.
next_color:
        lda COLOR
        cmp #15
        bne .inc                ; ungleich 15: hochzählen
        lda #0                  ; sonst von vorn
        sta COLOR
        rts
.inc:   inc COLOR
        rts

; clear_text – Schleife: die Prüfung meldet „nicht bestimmbar“ statt zu raten.
;   Löscht die ersten 256 Zeichen des Bildschirms ($0400-$04FF) mit dem Leerzeichen ($20). Verändert A und X (X = 0 am Ende, Z gesetzt).
;   Ein Durchlauf: LDX 2 + 256 * (LDA# 2 + STA abs,X 5 + INX 2) + 255 * BNE genommen 3 + BNE nicht genommen 2 + RTS 6 = 3079 Takte.
clear_text:
        ldx #0
.loop:  lda #$20                ; Bildschirmcode des Leerzeichens
        sta $0400,x
        inx
        bne .loop               ; X läuft von 0 bis 255 und dann auf 0: 256 Durchläufe
        rts
