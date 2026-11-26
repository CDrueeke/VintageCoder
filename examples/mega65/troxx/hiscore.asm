; =====================================================================================================================
; hiscore.asm - Bestenliste (zehn Eintraege) auf der SD-Karte, Namenseingabe
; =====================================================================================================================
; Herkunft: Aufbau und Dateiformat wie die MEGA65-Fassung von Prallball (examples/mega65/prallball/lib/hiscore.asm; fuer Troxx kopiert und umbenannt:
;   Datei TROXX.HS, Kennung "TRXX"); nicht in der Kopienliste.
; DATEI     "TROXX.HS" (127 Byte): "TRXX", Version 1, 10 x (8 Zeichen Name + 3 Byte Punkte lo/mid/hi (BCD) + 1 Byte Stufe), CRC-16.
; ZUGRIFF   Ueber das KERNAL des C65-Modus, Geraet 8 (Xemu: "XEMU EXTERNAL", echter MEGA65: SD-Karte), als sequenzielle Datei (OPEN/CHRIN/CHROUT).  Das
;           KERNAL ist im Spiel ausgeblendet ($01 = $35, map mit Nullen).  hs_env_in stellt fuer die Dauer eines Zugriffs den Zustand des BASIC-65-Starts
;           wieder her (Speicherabbildung: nur der Block $E000 auf das KERNAL), hs_env_out danach die Spielumgebung und richtet den Grafikchip neu ein.
;           Das Programm merkt beim Start $01 und $D030 (hs_k01, hs_kd030) und laedt die Liste, solange das KERNAL noch sichtbar ist (load).
;           Fehler (kein Laufwerk, keine Datei, falsche Pruefsumme) fuehren zur Standardliste; Speicherfehler werden gemeldet, das Spiel laeuft weiter.
; BEDIENUNG hiscore.load am Programmstart (vor vic.init); hiscore.enter am Spielende mit T0..T2 = Punkte (BCD), T3 = Stufe, A = Spieler: kommt der
;           Endstand in die Liste, fragt ein Fenster nach dem Namen (hoch/runter: Zeichen, links/rechts: Stelle, Feuer: fertig; nach 30 s ohne Eingabe
;           automatisch), dann wird gespeichert.  Mit -D NOSAVE=1 (Tests) bleibt die Datei unberuehrt.
; =====================================================================================================================

.const HS_OK     = 0
.const HS_NODEV  = 1
.const HS_NOFILE = 2
.const HS_BADSUM = 3
.const HS_WPROT  = 4

.const HS_SIZE   = 127
.const HS_ENTRIES = 10
.const HS_ENTRY  = 12                   ; 8 Name + 3 Punkte + 1 Stufe
.const HS_FIRST  = 5                    ; Offset des ersten Eintrags (4 Kennung + 1 Version)
.const HS_ZP     = $f0
.const hs_ptr    = HS_ZP                ; 2 Byte
.const hs_src    = HS_ZP + 2            ; 2 Byte
.const hs_crc    = HS_ZP + 4            ; 2 Byte

