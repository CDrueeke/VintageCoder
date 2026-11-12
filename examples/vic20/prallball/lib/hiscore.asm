// SPDX-License-Identifier: GPL-3.0-or-later
// Bestenliste VC-20 (PRALLBALL). Einbinden: .include "lib/hiscore.asm"
// Datei "PRALL.HS" (127 Byte): "PRBL", Version 1, 10 x (8 Zeichen + 3 Byte Punkte lo/mid/hi + 1 Byte Level), CRC-16.
// Zugriff über KERNAL (Diskette, Gerät, von dem das Programm gestartet wurde ($BA beim Start gemerkt), sonst 8). Ergebnis in A (HS_*), Z=1 bei HS_OK.
//   hs_default   Liste mit Standardeinträgen füllen
//   hs_load      Datei laden; bei Fehler bleibt die Standardliste, A = HS_NODEV/HS_NOFILE/HS_BADSUM
//   hs_save      speichern ("@0:" ersetzt die alte Datei), A = HS_OK/HS_NODEV/HS_WPROT/HS_FULL
//   hs_check     A = Platz 0..9, wenn hs_points (3 Byte) in die Liste kommt, sonst $FF
//   hs_insert    A = Platz; schiebt nach unten, trägt hs_points, hs_level und Name aus hs_name ein
// RAM: hs_buf (127 Byte), hs_points (3), hs_level, hs_name (8 Zeichen, PETSCII-Großbuchstaben)

#include <vic20/kernal.asm>

.const HS_OK     = 0
.const HS_NODEV  = 1
.const HS_NOFILE = 2
.const HS_BADSUM = 3
.const HS_WPROT  = 4
.const HS_FULL   = 5

.const HS_SIZE   = 127
.const HS_ENTRIES = 10
.const HS_ENTRY  = 12            // 8 Name + 3 Punkte + 1 Level
.const HS_FIRST  = 5             // Offset des ersten Eintrags (4 Kennung + 1 Version)

.ifndef HS_ZP
.const HS_ZP = $08
.endif
.const hs_ptr = HS_ZP            // 2 Byte
.const hs_src = HS_ZP + 2        // 2 Byte
.const hs_crc = HS_ZP + 4        // 2 Byte

hs_default:
        ldx #0
-       lda hs_head,x
        sta hs_buf,x
        inx
        cpx #5
        bne -
        lda #<(hs_buf + HS_FIRST)
        sta hs_ptr
        lda #>(hs_buf + HS_FIRST)
        sta hs_ptr + 1
        ldx #0                  // Eintrag 0..9: Punkte 1000 - 100 * Nr
.entry: ldy #0
-       lda hs_defname,y
        sta (hs_ptr),y
        iny
        cpy #8
        bne -
        // Punkte = (10 - x) * 100 (passt in 16 Bit): Tabelle
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

// CRC-16/CCITT über die ersten HS_SIZE-2 Byte; legt das Ergebnis in die letzten 2 Byte.
// Warum eine Pruefsumme: Eine halb geschriebene oder von Hand veraenderte Datei soll nicht als gueltige Liste durchgehen.  CRC-16/CCITT
//   (Polynom $1021, Startwert $FFFF), bitweise gerechnet: langsam (~14000 Takte), aber nur beim Laden/Speichern noetig und klein.
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

// hs_load: Datei "PRALL.HS" ueber das KERNAL laden.  SETLFS(Datei 1, Geraet, Sekundaeradresse 0) bedeutet: an die in X/Y uebergebene
//   Adresse laden statt an die im Dateikopf stehende.  Fehler kommen im Uebertrag (C=1) mit KERNAL-Fehlernummer in A
//   (4 = Datei nicht gefunden, 5 = Geraet fehlt, ...).  Bei JEDEM Fehler bleibt die Standardliste (hs_default) erhalten.
hs_load:
        jsr hs_default
        jsr hs_io_on
        lda #hs_fname_end - hs_fname
        ldx #<hs_fname
        ldy #>hs_fname
        jsr SETNAM
        lda #1
        jsr hs_device
        ldy #0                  // Sekundäradresse 0: an die in X/Y gegebene Adresse laden
        jsr SETLFS
        lda #0                  // 0 = LOAD
        ldx #<hs_buf
        ldy #>hs_buf
        jsr LOAD
        php
        pha
        jsr hs_io_off
        pla
        plp
        bcs .err
        // Kennung prüfen
        ldx #4
-       lda hs_buf - 1,x
        cmp hs_head - 1,x
        bne .bad
        dex
        bne -
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
.err:   pha
        jsr hs_default
        pla
        cmp #5
        beq .nodev
        lda #HS_NOFILE
        rts
