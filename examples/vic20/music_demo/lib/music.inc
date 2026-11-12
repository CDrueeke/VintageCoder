; ---------------------------------------------------------------------------------------------------------------------
; music.inc – VC-20: Spielroutine für Tracker-Songs (VIC-I: Bass, Alt, Sopran und Rauschen = 4 Stimmen) aus dem Export (Profil vic20.vic.song).
; ---------------------------------------------------------------------------------------------------------------------
;
; BENUTZUNG
;   .include "lib/music.inc"
;   music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; Labels/Konstanten aus dem Export
;   ...                                   ; einmal pro Bild (50/60 Hz, z. B. im IRQ):
;   jsr music_play
;   music_stop                            ; alle Stimmen aus (Makro)
;   music_runtime                         ; EINMAL, hinter dem Hauptprogramm (nicht ausgeführt): Spielroutine und Variablen
;   .include "lib/vic_freq.inc"           ; Frequenztabellen (mit .const NTSC = 1 vor dem Include die NTSC-Tabelle)
;
; DATENFORMAT (Export des Trackers, hier build/export/tune.asm)
;   order: eine Musternummer je Byte; $FF = Ende der Reihenfolge, das Byte dahinter ist die Position, bei der sie wieder beginnt (Wiederholung).
;   patterns: Muster zu je patsize Byte = rows Zeilen zu je 8 Byte; eine Zeile enthält die 4 Stimmen zu je 2 Byte (Note, Instrument). Stimme 0 =
;   Bass, 1 = Alt, 2 = Sopran, 3 = Rauschen. Note: 0 = nichts (der Ton läuft weiter), $FF = Ton aus, 1..96 = Note (C-0 = 1; größere Werte sind nicht
;   definiert, die Frequenztabellen enden bei 96). Instrument: $FF = beibehalten, sonst Nummer 0..7.
;   ins: Instrumentendatensätze zu je 32 Byte, Byte 0 = Verzerrung (wird ignoriert), Byte 1 = Wiederholungsstelle der Hüllkurve (0..15), Bytes 2..17
;   = 16 Lautstärkewerte der Hüllkurve, der Rest ist unbenutzt. speed = Bilder je Zeile, rows = Zeilen je Muster, patsize = Bytes je Muster (rows *
;   8).
;
; VIC-I UND GRENZEN
;   Der VIC-I hat nur eine gemeinsame Lautstärke ($900E, Bit 0-3), keine Hüllkurve je Stimme. Die Lautstärkehülle des Instruments (16 Werte, einer
;   je Bild) schaltet die Stimme deshalb nur ein (Wert > 0) oder aus (0); die Lautstärke bleibt bei 15. Die Verzerrung des Instruments wird
;   ignoriert. Stimme 0 = Bass ($900A), 1 = Alt ($900B), 2 = Sopran ($900C), 3 = Rauschen ($900D). Instrumente: höchstens 8 (Datensatz 32 Bytes,
;   Nummer * 32 muss in ein Byte passen). Noten außerhalb des Tonumfangs einer Stimme werden oktaviert (siehe tools/gen_vic_freq.py); beim Rauschen
;   bestimmt die Note nur die Klangfarbe.
;   Die Spielroutine ändert ihren eigenen Code: music_init trägt die Adressen von Reihenfolge, Mustern und Instrumenten in Befehlsoperanden von
;   music_runtime ein (die mit „.const mus_xx = *“ markierten Befehle). Sie läuft darum nur im RAM, nicht im ROM, und music_init muss vor dem ersten
;   music_play gerufen werden.
;
; SPEICHER
;   Belegte Zeropage: $FB-$FE (mus_ptr, mus_ip). Variablen mus_* stehen im Programm hinter dem Code. Alle Routinen sind nicht wiedereintrittsfähig:
;   music_play darf nur aus einer Stelle (IRQ oder Hauptschleife) gerufen werden. music_play verändert A, X, Y und die Flags.
; ---------------------------------------------------------------------------------------------------------------------

