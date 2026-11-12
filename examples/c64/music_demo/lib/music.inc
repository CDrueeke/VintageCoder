; music.inc – C64: Spielroutine für Tracker-Songs (SID, 3 Stimmen) aus dem Export.
; ERZEUGT aus music.inc.in durch tools/gen_music.py – Änderungen nur in der Vorlage music.inc.in vornehmen (sie ist die C64-Fassung und
; wird unverändert kopiert; die MEGA65-Fassung entsteht daraus durch Textersetzungen).
;
;   .include "lib/music.inc"
;   music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; Labels/Konstanten aus dem Export
;   ...                                   ; jedes Bild (50/60 Hz, z. B. im Rasterzeilen-IRQ) einmal:
;   jsr music_play
;   music_stop                            ; Gates aus, Lautstärke 0
;   music_runtime                         ; EINMAL, hinter dem Hauptprogramm (nicht ausgeführt)
;
; Frequenztabelle PAL; mit -D NTSC=1 (oder .const NTSC = 1 vor dem Include) die NTSC-Tabelle.
; Belegte Zeropage: $F7-$FA. Zustand: mus_row, mus_ord, mus_tick (Speicher); mus_sid = Abbild der SID-Register ($D400-$D418).
; Instrumente: höchstens 32 (SID-Datensatz 8 Bytes). NOTE_B7 ist bei PAL für den 16-Bit-Frequenzzähler zu hoch (Obergrenze 65535, siehe NOTE_TABLE_HIGHEST).
;
; Datenformat der Songtabellen (Export des Trackers, kodiert in crates/vc-media/src/song.rs):
;   order      Pattern-Nummern, dann $FF und die Wiederholungsposition (Index in order, bei dem es nach dem Ende weitergeht)
;   patterns   je Pattern: Zeilen * Stimmen * 2 Bytes, zeilenweise, je Stimme Note und Instrument.
;              Note: 0 = nichts (Ton läuft weiter), 1..96 = C-0..B-7 (Index in die Frequenztabelle), $FF = Gate aus (Ton klingt aus).
;              Instrument: $FF = unverändert lassen, sonst Nummer des Instruments (gilt ab dann für diese Stimme).
;   ins        je Instrument 8 Bytes: +0 Steuerbyte (Wellenform: $10 Dreieck, $20 Sägezahn, $40 Puls, $80 Rauschen; Gate-Bit 0 ist hier aus),
;              +1 Attack/Decay, +2 Sustain/Release, +3 Pulsbreite low, +4 Pulsbreite high (Bits 0-3), +5..+7 unbenutzt
;   speed      Bilder je Zeile (Tempo); rows = Zeilen je Pattern; patsize = Bytes je Pattern (rows * Stimmen * 2)
; Ablauf: music_play wird einmal je Bild aufgerufen und zählt mus_tick bis speed; bei mus_tick = 0 verarbeitet mus_step die nächste Zeile
;   des Patterns (für jede Stimme: Instrument merken, bei Note Frequenz, Pulsbreite, Hüllkurve und Steuerbyte in die SID-Register
;   schreiben, dabei das Gate kurz aus- und wieder einschalten, damit die Hüllkurve neu startet). Nach der letzten Zeile folgt das
;   nächste Pattern der Reihenfolge; steht dort $FF, springt die Routine auf die Wiederholungsposition.
; Selbstverändernder Code: music_init trägt die Tabellenadressen (order, patterns, ins, patsize) in die Operanden einzelner Befehle der
;   Spielroutine ein (Marken mus_o1.., mus_pb_.., mus_ps_.., mus_ib_..: `.const x = *` gibt dem nächsten Befehl einen Namen, x+1 ist
;   dessen Operand). Die Spielroutine muss deshalb im RAM stehen (nicht im ROM/Modul), und music_init muss vor dem ersten music_play laufen.
; Zeropage: $F7-$FA dürfen zwischen den music_play-Aufrufen nicht anderweitig benutzt werden: mus_ptr ($F7/$F8) behält den Stand der
;   aktuellen Patternzeile von einem Aufruf zum nächsten. (Die Kachel-Bibliothek benutzt $F7-$FE – nicht zusammen verwenden.)
; Interruptsicherheit: music_play ist nicht wiedereintrittsfähig; sie darf nur von einer Stelle aufgerufen werden (z. B. nur im IRQ).
;   Register A, X, Y und Flags werden verändert; der IRQ-Rahmen muss sie retten.