.nodev: lda #HS_NODEV
        rts

// hs_save: Datei schreiben.  Der Name "@0:PRALL.HS" ersetzt eine vorhandene Datei ("@0:" = Ersetzen auf Laufwerk 0).  SAVE bekommt
//   den Anfang ueber einen Zeiger in der Nullseite (hs_src) und das Ende (X/Y = Adresse + 1).  Der Dateikopf enthaelt die
//   Ladeadresse von hs_buf; beim Laden wird sie ignoriert (Sekundaeradresse 0).
hs_save:
        jsr hs_makecrc
        jsr hs_io_on
        lda #hs_wname_end - hs_wname
        ldx #<hs_wname
        ldy #>hs_wname
        jsr SETNAM
        lda #1
        jsr hs_device
        ldy #1
        jsr SETLFS
        lda #<hs_buf
        sta hs_src
        lda #>hs_buf
        sta hs_src + 1
        lda #hs_src
        ldx #<(hs_buf + HS_SIZE)
        ldy #>(hs_buf + HS_SIZE)
        jsr SAVE
        php
        pha
        jsr hs_io_off
        pla
        plp
        bcs .serr
        lda #HS_OK
        rts
.serr:  cmp #5
        beq .snodev
        lda #HS_WPROT
        rts
.snodev: lda #HS_NODEV
        rts

// hs_check: Platz 0..9 oder $FF. Vergleicht 24-Bit-Punkte (lo, mid, hi) mit jedem Eintrag, von oben nach unten.
hs_check:
        lda #<(hs_buf + HS_FIRST + 8)
        sta hs_ptr
        lda #>(hs_buf + HS_FIRST + 8)
        sta hs_ptr + 1
        ldx #0
.next:  ldy #2                  // hi, mid, lo vergleichen
-       lda hs_points,y
        cmp (hs_ptr),y
        bcc .lower              // kleiner: weiter
        bne .found              // größer: Platz gefunden
        dey
        bpl -
        jmp .lower              // alle drei Bytes gleich: dahinter einsortieren (Y ist hier $FF; ein Zweig auf Z waere falsch)
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

// hs_insert: A = Platz. Einträge ab dem Platz um einen nach unten schieben (letzter fällt heraus).
hs_insert:
        tax
        lda #HS_FIRST           // Offset = HS_FIRST + 12 * Platz
-       cpx #0
        beq +
        clc
        adc #HS_ENTRY
        dex
        bne -
+       sta hs_src              // hs_src = Offset des Zielplatzes
        ldx #HS_FIRST + (HS_ENTRIES - 1) * HS_ENTRY - 1     // letztes Byte, das noch verschoben wird
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

// hs_io_on / hs_io_off: vor und nach den KERNAL-Dateiaufrufen.  hs_io_on schaltet die Systemmeldungen aus (sonst schriebe das KERNAL "SEARCHING ..." in den
//   Bildschirm); die Interrupts bleiben gesperrt (die seriellen Routinen kommen damit aus).  WICHTIG: Port A von VIA 1 ($9111/$9113) gehoert zum Teil dem
//   seriellen Bus und darf von der Eingabe nicht umgestellt werden (frueherer Fehler: "DEVICE NOT PRESENT").
hs_io_on:
        lda #0
        jsr SETMSG
        rts
hs_io_off:
        rts

// hs_init: beim Programmstart aufrufen: Startgeraet merken (das KERNAL veraendert $BA bei Dateizugriffen).  hs_device: X = Geraet, ausserhalb 8..15 gilt 8.
hs_init:
        lda $ba
        sta hs_bootdev
        rts
hs_bootdev: .byte 8
hs_device:
        ldx hs_bootdev
        cpx #8
        bcc .hd8
        cpx #16
        bcc .hd9
.hd8:   ldx #8
.hd9:   rts

hs_head:       .byte "PRBL", 1
hs_fname:      .text "PRALL.HS"
hs_fname_end:
hs_wname:      .text "@0:PRALL.HS"
hs_wname_end:
hs_defname:    .text "SPIELER "
hs_defpts_lo:  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00   // Punkte sind BCD (Dezimalziffern): niedrige zwei Ziffern
hs_defpts_hi:  .byte $10, $09, $08, $07, $06, $05, $04, $03, $02, $01   // mittlere zwei Ziffern: 1000, 900, ... 100 Punkte
hs_points:     .byte 0, 0, 0
hs_level:      .byte 0
hs_name:       .text "SPIELER "
hs_buf:        .fill HS_SIZE, 0
