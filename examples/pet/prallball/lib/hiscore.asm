// SPDX-License-Identifier: GPL-3.0-or-later
// Bestenliste PET (PRALLBALL). Einbinden: .include "lib/hiscore.asm"
// Datei "PRALL.HS" (127 Byte): "PRBL", Version 1, 10 x (8 Zeichen + 3 Byte Punkte lo/mid/hi + 1 Byte Level), CRC-16.
// Zugriff über das KERNAL des PET (BASIC 4) als sequentielle Datei (OPEN/CHKIN/CHRIN, CHKOUT/CHROUT), Diskette am Gerät, von dem das Programm gestartet wurde
// ($D4 beim Start gemerkt; ohne Diskettenstart gibt es keine Dateizugriffe und hs_load/hs_save melden HS_NODEV). Ergebnis in A (HS_*), Z=1 bei HS_OK.
// WICHTIG: Das KERNAL des PET hat keine SETLFS/SETNAM-Einsprunge; die Dateiparameter stehen in der Nullseite ($D1 Namenslaenge, $D2 logische Datei, $D3 Sekundaeradresse,
//   $D4 Geraet, $DA/$DB Namenszeiger).  Fehler beim OPEN (Geraet fehlt) bricht das KERNAL mit einer BASIC-Fehlermeldung ab; darum wird ohne Diskettenstart gar nicht geoeffnet.
//   hs_default   Liste mit Standardeinträgen füllen
//   hs_load      Datei laden; bei Fehler bleibt die Standardliste, A = HS_NODEV/HS_NOFILE/HS_BADSUM
//   hs_save      speichern ("@0:" ersetzt die alte Datei), A = HS_OK/HS_NODEV/HS_WPROT/HS_FULL
//   hs_check     A = Platz 0..9, wenn hs_points (3 Byte) in die Liste kommt, sonst $FF
//   hs_insert    A = Platz; schiebt nach unten, trägt hs_points, hs_level und Name aus hs_name ein
// RAM: hs_buf (127 Byte), hs_points (3), hs_level, hs_name (8 Zeichen, PETSCII-Großbuchstaben)

#include <pet/kernal.asm>

// KERNAL-Nullseite des PET (BASIC 4) fuer OPEN
.const ZP_FNLEN = $d1
.const ZP_LA    = $d2
.const ZP_SA    = $d3
.const ZP_FA    = $d4
.const ZP_FNADR = $da
.const ZP_ST    = $96           // Status der letzten E/A-Operation
// WICHTIG: Die Einsprunge OPEN ($FFC0) und CLOSE ($FFC3) der Sprungtabelle sind beim PET (BASIC 4) die BASIC-ANWEISUNGEN: sie lesen Parameter aus dem BASIC-Text.
//   Das eigentliche KERNAL-OPEN liegt bei $F563 (Parameter in der Nullseite), das KERNAL-CLOSE bei $F2E0 (A = Datei).  CHKIN/CHKOUT/CHRIN/CHROUT/CLRCHN stimmen.
.const OPEN_RAW  = $F563
.const CLOSE_RAW = $F2E0

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

// hs_open: Datei nach Namenszeiger (A = Laenge, X/Y = Name) mit logischer Nummer 1, Sekundaeradresse 2 und dem Startgeraet oeffnen.
hs_open:
        sta ZP_FNLEN
        stx ZP_FNADR
        sty ZP_FNADR + 1
        lda #1
        sta ZP_LA
        lda #2
        sta ZP_SA
        lda hs_bootdev
        sta ZP_FA
        jmp OPEN_RAW

// hs_close: Kanaele zuruecksetzen und Datei 1 schliessen.
hs_close:
        jsr CLRCHN
        lda #1
        jmp CLOSE_RAW

// hs_load: Datei "PRALL.HS" lesen (127 Byte).  Bei JEDEM Fehler bleibt die Standardliste (hs_default) erhalten.  Fehlt die Datei, liefert das
//   Laufwerk keine Daten; das erkennt man am Status ($96) schon vor dem letzten Byte.
hs_load:
        jsr hs_default
        lda hs_on
        bne .hl1
        lda #HS_NODEV
        rts
.hl1:   jsr hs_io_on
        lda #hs_rname_end - hs_rname
        ldx #<hs_rname
        ldy #>hs_rname
        jsr hs_open
        ldx #1
        jsr CHKIN
        ldy #0
.hl2:   jsr CHRIN
        sta hs_buf,y
        iny
        lda ZP_ST
        bne .hl3                // Status ungleich 0: Dateiende oder Fehler
        cpy #HS_SIZE
        bne .hl2
.hl3:   sty hs_cnt
        lda ZP_ST
        sta hs_st
        jsr hs_close
        jsr hs_io_off
        lda hs_cnt
        cmp #HS_SIZE            // alle Byte gelesen?
        bne .nofile
        lda hs_st
        and #$bf                // nur das Dateiende-Bit ($40) ist beim letzten Byte erlaubt
        bne .nofile
        // Kennung pruefen
        ldx #4
.hl4:   lda hs_buf - 1,x
        cmp hs_head - 1,x
        bne .bad
        dex
        bne .hl4
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
.nofile: jsr hs_default
        lda #HS_NOFILE
        rts

// hs_save: Datei schreiben.  Der Name "@0:PRALL.HS,S,W" ersetzt eine vorhandene Datei ("@0:" = Ersetzen auf Laufwerk 0).
hs_save:
        lda hs_on
        bne .hs1
        lda #HS_NODEV
        rts
.hs1:   jsr hs_makecrc
        jsr hs_io_on
        lda #hs_wname_end - hs_wname
        ldx #<hs_wname
        ldy #>hs_wname
        jsr hs_open
        ldx #1
        jsr CHKOUT
        ldy #0
.hs2:   lda hs_buf,y
        jsr CHROUT
        iny
        cpy #HS_SIZE
        bne .hs2
        jsr hs_close
        jsr hs_io_off
        lda #HS_OK
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

// hs_io_on / hs_io_off: vor und nach den Dateiaufrufen.  Der Interrupt (Editor: Tastaturabfrage) bleibt waehrend der IEEE-488-Uebertragung aus.
hs_io_on:
        sei
        rts
hs_io_off:
        cli
        rts

// hs_init: beim Programmstart aufrufen: Startgeraet merken ($D4 = zuletzt benutztes Geraet, nach LOAD von der Diskette 8..15).  Nur dann gibt es Dateizugriffe.
hs_init:
        lda #0
        sta hs_on
        lda ZP_FA
        cmp #8
        bcc .hi9                // Kassette oder kein Geraet: keine Dateizugriffe
        cmp #16
        bcs .hi9
        sta hs_bootdev
        lda #1
        sta hs_on
.hi9:   rts
hs_bootdev: .byte 8
hs_on:  .byte 0
hs_cnt: .byte 0
hs_st:  .byte 0

hs_head:       .byte "PRBL", 1
hs_rname:      .text "0:PRALL.HS,S,R"
hs_rname_end:
hs_wname:      .text "@0:PRALL.HS,S,W"
hs_wname_end:
hs_defname:    .text "SPIELER "
hs_defpts_lo:  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00   // Punkte sind BCD (Dezimalziffern): niedrige zwei Ziffern
hs_defpts_hi:  .byte $10, $09, $08, $07, $06, $05, $04, $03, $02, $01   // mittlere zwei Ziffern: 1000, 900, ... 100 Punkte
hs_points:     .byte 0, 0, 0
hs_level:      .byte 0
hs_name:       .text "SPIELER "
hs_buf:        .fill HS_SIZE, 0