.const mus_ptr = $f7            ; 16 Bit: Zeiger auf die aktuelle Patternzeile (bleibt zwischen den Aufrufen erhalten)
.const mus_ip  = $f9            ; 16 Bit: Zeiger auf den Instrumentendatensatz (nur während mus_step gültig)

; music_init(order, patterns, ins, speed, rows, patsize) – Spielroutine mit den Songtabellen verbinden und starten.
;   Eingabe: Labels der Tabellen (order, patterns, ins) und Konstanten speed, rows, patsize, wie sie der Export liefert.
;   Wirkung: patcht die Tabellenadressen in die Spielroutine (siehe „Selbstverändernder Code“), setzt Tick, Zeile, Reihenfolgeposition und
;   die Instrumente der Stimmen auf 0, löscht alle SID-Register samt Abbild mus_sid, setzt die Lautstärke auf 15 und schaltet die Musik
;   ein (mus_on = 1). Verändert A, X und die Flags. Der erste music_play-Aufruf spielt dann sofort die erste Zeile.
.macro music_init(order, patterns, ins, speed, rows, patsize)
        lda #<order
        sta mus_o1+1
        lda #>order
        sta mus_o1+2
        lda #<(order+1)
        sta mus_o2+1
        lda #>(order+1)
        sta mus_o2+2
        lda #<order
        sta mus_o3+1
        lda #>order
        sta mus_o3+2
        lda #<patterns
        sta mus_pb_l+1
        lda #>patterns
        sta mus_pb_h+1
        lda #<patsize
        sta mus_ps_l+1
        lda #>patsize
        sta mus_ps_h+1
        lda #<ins
        sta mus_ib_l+1
        lda #>ins
        sta mus_ib_h+1
        lda #speed
        sta mus_speed
        lda #rows
        sta mus_rows
        lda #0
        sta mus_tick
        sta mus_row
        sta mus_ord
        sta mus_ins
        sta mus_ins+1
        sta mus_ins+2
        ; alle SID-Register samt Abbild löschen (Schleife über alle Registeradressen, abwärts bis X < 0)
        ldx #24
@clr:   sta mus_sid,x
        sta $d400,x
        dex
        bpl @clr
        lda #15
        sta mus_sid+24
        sta $d418
        lda #1
        sta mus_on
.endmacro

; music_stop – Musik anhalten: mus_on = 0, Steuerregister aller Stimmen auf 0 (Gate aus, keine Wellenform), Lautstärke 0.
;   Verändert A und die Flags. Der Zustand der Spielroutine bleibt (mus_row usw.); ein Neustart geht nur mit music_init.
.macro music_stop
        lda #0
        sta mus_on
        sta $d404
        sta $d40b
        sta $d412
        sta $d418
.endmacro

; music_runtime – die eigentliche Spielroutine, Zustandsvariablen und Frequenztabelle (einmal einbinden, nicht ausführen).
.macro music_runtime
; music_play – einen Takt (ein Bild) weiterspielen.
;   Eingabe: keine. Ausgabe: SID-Register $D400.. und Abbild werden bei einem neuen Zeilenanfang aktualisiert.
;   Ruft bei mus_tick = 0 mus_step auf; zählt mus_tick bis mus_speed hoch und setzt ihn dann auf 0. Ohne music_init (mus_on = 0)
;   passiert nichts. Verändert A, X, Y und die Flags.
music_play:                         ; einmal pro Bild aufrufen
        lda mus_on
        beq @rts
        lda mus_tick
        bne @t
        jsr mus_step
@t:     inc mus_tick
        lda mus_tick
        cmp mus_speed
        bcc @rts
        lda #0
        sta mus_tick
@rts:   rts

; mus_step – eine Patternzeile verarbeiten (von music_play aufgerufen, wenn mus_tick = 0).
;   Am Zeilenanfang 0 wird zuerst das Pattern aus der Reihenfolge bestimmt und mus_ptr = patterns + Patternnummer * patsize berechnet
;   (durch wiederholtes Addieren). Dann werden die Stimmen der Zeile der Reihe nach bearbeitet (mus_v = Stimme, mus_y = Byteposition in der
;   Zeile, mus_r = Registerversatz der Stimme im SID). Am Ende rückt mus_ptr eine Zeile weiter; nach der letzten Zeile beginnt mit
;   mus_row = 0 und erhöhtem mus_ord das nächste Pattern. Verändert A, X, Y und die Flags.
mus_step:
        lda mus_row
        bne @row
        ldx mus_ord                 ; neues Pattern: Reihenfolge lesen ($ff = Ende, danach die Wiederholungsposition)
.const mus_o1 = *
        lda $ffff,x
        cmp #$ff
        bne @pat