.const mus_ptr = $fb            ; 16 Bit: Zeiger auf die aktuelle Patternzeile
.const mus_ip  = $fd            ; 16 Bit: Zeiger auf Instrumentendatensatz bzw. Frequenztabelle der Stimme (jeweils nur kurz benutzt)

; music_init(order, patterns, ins, speed, rows, patsize): Player vorbereiten und starten (Makro, einmal vor dem ersten music_play; neu aufrufen = Song von vorn).
;   order/patterns/ins sind Labels, speed/rows/patsize Konstanten des Exports. Trägt die Adressen in die Operanden von music_runtime ein (siehe oben),
;   löscht den Zustand aller Stimmen (aus, Instrument 0, Hüllkurve von vorn), setzt die Stimmenregister $900A-$900D auf 0 und die Lautstärke auf 15. Verändert A, X.

.macro music_init(order, patterns, ins, speed, rows, patsize)
        lda #<order                 ; Reihenfolge in die drei Befehle eintragen, die sie lesen:
        sta mus_o1+1                ;   mus_o1: Mustername an der Position lesen
        lda #>order
        sta mus_o1+2
        lda #<(order+1)             ;   mus_o2: beim Endezeichen $FF das Byte dahinter (Wiederholungsposition) lesen, daher order+1
        sta mus_o2+1
        lda #>(order+1)
        sta mus_o2+2
        lda #<order                 ;   mus_o3: Mustername an der Wiederholungsposition lesen
        sta mus_o3+1
        lda #>order
        sta mus_o3+2
        lda #<patterns              ; Anfang der Muster, Größe eines Musters (für „Nummer * patsize“) und Instrumentenbasis eintragen
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
        lda #speed                  ; Bilder je Zeile und Zeilen je Muster
        sta mus_speed
        lda #rows
        sta mus_rows
        lda #0                      ; ganz von vorn beginnen: Takt, Zeile und Position in der Reihenfolge 0
        sta mus_tick
        sta mus_row
        sta mus_ord
        ldx #3                      ; Stimme 3..0:
@clr:   sta mus_act,x               ;   nicht klingend, Instrument 0, Hüllkurvenposition 0, kein Registerwert, VIC-I-Stimme aus
        sta mus_ins,x
        sta mus_pos,x
        sta mus_reg,x
        sta $900a,x
        dex
        bpl @clr
        lda $900e                   ; Hilfsfarbe (Bit 4-7) behalten, Lautstärke 15
        and #$f0
        ora #$0f
        sta $900e
        lda #1                      ; Player freigeben (music_play tut nur etwas, wenn mus_on <> 0)
        sta mus_on
.endmacro

; music_stop: Player anhalten und alle vier Stimmen ausschalten (Makro). Wieder starten mit music_init. Verändert A.
.macro music_stop
        lda #0
        sta mus_on
        sta $900a
        sta $900b
        sta $900c
        sta $900d
.endmacro

; music_runtime: Spielroutine und Variablen (Makro, EINMAL hinter dem Hauptprogramm aufrufen).
.macro music_runtime
; music_play: eine Spielstufe, einmal pro Bild aufrufen (tut nichts, solange mus_on = 0). Beim ersten Bild jeder Zeile (mus_tick = 0) wird die nächste
;   Patternzeile gelesen (mus_step); in jedem Bild läuft die Hüllkurve einen Schritt weiter (mus_env). Nach speed Bildern beginnt die nächste Zeile.
;   Verändert A, X, Y, Flags, mus_* und $FB-$FE; schreibt $900A-$900D.
music_play:                         ; einmal pro Bild aufrufen
        lda mus_on
        beq @rts
        lda mus_tick
        bne @t
        jsr mus_step
@t:     jsr mus_env
        inc mus_tick
        lda mus_tick
        cmp mus_speed
        bcc @rts
        lda #0
        sta mus_tick
