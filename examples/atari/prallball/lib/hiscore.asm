// SPDX-License-Identifier: GPL-3.0-or-later
// Bestenliste Atari 8 Bit (PRALLBALL). Einbinden: .include "lib/hiscore.asm"
// Datei "D:PRALL.HSC" (127 Byte): "PRBL", Version 1, 10 x (8 Zeichen + 3 Byte Punkte lo/mid/hi als BCD + 1 Byte Level), CRC-16/CCITT.
// Zugriff über das CIO des Betriebssystems (Kanal 1).  Ergebnis in A (HS_*), Z=1 bei HS_OK.
//   hs_dev_init  Laufwerk des Dateinamens = Ladelaufwerk (aus DUNIT); einmal beim Start aufrufen
//   hs_default   Liste mit Standardeinträgen füllen
//   hs_load      Datei laden; bei Fehlern bleibt die Standardliste, A = HS_NODEV/HS_NOFILE/HS_BADSUM
//   hs_save      speichern (OPEN mit Schreibmodus ersetzt die alte Datei), A = HS_OK/HS_NODEV/HS_WPROT/HS_FULL
//   hs_check     A = Platz 0..9, wenn hs_points (3 Byte) in die Liste kommt, sonst $FF
//   hs_insert    A = Platz; schiebt nach unten, trägt hs_points, hs_level und hs_name ein
// RAM: hs_buf (127 Byte), hs_points (3), hs_level, hs_name (8 Zeichen, ASCII-Großbuchstaben)
// CIO-Fehlercodes (Y): 170 Datei nicht gefunden, 138/139 Gerät antwortet nicht, 144 schreibgeschützt/Gerätefehler, 165 Datei gesperrt, 162 Diskette voll.
.const HS_OK     = 0
.const HS_NODEV  = 1
.const HS_NOFILE = 2
.const HS_BADSUM = 3
.const HS_WPROT  = 4
.const HS_FULL   = 5
.const HS_SIZE   = 127
.const HS_ENTRIES = 10
.const HS_ENTRY  = 12
.const HS_FIRST  = 5

.const CIOV   = $e456
.const ICCOM  = $0342
.const ICBAL  = $0344
.const ICBAH  = $0345
.const ICBLL  = $0348
.const ICBLH  = $0349
.const ICAX1  = $034a
.const ICAX2  = $034b
.const HS_IOCB = $10                    // Kanal 1

.ifndef HS_ZP
.const HS_ZP = $e0
.endif
.const hs_ptr = HS_ZP                   // 2 Byte
.const hs_src = HS_ZP + 2               // 2 Byte
.const hs_crc = HS_ZP + 4               // 2 Byte
.const hs_st  = HS_ZP + 6               // CIO-Status (letzter Fehlercode)

// hs_dev_init: Die Bestenliste gehoert auf das Laufwerk, von dem das Spiel gestartet wurde.  Nach dem Laden einer Datei steht in DUNIT ($0301, SIO-Geraetenummer)
//   die Nummer des zuletzt benutzten Laufwerks (1..8) - das ist das Ladelaufwerk.  Sie wird als Ziffer in den Dateinamen "Dn:PRALL.HSC" eingesetzt.
//   Ausserhalb von 1..8 (kein Diskettenstart, z. B. Emulator mit -run) bleibt "D1:".  Mit HS_HOSTTEST bleibt der Name "H1:...".
.const HS_DUNIT = $0301
hs_dev_init:
.ifndef HS_HOSTTEST
        lda HS_DUNIT
        beq .hdi9
        cmp #9
        bcs .hdi9
        clc
        adc #'0'
        sta hs_fname + 1
.endif
.hdi9:  rts

hs_default:
        ldx #0
.hd0:   lda hs_head,x
        sta hs_buf,x
        inx
        cpx #5
        bne .hd0
        lda #<(hs_buf + HS_FIRST)
        sta hs_ptr
        lda #>(hs_buf + HS_FIRST)
        sta hs_ptr + 1
        ldx #0                          // Eintrag 0..9: Punkte (10 - Nr) * 100
.entry: ldy #0
.hd1:   lda hs_defname,y
        sta (hs_ptr),y
        iny
        cpy #8
        bne .hd1
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
        bcc +
        inc hs_ptr + 1
+       inx
        cpx #HS_ENTRIES
        bne .entry
        jsr hs_makecrc
        lda #HS_OK
        rts

// CRC-16/CCITT (Polynom $1021, Startwert $FFFF) über die ersten HS_SIZE-2 Byte; hs_makecrc legt das Ergebnis in die letzten 2 Byte.
hs_makecrc:
        jsr hs_calc
        lda hs_crc
        sta hs_buf + HS_SIZE - 2
        lda hs_crc + 1
        sta hs_buf + HS_SIZE - 1
        rts
hs_calc:
        lda #$ff
        sta hs_crc
        sta hs_crc + 1
        ldx #0
.byte_: lda hs_buf,x
        eor hs_crc + 1
        sta hs_crc + 1
        ldy #8
.bit:   asl hs_crc
        rol hs_crc + 1
        bcc +
        lda hs_crc + 1
        eor #$10
        sta hs_crc + 1
        lda hs_crc
        eor #$21
        sta hs_crc
+       dey
        bne .bit
        inx
        cpx #HS_SIZE - 2
        bne .byte_
        rts

