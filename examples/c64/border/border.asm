; border.asm – Rahmenfarbe im Takt der Rasterzeile weiterschalten (C64) – erstes VintageCoder-Beispiel
;
; Zweck: das kleinste sinnvolle C64-Assemblerprogramm. Es zeigt den Aufbau einer Quelldatei (.target, .cpu, .const, .org, .basicstub),
;   lokale Marken (.frame, .wait), das Warten auf eine bestimmte Rasterzeile des VIC-II und – mit Absicht – einen undokumentierten
;   Befehl (LAX), für den der Assembler die Warnung W104 ausgibt.
;
; Bedienung: bauen mit `vcc build examples/c64/border` (oder `vcc asm border.asm --target c64 --cpu 6510 -o border.prg`), das PRG laden
;   und mit RUN starten. Beenden kann man es nur mit RESET: das Programm kehrt nie zurück, und weil der Interrupt gesperrt bleibt
;   (SEI ohne CLI), reagiert auch die Taste RUN/STOP nicht.
;
; Ablauf: Hintergrundfarbe auf Schwarz, dann endlos: warten, bis der VIC-II die Rasterzeile $80 (128) zeichnet, und dann die Rahmenfarbe
;   um eins erhöhen. Ein Durchlauf der Schleife dauert 21 Takte (LDA# 2 + CMP abs 4 + BNE nicht genommen 2 + INC abs 6 + LAX abs 4 + JMP 3),
;   eine PAL-Rasterzeile dauert 63 Takte. Deshalb wird die Farbe innerhalb der einen Zeile $80 mehrmals (etwa dreimal) weitergeschaltet;
;   danach passt der Vergleich nicht mehr, und das Programm wartet bis zur Zeile $80 des nächsten Bildes. Vermutlich flackert der Rahmen
;   dadurch schnell durch die Farben, mit einer Farbgrenze an der Rasterzeile $80 (nicht auf Hardware nachgeprüft).
;
; Hardware (VIC-II, siehe c64.inc bzw. data/chips/vic2.chip.json):
;   $D012  Rasterzeile, Bits 0-7 (Bit 8 steht in Bit 7 von $D011; hier genügt das untere Byte, weil $80 < 256)
;   $D020  Rahmenfarbe (Farbnummer 0-15; nur die unteren vier Bits wirken, INC läuft also nach 15 wieder auf 0)
;   $D021  Hintergrundfarbe 0
;
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061`, Programm ab $080D (= 2061), zusammen 34 Bytes ohne Ladeadresse. Zeropage: unbenutzt.
; Besonderheit: LAX ist ein undokumentierter 6510-Befehl (lädt A und X zugleich). Er steht nur zur Vorführung der Warnung W104 hier und
;   hat keine Wirkung auf den Ablauf, weil A in .frame ohnehin neu geladen wird. `illegals = warn` übersetzt ihn mit Warnung;
;   `deny` würde ihn als Fehler abweisen, `allow` ihn ohne Meldung übersetzen.
.target c64
.cpu 6510 { illegals = warn }
.const RASTER = $d012           ; VIC-II: aktuelle Rasterzeile (lesen: Bit 0-7)
.const BORDER = $d020           ; VIC-II: Rahmenfarbe
.const SCREEN = $d021           ; Achtung, irreführender Name: das ist die Hintergrundfarbe 0, nicht der Bildschirmspeicher ($0400)

.org $0801                      ; BASIC-Startadresse des C64
.basicstub start                ; erzeugt `10 SYS <Adresse von start>`, damit RUN das Maschinenprogramm startet

start:  sei                     ; Interrupts sperren: der KERNAL-IRQ würde nur Zeit kosten und die Schleife stören
        lda #$00
        sta SCREEN              ; Hintergrund schwarz
.frame: lda #$80                ; auf Rasterzeile $80 warten
.wait:  cmp RASTER              ; Vergleich mit dem unteren Byte der aktuellen Rasterzeile
        bne .wait               ; solange ungleich: weiter warten
        inc BORDER              ; Farbe weiterschalten
        lax $d020               ; absichtlich undokumentiert: A und X laden (W104)
        jmp .frame
