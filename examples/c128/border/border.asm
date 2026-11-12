; =====================================================================================================================
; border.asm - Rahmenfarbe im Takt der Rasterzeile (C128, 40-Spalten-Bildschirm des VIC-IIe)
; =====================================================================================================================
;
; WAS DAS BEISPIEL ZEIGT
;   Das kleinste "Rasterprogramm": Der Prozessor wartet in einer Schleife, bis der Elektronenstrahl des VIC-IIe die Rasterzeile $80
;   (= 128, etwa die Bildmitte) erreicht, und schaltet dann die Rahmenfarbe weiter.  Der Rahmen hat dadurch ab Zeile $80 eine andere
;   Farbe als davor, und weil die Farbe bei jedem Bild (50 Hz, PAL) weiterwandert, flackert er bunt.  Man lernt daraus: die Rasterzeile
;   abfragen ($D012), ein VIC-Register direkt beschreiben, Interrupts sperren und ein Maschinenprogramm mit einer BASIC-Zeile starten.
;
; BAUEN UND STARTEN
;   vcc build .  (im Ordner border)     oder     vcc asm border.asm --target c128 --cpu 8502 -o border.prg
;   Im 40-Spalten-Modus starten (Taste 40/80 DISPLAY bzw. x128 -40col), sonst zeigt der Monitor nur den VDC-Schirm.  Start mit RUN.
;   Das Programm endet nie von selbst; verlassen nur per Reset (der IRQ ist gesperrt, das KERNAL fragt die Tastatur nicht mehr ab).
;
; HARDWARE (Namen und Adressen wie am C64: data/sys/c64/hardware.asm, VIC-IIe)
;   $D012  Rasterzeile (Lesen: aktuelle Zeile, nur die niedrigen 8 Bit; Bit 8 steht in $D011 Bit 7 und bleibt hier unbeachtet.
;          Die Zeile $80 kommt je Bild nur einmal vor, weil ein PAL-Bild nur 312 Zeilen hat und $180 = 384 nie erreicht wird.)
;   $D020  Rahmenfarbe (nur die unteren 4 Bit zaehlen = 16 Farben; INC laeuft deshalb zyklisch durch alle Farben)
;   $D021  Hintergrundfarbe 0 (hier nur auf Schwarz gesetzt, damit der Bildschirm ruhig wirkt)
;
; SPEICHER
;   $1C01        BASIC-Programmstart des C128 (BASIC 7).  Die vom Assembler erzeugte Zeile "10 SYS <Adresse von start>" (hier SYS 7181)
;                belegt $1C01-$1C0C und startet das Programm mit RUN.
;   $1C0D-$1C1F  der Maschinencode (19 Byte); die Datei hat mit Ladeadresse zusammen 33 Byte.
;   Es werden weder Zeropage noch weiteres RAM benutzt und die MMU wird nicht umgeschaltet: die Konfiguration bleibt wie nach dem
;   Start ($00: Systemroms und E/A sichtbar), die VIC-Register sind darum direkt erreichbar.
;
; ZEITVERHALTEN (warum der Effekt so aussieht)
;   Die Schleife braucht nur etwa 17 Takte (LDA 2 + CMP 4 + BNE 2 + INC 6 + JMP 3), eine Rasterzeile dauert dagegen etwa 63 Takte
;   (1 MHz).  Nach dem INC steht der Strahl meist noch in Zeile $80, die Schleife findet die Zeile also gleich wieder und schaltet die
;   Farbe noch ein paar Mal weiter (grob 3 bis 4 Mal je Bild).  In Zeile $80 selbst entstehen so einige feine Farbstreifen, danach gilt
;   die letzte Farbe bis zum Bildende.  Genau ein Wechsel je Bild liesse sich nur erreichen, wenn man nach dem INC erst auf eine
;   andere Zeile wartet.
; =====================================================================================================================
.target c128
.cpu 8502
.const RASTER = $d012                   ; VIC-IIe: aktuelle Rasterzeile (niedrige 8 Bit), nur lesen
.const BORDER = $d020                   ; VIC-IIe: Rahmenfarbe
.const SCREEN = $d021                   ; VIC-IIe: Hintergrundfarbe 0 (Namensgleichheit mit dem KERNAL-Aufruf SCREEN, $FFED, ist Zufall)

.org $1c01                              ; BASIC-Start des C128 (im C64-Modus waere es $0801)
.basicstub start                        ; erzeugt "10 SYS <start>", damit RUN das Maschinenprogramm startet

start:  sei                             ; IRQs sperren: sonst unterbraeche die KERNAL-Interruptroutine (Tastaturabfrage, Cursor) die
                                        ; Schleife und verfaelschte die Zeitverhaeltnisse
        lda #$00
        sta SCREEN                      ; Hintergrund schwarz
.frame: lda #$80                        ; Zielzeile (A bleibt beim CMP erhalten; das erneute Laden je Durchlauf ist nicht noetig)
.wait:  cmp RASTER                      ; Vergleich mit der laufenden Zeile (Z=1, wenn der Strahl sie erreicht hat)
        bne .wait                       ; warten, bis die Zeile $80 beginnt
        inc BORDER                      ; Rahmenfarbe um 1 weiterschalten (nach 15 folgt wieder 0)
        jmp .frame                      ; endlos weiter