hiscore: {

; load: Bestenliste laden (KERNAL muss sichtbar sein: am Programmstart) und die besten Punkte nach HISCORE uebernehmen.  Veraendert A, X, Y.
load:   jsr hs_load_kernal
        jmp take_best

; take_best: HISCORE = Punkte des ersten Listeneintrags.  Veraendert A.
take_best:
        lda hs_buf + HS_FIRST + 8
        sta HISCORE
        lda hs_buf + HS_FIRST + 9
        sta HISCORE + 1
        lda hs_buf + HS_FIRST + 10
        sta HISCORE + 2
        rts

; default: Standardliste (nur im RAM, ohne Datei) fuer Tests.  Veraendert A, X, Y.
default:
        jsr hs_default
        jmp take_best

; enter: Endstand T0..T2 (BCD, niedrigstes zuerst), Stufe T3, Spieler A (0 oder 1) in die Bestenliste eintragen.  Veraendert A, X, Y und mehr.
enter:  lda T0
        sta hs_points
        lda T1
        sta hs_points + 1
        lda T2
        sta hs_points + 2
        lda T3
        sta hs_level
        jsr hs_check                    ; A = Platz 0..9 oder $FF (nicht in der Liste)
        cmp #$ff
        beq .out
        pha                             ; Platz merken
        jsr name_entry
        pla
        jsr hs_insert                   ; Eintrag an den Platz setzen, Rest rutscht nach unten
        jsr take_best
.ifdef NOSAVE
        lda #HS_OK
.else
        jsr hs_save                     ; auf die SD-Karte schreiben; A = HS_OK oder Fehlercode
.endif
        sta hs_res
        jsr hud.clrmsg
        ldx #HT_HS_SAVED
        lda hs_res
        beq .msg
        ldx #HT_HS_NOSAVE
.msg:   jsr hud.print
        ldx #100                        ; zwei Sekunden anzeigen
        jsr delay
.out:   rts

; name_entry: Fenster mit Namenseingabe (8 Zeichen, Vorgabe "SPIELER").  hoch/runter wechselt das Zeichen an der Cursorstelle (Leerzeichen, A-Z, 0-9, im
;   Kreis), links/rechts bewegt den Cursor, Feuer bestaetigt; Wiederholung beim Halten; ohne Eingabe nach 30 Sekunden automatisch.  Veraendert alles.
name_entry:
        lda #40
        sta T6
        lda #10
        sta T7
        lda #2
        ldx #20
        ldy #18
        jsr hud.panel
        ldx #HT_HS_NEW
        jsr hud.print
        ldx #HT_HS_NAME
        jsr hud.print
        ldx #HT_HS_HELP
        jsr hud.print
        ldx #7
.nd:    lda hs_defname,x                ; Vorgabe "SPIELER " in den Namenspuffer
        sta hs_name,x
        dex
        bpl .nd
        lda #0
        sta hs_cur                      ; Cursorstelle
        sta hs_idle
        sta hs_idle + 1
        sta hs_rep                      ; Zaehler der Tastenwiederholung
        sta hs_last                     ; zuletzt gelesene Maske
.ne:    jsr show_name
        jsr tick
        jsr input.read_any
        sta T7
        cmp hs_last
        beq .held
        sta hs_last                     ; neue Eingabe: sofort reagieren
        lda #0
        sta hs_rep
        jmp .act
.held:  lda T7
        beq .quiet
        inc hs_rep
        lda hs_rep
        cmp #25                         ; erst nach 25 Bildern wiederholen, dann alle 6
        bcc .noact
        sec
        sbc #25
.m6:    cmp #6
        bcc .m6d
        sbc #6
        bne .m6
.m6d:   cmp #0
        bne .noact
        jmp .act
.quiet: lda hs_idle                     ; ohne Eingabe: Zaehler der automatischen Bestaetigung
        clc
        adc #1
        sta hs_idle
        bcc .noact
        inc hs_idle + 1
        lda hs_idle + 1
        ldx FPS
        cpx #60
        bne .t50
        cmp #7                          ; 30 s * 60 / 256 = 7
        bcs .done
        bcc .noact
.t50:   cmp #6                          ; 30 s * 50 / 256 = 6
        bcs .done
.noact: jmp .ne
.act:   lda T7
        beq .noact
        lda #0
        sta hs_idle
        sta hs_idle + 1
        lda T7
        and #IN_FIRE
        bne .done
        lda T7
        and #IN_UP
        beq .ndn
        lda #1
        jsr change
        jmp .ne
.ndn:   lda T7
        and #IN_DOWN
        beq .nlf
        lda #$ff
        jsr change
        jmp .ne
.nlf:   lda T7
        and #IN_LEFT
        beq .nrt
        lda hs_cur
        bne .lf1
        jmp .ne
.lf1:   dec hs_cur
        jmp .ne
.nrt:   lda T7
        and #IN_RIGHT
        bne .rt0
        jmp .ne
.rt0:   lda hs_cur
        cmp #7
        bcc .rt1
        jmp .ne
.rt1:   inc hs_cur
        jmp .ne
.done:  ldx #0
        jsr fire_wait_any               ; Feuer loslassen
        rts

; change: Zeichen an der Cursorstelle weiterschalten (A = +1 oder $FF = -1): Leerzeichen, A-Z, 0-9 im Kreis (37 Zeichen).  Veraendert A, X, Y.
change: sta T6
        ldx hs_cur
        lda hs_name,x
        jsr to_idx                      ; A = Nummer 0..36
        clc
        adc T6
        bpl .p
        lda #36
        bne .s
.p:     cmp #37
        bcc .s
        lda #0
.s:     tay
        lda HS_CHARS,y
        sta hs_name,x
        rts

; to_idx: ASCII-Zeichen A (Leerzeichen, A-Z, 0-9) -> Nummer 0..36.  Veraendert X.
to_idx: ldx #36
.f:     cmp HS_CHARS,x
        beq .ok
        dex
        bpl .f
        ldx #0
.ok:    txa
        rts

HS_CHARS: .byte 32
        .for c in range(65, 91)
            .byte c
        .endfor
        .for c in range(48, 58)
            .byte c
        .endfor

; show_name: Name im Fenster ausgeben (Spalte 34.., Zeile 23), die Stelle des Cursors weiss, die uebrigen gelb.  Veraendert A, X, Y, T4, T5, Z, sq, cq.
show_name:
        lda #2
        sta T5
        ldx #0
.c:     stx RT6
        lda #COL_YELLOW
        cpx hs_cur
        bne .col
        lda #COL_WHITE
.col:   sta T4
        lda hs_name,x
        cmp #65
        bcc .ok                         ; Leerzeichen und Ziffern: ASCII = Bildschirmcode
        sec
        sbc #64                         ; ASCII A-Z -> Bildschirmcode 1..26
.ok:    pha
        lda RT6
        clc
        adc #34
        tax
        ldy #23
        pla
        jsr vic.put_char
        ldx RT6
        inx
        cpx #8
        bne .c
        rts

; ---- Bestenliste (wie Prallball)
; hs_default: Liste mit Standardeintraegen fuellen.
hs_default:
        ldx #0
.h:     lda hs_head,x
        sta hs_buf,x
        inx
        cpx #5
        bne .h
        lda #<(hs_buf + HS_FIRST)
        sta hs_ptr
        lda #>(hs_buf + HS_FIRST)
        sta hs_ptr + 1
        ldx #0                          ; Eintrag 0..9: Punkte 1000 - 100 * Nr
.entry: ldy #0
.nm:    lda hs_defname,y
        sta (hs_ptr),y
        iny
        cpy #8
        bne .nm
        lda hs_defpts_lo,x
        sta (hs_ptr),y
        iny
        lda hs_defpts_hi,x
        sta (hs_ptr),y
        iny
        lda #0
        sta (hs_ptr),y
        iny
        lda #1
        sta (hs_ptr),y
        clc
        lda hs_ptr
        adc #HS_ENTRY
        sta hs_ptr
        bcc .nc
        inc hs_ptr + 1
.nc:    inx
        cpx #HS_ENTRIES
        bne .entry
        ; weiter mit hs_makecrc

; hs_makecrc: CRC-16/CCITT ueber die ersten HS_SIZE - 2 Byte; legt das Ergebnis in die letzten 2 Byte.
hs_makecrc:
        jsr hs_calc
        lda hs_crc
        sta hs_buf + HS_SIZE - 2
        lda hs_crc + 1
        sta hs_buf + HS_SIZE - 1
        rts

; hs_calc: CRC-16/CCITT (Polynom $1021, Startwert $FFFF), bitweise gerechnet: langsam (~14000 Takte), aber nur beim Laden/Speichern noetig.
hs_calc:
        lda #$ff
        sta hs_crc
        sta hs_crc + 1
        ldx #0
.byt:   lda hs_buf,x
        eor hs_crc + 1
        sta hs_crc + 1
        ldy #8
.bit:   asl hs_crc
        rol hs_crc + 1
        bcc .nx
        lda hs_crc + 1
        eor #$10
        sta hs_crc + 1
        lda hs_crc
        eor #$21
        sta hs_crc
.nx:    dey
        bne .bit
        inx
        cpx #HS_SIZE - 2
        bne .byt
        rts

; hs_load_kernal: Datei "TROXX.HS" ueber das KERNAL laden (nur mit sichtbarem KERNAL).  Bei JEDEM Fehler bleibt die Standardliste.  A = HS_*.
hs_load_kernal:
        jsr hs_default
        jsr hs_io_setup
        lda #hs_rname_end - hs_rname
        ldx #<hs_rname
        ldy #>hs_rname
        jsr SETNAM
        lda #2                          ; Datei 2, Geraet 8, Sekundaeradresse 2 (sequenzielle Datei lesen)
        ldx #8
        ldy #2
        jsr SETLFS
        jsr OPEN
        bcs .oerr
        ldx #2
        jsr CHKIN
        bcs .cerr
        ldy #0
.rl:    jsr CHRIN                       ; Byte holen; READST <> 0 vor dem letzten Byte = Datei zu kurz oder nicht vorhanden
        sta hs_tmp,y
        pha
        jsr READST
        tax
        pla
        cpx #0
        beq .rok
        cpy #HS_SIZE - 1
        bcc .rbad                       ; Fehler oder Dateiende vor dem letzten Byte
        txa
        and #$bf                        ; am letzten Byte ist Bit 6 (Dateiende) erlaubt
        bne .rbad
.rok:   iny
        cpy #HS_SIZE
        bne .rl
        jsr hs_io_close
        ldx #4                          ; Kennung pruefen
.id:    lda hs_tmp - 1,x
        cmp hs_head - 1,x
        bne .bad
        dex
        bne .id
        ldx #0
.cp:    lda hs_tmp,x
        sta hs_buf,x
        inx
        cpx #HS_SIZE
        bne .cp
        lda hs_buf + HS_SIZE - 2
        pha
        lda hs_buf + HS_SIZE - 1
        pha
        jsr hs_calc
        pla
        cmp hs_crc + 1
        bne .bad2
        pla
        cmp hs_crc
        bne .bad
        lda #HS_OK
        rts
.bad2:  pla
.bad:   jsr hs_default
        lda #HS_BADSUM
        rts
.rbad:  jsr hs_io_close
        jsr hs_default
        lda #HS_NOFILE
        rts
.oerr:  pha
        jsr hs_io_close
        jsr hs_default
        pla
        cmp #5
        beq .nodev
        lda #HS_NOFILE
        rts
.cerr:  jsr hs_io_close
        jsr hs_default
        lda #HS_NOFILE
        rts
.nodev: lda #HS_NODEV
        rts

; hs_io_setup: keine Meldungen auf den Bildschirm, Daten und Dateinamen in Bank 0 (C65/MEGA65-KERNAL).
hs_io_setup:
        lda #0
        jsr SETMSG
        lda #0
        ldx #0
        jmp SETBNK

; hs_io_close: Kanaele zuruecksetzen, Datei 2 schliessen, alle Dateien schliessen.  (Ohne das haengt das naechste OPEN auf dem MEGA65.)
hs_io_close:
        jsr CLRCHN
        lda #2
        jsr CLOSE
        jmp CLALL

; hs_save: Datei schreiben (KERNAL einblenden, zugreifen, Spielumgebung wiederherstellen).  A = HS_*; Z = 1 bei HS_OK.
hs_save:
        jsr hs_env_in
        jsr hs_save_kernal
        sta hs_res
        jsr hs_env_out
        lda hs_res
        rts

; hs_save_kernal: eine vorhandene Datei wird vorher ueber den Befehlskanal geloescht ("S:TROXX.HS"; ein OPEN zum Schreiben auf eine vorhandene Datei
;   haengt auf dem MEGA65), dann als sequenzielle Datei ("TROXX.HS,P,W") geschrieben.
hs_save_kernal:
        jsr hs_makecrc
        jsr hs_io_setup
        lda #hs_sname_end - hs_sname
        ldx #<hs_sname
        ldy #>hs_sname
        jsr SETNAM
        lda #15
        ldx #8
        ldy #15
        jsr SETLFS
        jsr OPEN
        lda #15
        jsr CLOSE
        jsr CLALL
        lda #hs_wname_end - hs_wname
        ldx #<hs_wname
        ldy #>hs_wname
        jsr SETNAM
        lda #2
        ldx #8
        ldy #2
        jsr SETLFS
        jsr OPEN
        bcs .serr
        ldx #2
        jsr CHKOUT
        bcs .serr
        ldy #0
.sl:    lda hs_buf,y
        jsr CHROUT
        iny
        cpy #HS_SIZE
        bne .sl
        jsr hs_io_close
        lda #HS_OK
        rts
.serr:  pha
        jsr hs_io_close
        pla
        cmp #5
        beq .snodev
        lda #HS_WPROT
        rts
.snodev: lda #HS_NODEV
        rts

; hs_check: Platz 0..9 oder $FF.  Vergleicht die 24-Bit-Punkte hs_points mit jedem Eintrag, von oben nach unten.
hs_check:
        lda #<(hs_buf + HS_FIRST + 8)
        sta hs_ptr
        lda #>(hs_buf + HS_FIRST + 8)
        sta hs_ptr + 1
        ldx #0
.next:  ldy #2                          ; hi, mid, lo vergleichen
.cmp:   lda hs_points,y
        cmp (hs_ptr),y
        bcc .lower                      ; kleiner: weiter
        bne .found                      ; groesser: Platz gefunden
        dey
        bpl .cmp
        jmp .lower                      ; alle drei Bytes gleich: dahinter einsortieren
.found: txa
        rts
.lower: clc
        lda hs_ptr
        adc #HS_ENTRY
        sta hs_ptr
        bcc .nc
        inc hs_ptr + 1
.nc:    inx
        cpx #HS_ENTRIES
        bne .next
        lda #$ff
        rts

; hs_insert: A = Platz.  Eintraege ab dem Platz um einen nach unten schieben (der letzte faellt heraus), neuen Eintrag setzen.
hs_insert:
        tax
        lda #HS_FIRST                   ; Offset = HS_FIRST + 12 * Platz
.off:   cpx #0
        beq .go
        clc
        adc #HS_ENTRY
        dex
        bne .off
.go:    sta hs_src                      ; hs_src = Offset des Zielplatzes
        ldx #HS_FIRST + (HS_ENTRIES - 1) * HS_ENTRY - 1     ; letztes Byte, das noch verschoben wird
.shift: cpx hs_src
        bcc .shdone
        lda hs_buf,x
        sta hs_buf + HS_ENTRY,x
        dex
        jmp .shift
.shdone:
        ldx hs_src
        ldy #0
.nm:    lda hs_name,y
        sta hs_buf,x
        inx
        iny
        cpy #8
        bne .nm
        lda hs_points
        sta hs_buf,x
        inx
        lda hs_points + 1
        sta hs_buf,x
        inx
        lda hs_points + 2
        sta hs_buf,x
        inx
        lda hs_level
        sta hs_buf,x
        rts

; ---- MEGA65: KERNAL-Umgebung (siehe Kopf)
hs_env_in:
        php
        sei
        pla
        sta hs_sp                       ; Prozessorstatus (I-Flag) merken
        lda #0
        sta $d01a                       ; Rasterinterrupt des VIC aus
        lda #$ff
        sta $d019                       ; anstehende VIC-Interruptmeldungen quittieren
        lda $01
        sta hs_s01
        lda $d030
        sta hs_sd030
        lda hs_kd030
        sta $d030                       ; zuerst $D030 (die Ein-/Ausgabe ist noch sichtbar) ...
        lda hs_k01
        sta $01                         ; ... dann $01
        lda #$00                        ; MAP im Zustand von SYS: unten nichts abgebildet, oben nur der Block $E000 = KERNAL (Versatz $30000)
        ldx #$00
        ldy #$00
        ldz #$83
        map
        eom
        ldz #0
        rts

hs_env_out:
        lda #$00                        ; Abbildung aufheben: RAM ueberall (wie im Spiel)
        tax
        tay
        taz
        map
        eom
        lda hs_s01
        sta $01
        lda hs_sd030
        sta $d030
        jsr vic.init                    ; Grafikchip neu einrichten (das KERNAL kann Register veraendert haben)
        lda hs_sp
        pha
        plp                             ; I-Flag wie vorher
        rts

hs_head:       .byte "TRXX", 1
hs_rname:      .text "TROXX.HS,P,R"
hs_rname_end:
hs_sname:      .text "S:TROXX.HS"
hs_sname_end:
hs_wname:      .text "TROXX.HS,P,W"
hs_wname_end:
hs_defname:    .text "SPIELER "
hs_defpts_lo:  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00   ; Punkte sind BCD (Dezimalziffern): niedrige zwei Ziffern
hs_defpts_hi:  .byte $10, $09, $08, $07, $06, $05, $04, $03, $02, $01   ; mittlere zwei Ziffern: 1000, 900, ... 100 Punkte
hs_points:     .byte 0, 0, 0
hs_level:      .byte 0
hs_name:       .text "SPIELER "
hs_buf:        .fill HS_SIZE, 0
hs_tmp:        .fill HS_SIZE + 1, 0
hs_k01:        .byte 0
hs_kd030:      .byte 0
hs_s01:        .byte 0
hs_sd030:      .byte 0
hs_sp:         .byte 0
hs_res:        .byte 0
hs_cur:        .byte 0
hs_idle:       .byte 0, 0
hs_rep:        .byte 0
hs_last:       .byte 0
}