.const mus_o2 = *
        lda $ffff,x
        sta mus_ord
        tax
.const mus_o3 = *
        lda $ffff,x
@pat:   sta mus_t
.const mus_pb_l = *
        lda #0
        sta mus_ptr
.const mus_pb_h = *
        lda #0
        sta mus_ptr+1
        ldx mus_t
        beq @row
@add:   clc
        lda mus_ptr
.const mus_ps_l = *
        adc #0
        sta mus_ptr
        lda mus_ptr+1
.const mus_ps_h = *
        adc #0
        sta mus_ptr+1
        dex
        bne @add
@row:   ldx #0                      ; Stimme 0..2
        stx mus_v
        ldy #0
@voice: lda (mus_ptr),y
        sta mus_n
        iny
        lda (mus_ptr),y
        iny
        sty mus_y
        cmp #$ff
        beq @keep
        sta mus_ins,x
@keep:  lda mus_n
        beq @next
        cmp #$ff
        beq @off
        lda mus_voff,x              ; Registerversatz der Stimme
        sta mus_r
        lda mus_ins,x               ; Instrumentendatensatz = Basis + Nummer * 8
        asl a
        asl a
        asl a
        clc
.const mus_ib_l = *
        adc #0
        sta mus_ip
.const mus_ib_h = *
        lda #0
        adc #0
        sta mus_ip+1
        ldx mus_r
        ldy mus_n
        lda note_sid_lo,y
        sta mus_sid,x
        sta $d400,x
        lda note_sid_hi,y
        sta mus_sid+1,x
        sta $d401,x
        ldy #3
        lda (mus_ip),y
        sta mus_sid+2,x
        sta $d402,x
        iny
        lda (mus_ip),y
        sta mus_sid+3,x
        sta $d403,x
        ldy #1
        lda (mus_ip),y
        sta mus_sid+5,x
        sta $d405,x
        iny
        lda (mus_ip),y
        sta mus_sid+6,x
        sta $d406,x
        ldy #0
        lda (mus_ip),y
        sta mus_sid+4,x             ; Gate aus (Neustart der Hüllkurve) …
        sta $d404,x
        ora #1                      ; … und wieder an
        sta mus_sid+4,x
        sta $d404,x
        jmp @next
@off:   lda mus_voff,x
        tax
        lda mus_sid+4,x
        and #$fe
        sta mus_sid+4,x
        sta $d404,x
@next:  inc mus_v
        ldx mus_v
        ldy mus_y
        cpx #3
        beq @rowend
        jmp @voice
@rowend:
        clc                         ; Zeiger auf die nächste Zeile (3 Stimmen * 2 Bytes)
        lda mus_ptr
        adc #6
        sta mus_ptr
        bcc @c
        inc mus_ptr+1
@c:     inc mus_row
        lda mus_row
        cmp mus_rows
        bcc @done
        lda #0
        sta mus_row
        inc mus_ord
@done:  rts

; --- Zustand der Spielroutine (im RAM, daher veränderbar) ----------------------------------------------------------------
; mus_on: 1 = Musik läuft, 0 = angehalten. mus_speed/mus_rows: Bilder je Zeile und Zeilen je Pattern (music_init überschreibt die
; Vorgaben 6 und 32). mus_tick: Bildzähler 0..speed-1. mus_row: aktuelle Zeile im Pattern. mus_ord: Position in der Reihenfolge.
; mus_t, mus_n, mus_v, mus_y, mus_r: Arbeitsvariablen von mus_step (Patternnummer, Note, Stimme, Byteposition, Registerversatz).
; mus_ins: aktuelles Instrument je Stimme. mus_voff: Versatz des ersten SID-Registers je Stimme (je Stimme +7).
; mus_sid: Abbild der SID-Register, weil die Register nur beschreibbar sind (für Gate-Bit und Lautstärke).
mus_on:    .byte 0
mus_speed: .byte 6
mus_rows:  .byte 32
mus_tick:  .byte 0
mus_row:   .byte 0
mus_ord:   .byte 0
mus_t:     .byte 0
mus_n:     .byte 0
mus_v:     .byte 0
mus_y:     .byte 0
mus_r:     .byte 0
mus_ins:   .byte 0,0,0
mus_voff:  .byte 0,7,14
mus_sid:   .fill 25, 0
; Frequenztabelle (note_sid_lo/hi, mit dem Notenwert indiziert) und Notenkonstanten: <c64/notes.inc>, PAL oder mit NTSC die NTSC-Tabelle
#include <c64/notes.inc>
.endmacro