@rts:   rts

; mus_step: eine Patternzeile lesen und die Noten anstoßen. Ist mus_row = 0, wird zuerst das Muster bestimmt: Name aus der Reihenfolge (mus_ord), mus_ptr =
;   patterns + Name * patsize (durch wiederholtes Addieren). Dann für die Stimmen 0..3: Note und Instrument lesen; Note 0 = nichts tun, $FF = Stimme aus, sonst
;   Registerwert aus der Frequenztabelle (Index Note - 1), Stimme aktiv, Hüllkurve von vorn. Am Ende mus_ptr + 8, mus_row + 1; nach rows Zeilen mus_row = 0 und
;   mus_ord + 1. Verändert A, X, Y, mus_ptr, mus_ip und die Stimmenvariablen. mus_v = laufende Stimme, mus_y = Y-Merker (Zeilenposition), mus_n = Note.
mus_step:                           ; eine Patternzeile lesen und die Noten anstoßen
        lda mus_row
        bne @row
        ldx mus_ord                 ; neues Pattern: Reihenfolge lesen ($ff = Ende, danach die Wiederholungsposition)
.const mus_o1 = *                   ; (Operand dieses Befehls trägt music_init ein: Adresse der Reihenfolge)
        lda $ffff,x
        cmp #$ff
        bne @pat
.const mus_o2 = *                   ; (Operand = order + 1: das Byte hinter dem $FF)
        lda $ffff,x
        sta mus_ord                 ; Wiederholungsposition
        tax
.const mus_o3 = *                   ; (Operand = order)
        lda $ffff,x
@pat:   sta mus_t                   ; Musternummer
.const mus_pb_l = *                 ; (Operand von lda: Low-Byte der Musteradresse; ebenso mus_pb_h, mus_ps_l/h, mus_ib_l/h)
        lda #0
        sta mus_ptr
.const mus_pb_h = *
        lda #0
        sta mus_ptr+1
        ldx mus_t
        beq @row                    ; Muster 0: mus_ptr zeigt schon auf den Anfang
@add:   clc                         ; mus_ptr += patsize, Musternummer-mal
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
@row:   ldx #0                      ; Stimme 0..3
        stx mus_v
        ldy #0
@voice: lda (mus_ptr),y            ; Note der Stimme
        sta mus_n
        iny
        lda (mus_ptr),y             ; Instrument (Y bleibt als Zeilenposition in mus_y)
        iny
        sty mus_y
        cmp #$ff
        beq @keep                   ; $FF: Instrument beibehalten
        sta mus_ins,x
@keep:  lda mus_n
        beq @next                   ; Note 0: nichts tun
        cmp #$ff
        beq @off                    ; Note $FF: Stimme aus
        lda mus_fpl,x               ; Frequenztabelle der Stimme
        sta mus_ip
        lda mus_fph,x
        sta mus_ip+1
        ldy mus_n
        dey
        lda (mus_ip),y
        sta mus_reg,x
        beq @off                    ; 0 = Ton nicht darstellbar
        lda #1                      ; Stimme klingt; Hüllkurve beginnt (mus_env schreibt das Register im selben Bild)
        sta mus_act,x
        lda #0
        sta mus_pos,x
        beq @next                   ; (immer, A = 0)
@off:   lda #0
        sta mus_act,x
        sta $900a,x                 ; VIC-I-Stimme x aus (X = 0..3 ist zugleich der Registeroffset)
@next:  inc mus_v
        ldx mus_v
        ldy mus_y
        cpx #4
        beq @rowend
        jmp @voice                  ; nächste Stimme
@rowend:
        clc                         ; Zeiger auf die nächste Zeile (4 Stimmen * 2 Bytes)
        lda mus_ptr
        adc #8
        sta mus_ptr
        bcc @c
        inc mus_ptr+1