// hs_cio: IOCB 1 mit Befehl A, Puffer hs_ptr, Länge in X (niedrig) / Y (hoch) ausführen; Rückgabe: Status in Y (negativ = Fehler), Z/N aus dem Status.
hs_cio:
        sta ICCOM + HS_IOCB
        stx ICBLL + HS_IOCB
        sty ICBLH + HS_IOCB
        lda hs_ptr
        sta ICBAL + HS_IOCB
        lda hs_ptr + 1
        sta ICBAH + HS_IOCB
        ldx #HS_IOCB
        jsr CIOV
        sty hs_st
        rts

// hs_open: A = Modus (4 lesen, 8 schreiben); Name D:PRALL.HSC.  Rückgabe: Status in hs_st und Y.
hs_open:
        pha
        lda #<hs_fname
        sta hs_ptr
        lda #>hs_fname
        sta hs_ptr + 1
        pla
        sta ICAX1 + HS_IOCB
        lda #0
        sta ICAX2 + HS_IOCB
        lda #3                          // OPEN
        ldx #0
        ldy #0
        jmp hs_cio

hs_close:
        lda #<hs_buf
        sta hs_ptr
        lda #>hs_buf
        sta hs_ptr + 1
        lda #$0c                        // CLOSE
        ldx #0
        ldy #0
        jsr hs_cio
        rts

// hs_status: Fehlercode in A (Y) -> HS_*.
hs_status:
        lda hs_st
        cmp #170
        beq .e_nofile
        cmp #138
        beq .e_nodev
        cmp #139
        beq .e_nodev
        cmp #162
        beq .e_full
        cmp #144
        beq .e_wprot
        cmp #165
        beq .e_wprot
        lda #HS_NOFILE
        rts
.e_nofile: lda #HS_NOFILE
        rts
.e_nodev: lda #HS_NODEV
        rts
.e_full: lda #HS_FULL
        rts
.e_wprot: lda #HS_WPROT
        rts

hs_load:
        jsr hs_default
        lda #4
        jsr hs_open
        lda hs_st
        bpl .ok1
        jsr hs_status
        pha
        jsr hs_close
        pla
        rts
.ok1:   lda #<hs_tmp
        sta hs_ptr
        lda #>hs_tmp
        sta hs_ptr + 1
        lda #7                          // Block lesen (GET CHARACTERS)
        ldx #HS_SIZE
        ldy #0
        jsr hs_cio
        lda hs_st
        pha
        jsr hs_close
        pla
        bpl .rd
        cmp #136                        // 136 = Dateiende (zu kurze Datei)
        beq .bad
        sta hs_st
        jmp hs_status
.rd:    ldx #4
.m1:    lda hs_tmp - 1,x
        cmp hs_head - 1,x
        bne .bad
        dex
        bne .m1
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

hs_save:
        jsr hs_makecrc
        lda #8
        jsr hs_open
        lda hs_st
        bpl .ok2
        jsr hs_status
        pha
        jsr hs_close
        pla
        rts
.ok2:   lda #<hs_buf
        sta hs_ptr
        lda #>hs_buf
        sta hs_ptr + 1
        lda #$0b                        // Block schreiben (PUT CHARACTERS)
        ldx #HS_SIZE
        ldy #0
        jsr hs_cio
        lda hs_st
        pha
        jsr hs_close
        pla
        bpl .okw
        sta hs_st
        jmp hs_status
.okw:   lda #HS_OK
        rts

hs_check:
        lda #<(hs_buf + HS_FIRST + 8)
        sta hs_ptr
        lda #>(hs_buf + HS_FIRST + 8)
        sta hs_ptr + 1
        ldx #0
.next:  ldy #2                          // hi, mid, lo vergleichen
-       lda hs_points,y
        cmp (hs_ptr),y
        bcc .lower
        bne .found
        dey
        bpl -
        jmp .lower                      // gleich: dahinter einsortieren
.found: txa
        rts
.lower: clc
        lda hs_ptr
        adc #HS_ENTRY
        sta hs_ptr
        bcc +
        inc hs_ptr + 1
+       inx
        cpx #HS_ENTRIES
        bne .next
        lda #$ff
        rts

hs_insert:
        tax
        lda #HS_FIRST
-       cpx #0
        beq +
        clc
        adc #HS_ENTRY
        dex
        bne -
+       sta hs_src
        ldx #HS_FIRST + (HS_ENTRIES - 1) * HS_ENTRY - 1
.shift: cpx hs_src
        bcc .shdone
        lda hs_buf,x
        sta hs_buf + HS_ENTRY,x
        dex
        jmp .shift
.shdone:
        ldx hs_src
        ldy #0
-       lda hs_name,y
        sta hs_buf,x
        inx
        iny
        cpy #8
        bne -
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

hs_head:       .byte "PRBL", 1
.ifdef HS_HOSTTEST
hs_fname:      .byte "H1:PRALL.HSC", $9b      // Test im Emulator: atari800 mit -H1 <Ordner> stellt das Geraet H: bereit (ohne DOS)
.else
hs_fname:      .byte "D1:PRALL.HSC", $9b     // Laufwerk: hs_dev_init setzt die Ziffer auf das Laufwerk, von dem das Programm geladen wurde
.endif
hs_defname:    .byte "SPIELER "
hs_defpts_lo:  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00
hs_defpts_hi:  .byte $10, $09, $08, $07, $06, $05, $04, $03, $02, $01
hs_points:     .byte 0, 0, 0
hs_level:      .byte 0
hs_name:       .byte "SPIELER "
hs_buf:        .fill HS_SIZE, 0
hs_tmp:        .fill HS_SIZE + 1, 0
