; =====================================================================================================================
; border.asm - Rahmenfarben-Wechsel im Takt der Rasterzeile (Commodore Plus/4 und C16, TED)
; =====================================================================================================================
;
; WAS ZEIGT DAS BEISPIEL
;   Das kleinste TED-Programm mit Rasterbezug: Es wartet in einer Schleife auf eine bestimmte Rasterzeile des Bildschirms und schaltet
;   dort die Rahmenfarbe weiter.  Weil das mitten im Bild geschieht, ist der Rahmen oberhalb und unterhalb dieser Zeile verschieden
;   gefaerbt; da sich die Farbe laufend aendert, ist das Bild unruhig.  Das Programm laeuft endlos.
;   Man lernt daran: den BASIC-Starter (.basicstub), das Warten auf eine Rasterzeile durch Abfragen von $FF1D (Polling) und die
;   Farbregister des TED.
;
; BAUEN UND STARTEN
;   vcc asm border.asm --target c16 --cpu 7501 -o border.prg      (oder vcc build im Projektordner, siehe border.vcprj)
;   Start im Emulator z. B. mit xplus4 -model c16 -autostart border.prg; auf einem echten Rechner das Programm laden und RUN eingeben.
;   Zum Beenden hilft nur ein Reset: die Interrupts sind aus, die Taste RUN/STOP wird deshalb nicht mehr abgefragt.
;
; WAS DER CODE AUF DER HARDWARE TUT
;   - sei: Alle Interrupts sperren.  Die Interruptroutine des KERNAL (Tastaturabfrage, Cursorblinken, Uhr) wuerde sonst mitten in der
;     Warteschleife laufen und die Zeitpunkte verschieben; hier braucht das Programm sie nicht.
;   - Hintergrundfarbe 0 ($FF15) auf Schwarz.  (Der Name SCREEN der Konstante hat nichts mit dem Bildschirmspeicher zu tun, er bezeichnet das
;     Register der Hintergrundfarbe.)
;   - Dann endlos: warten, bis die laufende Rasterzeile $80 (= 128) ist; das Rahmenfarbregister $FF19 um 1 erhoehen.
;   Die Schleife (wenige Befehle) ist vermutlich kuerzer als eine Rasterzeile.  Dann ist die Bedingung "Zeile = $80" mehrmals hintereinander
;   erfuellt, solange der Strahl noch in Zeile $80 steht: die Farbe wird in dieser einen Zeile wohl mehrfach erhoeht (es entsteht ein
;   farbiger Streifen) und nicht genau einmal je Bild.  Das Beispiel will den Effekt zeigen, keinen sauberen Einzelschritt je Bild; wer
;   genau einen Schritt je Bild braucht, muss nach dem Erhoehen warten, bis die Rasterzeile wechselt.
;
; BENUTZTE TED-REGISTER
;   $FF1D  Rasterzeile, Bit 0-7 (nur lesen; Bit 8 steht in $FF1C und wird hier nicht beachtet: $80 liegt unter 256)
;   $FF19  Rahmenfarbe: Bit 0-3 Farbton, Bit 4-6 Helligkeit (TED-Farbwert = Farbton + 16 * Helligkeit, siehe plus4/hardware.asm).
;          Das Erhoehen von $FF19 um 1 geht also durch die 16 Farbtoene und danach durch die Helligkeitsstufen.
;   $FF15  Hintergrundfarbe 0 (wird hier nur auf Schwarz gesetzt)
;
; SPEICHERPLAN
;   $1001      BASIC-Starter (eine Zeile "10 SYS 4109", 12 Byte), danach bei $100D = 4109 das Programm (19 Byte).  Keine Daten, keine Nullseite.
;
; BEKANNTE GRENZEN
;   Das Beispiel ist vom Plus/4 uebernommen; der C16 hat denselben TED und nur 16 KB RAM (das Programm belegt 31 Bytes). Geprueft in VICE 3.10 (xplus4 -model c16): die Rasterbalken laufen.  Die Zeilennummer
;   $80 ist fuer PAL gedacht; NTSC wurde nicht untersucht.
; =====================================================================================================================
.target c16
.cpu 7501
.const RASTER = $ff1d       ; aktuelle Rasterzeile, Bit 0-7 (nur lesen; TED_RASTER_LO in plus4/hardware.asm)
.const BORDER = $ff19       ; Rahmenfarbe: Farbton + 16 * Helligkeit (TED_BORDERCOLOR)
.const SCREEN = $ff15       ; Hintergrundfarbe 0 (TED_BGCOLOR) - nicht der Bildschirmspeicher

.org $1001                  ; Start des BASIC-Speichers beim C16 und Plus/4 (Ladeadresse der .prg-Datei)
.basicstub start            ; erzeugt "10 SYS <start>": RUN springt dadurch nach start

start:  sei                 ; keine Interrupts: die KERNAL-Interruptroutine wuerde die Warteschleife stoeren
        lda #$00
        sta SCREEN          ; Hintergrund schwarz, damit nur der Rahmen Farbe zeigt
.frame: lda #$80            ; auf Rasterzeile $80 warten (A bliebe auch ohne dieses Neuladen erhalten, cmp und inc aendern A nicht)
.wait:  cmp RASTER          ; steht der Strahl schon in Zeile $80?
        bne .wait           ; nein: weiter abfragen (aktives Warten, "Polling")
        inc BORDER          ; ja: Farbe weiterschalten (Lesen-Aendern-Schreiben direkt im Register)
        jmp .frame          ; und von vorn