@c:     inc mus_row
        lda mus_row
        cmp mus_rows
        bcc @done
        lda #0                      ; Musterende: nächstes Muster der Reihenfolge
        sta mus_row
        inc mus_ord
@done:  rts

; mus_env: Hüllkurve einen Schritt weiter, für jede aktive Stimme (X = 3..0): Instrumentendatensatz = ins + Nummer * 32 (Operanden von music_init gesetzt), Wert =
;   Datensatz[2 + mus_pos]. Wert > 0 (und Registerwert vorhanden): Stimme ein, Register $900A+x = mus_reg; Wert 0: Register 0. Danach mus_pos + 1; bei 16 springt
;   mus_pos auf die Wiederholungsstelle (Datensatz[1]). Verändert A, X, Y, mus_ip und die Stimmenregister.
mus_env:                            ; Hüllkurve: Stimme an (Lautstärkewert > 0) oder aus, einen Schritt weiter
        ldx #3
@v:     lda mus_act,x
        beq @nx
        lda mus_ins,x               ; Instrumentendatensatz = Basis + Nummer * 32
        asl a
        asl a
        asl a
        asl a
        asl a                       ; * 32 (bei Instrument >= 8 ginge das Übertragsbit verloren: höchstens 8 Instrumente)
        clc
.const mus_ib_l = *
        adc #0                      ; (Operand: Low-Byte von ins, von music_init)
        sta mus_ip
.const mus_ib_h = *
        lda #0
        adc #0
        sta mus_ip+1
        lda mus_pos,x               ; Position in der Hüllkurve (Werte beginnen bei Byte 2 des Datensatzes)
        clc
        adc #2
        tay
        lda (mus_ip),y              ; Lautstärkewert
        beq @quiet
        lda mus_reg,x
        bne @wr
@quiet: lda #0
@wr:    sta $900a,x                 ; Registerwert ($80 | N) schaltet die Stimme ein, 0 aus
        inc mus_pos,x
        lda mus_pos,x
        cmp #16
        bcc @nx
        ldy #1                      ; Wiederholungsstelle
        lda (mus_ip),y
        sta mus_pos,x
@nx:    dex
        bpl @v
        rts

; Variablen des Players (alle von music_init gesetzt; die Anfangswerte hier gelten nur bis dahin)
mus_on:    .byte 0                  ; 1 = Player läuft
mus_speed: .byte 6                  ; Bilder je Zeile (von music_init)
mus_rows:  .byte 32                 ; Zeilen je Muster (von music_init)
mus_tick:  .byte 0                  ; Bild innerhalb der Zeile (0 = Zeilenanfang)
mus_row:   .byte 0                  ; aktuelle Zeile im Muster
mus_ord:   .byte 0                  ; Position in der Reihenfolge
mus_t:     .byte 0                  ; Nummer des aktuellen Musters
mus_n:     .byte 0                  ; Hilfsspeicher in mus_step: Note der laufenden Stimme
mus_v:     .byte 0                  ; Hilfsspeicher: laufende Stimme 0..3
mus_y:     .byte 0                  ; Hilfsspeicher: Y-Merker (Position in der Zeile)
mus_act:   .byte 0,0,0,0            ; je Stimme: 1 = klingt (Hüllkurve läuft)
mus_ins:   .byte 0,0,0,0            ; je Stimme: aktuelle Instrumentennummer
mus_pos:   .byte 0,0,0,0            ; je Stimme: Schritt in der Hüllkurve (0..15)
mus_reg:   .byte 0,0,0,0            ; je Stimme: VIC-I-Registerwert der aktuellen Note ($80 | N)
mus_fpl:   .byte <mus_f_bass,<mus_f_alto,<mus_f_sopr,<mus_f_nois   ; Frequenztabelle je Stimme, Low-Byte
mus_fph:   .byte >mus_f_bass,>mus_f_alto,>mus_f_sopr,>mus_f_nois   ; und High-Byte
.endmacro
