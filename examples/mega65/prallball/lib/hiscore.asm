// SPDX-License-Identifier: GPL-3.0-or-later
// Bestenliste MEGA65 (PRALLBALL). Einbinden: .include "lib/hiscore.asm"
// Datei "PRALL.HS" (127 Byte): "PRBL", Version 1, 10 x (8 Zeichen + 3 Byte Punkte lo/mid/hi + 1 Byte Level), CRC-16.
// MEGA65: Zugriff über das KERNAL des C65-Modus, Gerät 8 (Xemu: "XEMU EXTERNAL", echter MEGA65: SD-Karte), als sequenzielle Datei (OPEN/CHRIN/CHROUT):
// Laden/Schreiben mit LOAD/SAVE hing nach einem erfolgreichen LOAD bzw. beim Anlegen, ein OPEN zum Schreiben auf eine vorhandene Datei hängt ebenfalls
// (auch mit "@0:"); deshalb wird die alte Datei zuvor über den Befehlskanal gelöscht ("S:PRALL.HS").  Ergebnis in A (HS_*), Z=1 bei HS_OK.
//   hs_load_kernal / hs_save_kernal  direkt (nur wenn das KERNAL sichtbar ist, z. B. ganz am Programmstart)
//   hs_load / hs_save                blenden das KERNAL ein (hs_env_in), greifen zu und stellen die Spielumgebung wieder her (hs_env_out, ruft hs_after)
//   hs_default   Liste mit Standardeinträgen füllen
//   hs_load      Datei laden; bei Fehler bleibt die Standardliste, A = HS_NODEV/HS_NOFILE/HS_BADSUM
//   hs_save      speichern ("@0:" ersetzt die alte Datei), A = HS_OK/HS_NODEV/HS_WPROT/HS_FULL
//   hs_check     A = Platz 0..9, wenn hs_points (3 Byte) in die Liste kommt, sonst $FF
//   hs_insert    A = Platz; schiebt nach unten, trägt hs_points, hs_level und Name aus hs_name ein
// RAM: hs_buf (127 Byte), hs_points (3), hs_level, hs_name (8 Zeichen, PETSCII-Großbuchstaben)

#include <mega65/hardware.asm>
#include <mega65/kernal.asm>

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
.const HS_ZP = $f0
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
        jsr hs_env_in
        jsr hs_load_kernal
        sta hs_res
        jsr hs_env_out
        lda hs_res
        rts
hs_load_kernal:
        jsr hs_default
        jsr hs_io_setup
        lda #hs_rname_end - hs_rname
        ldx #<hs_rname
        ldy #>hs_rname
        jsr SETNAM
        lda #2                  // Datei 2, Geraet 8, Sekundaeradresse 2 (Sequenzielle Datei lesen)
        ldx #8
        ldy #2
        jsr SETLFS
        jsr OPEN
        bcs .oerr
        ldx #2
        jsr CHKIN
        bcs .cerr
        ldy #0
.rl:    jsr CHRIN               // Byte holen; READST <> 0 vor dem letzten Byte = Datei zu kurz oder nicht vorhanden
        sta hs_tmp,y
        pha
        jsr READST
        tax
        pla
        cpx #0
        beq .rok
        cpy #HS_SIZE - 1
        bcc .rbad               // Fehler oder Dateiende vor dem letzten Byte
        txa
        and #$bf                // am letzten Byte ist Bit 6 (Dateiende) erlaubt
        bne .rbad
.rok:   iny
        cpy #HS_SIZE
        bne .rl
        jsr hs_io_close
        // Kennung pruefen
        ldx #4
-       lda hs_tmp - 1,x
        cmp hs_head - 1,x
        bne .bad
        dex
        bne -
        ldx #0
-       lda hs_tmp,x
        sta hs_buf,x
        inx
        cpx #HS_SIZE
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

// hs_io_setup: keine Meldungen auf den Bildschirm, Daten und Dateinamen in Bank 0 (C65/MEGA65-KERNAL).
hs_io_setup:
        lda #0
        jsr SETMSG
        lda #0
        ldx #0
        jmp SETBNK
// hs_io_close: Kanaele zuruecksetzen, Datei 2 schliessen, alle Dateien schliessen.  (Ohne das haengt das naechste OPEN auf dem MEGA65.)
hs_io_close:
        jsr CLRCHN
        lda #2
        jsr CLOSE
        jmp CLALL

// hs_save: Datei schreiben.  Eine vorhandene Datei wird vorher ueber den Befehlskanal geloescht ("S:PRALL.HS"); ein OPEN zum Schreiben auf
//   eine vorhandene Datei haengt auf dem MEGA65 (auch mit "@0:").  Danach wird die Datei als sequenzielle Datei ("PRALL.HS,P,W") geschrieben.
hs_save:
        jsr hs_env_in           // KERNAL-Umgebung des BASIC-65-Starts einblenden
        jsr hs_save_kernal
        sta hs_res
        jsr hs_env_out          // Spielumgebung wiederherstellen
        lda hs_res              // Ergebnis; Z = 1 bei HS_OK
        rts
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

hs_head:       .byte "PRBL", 1
hs_rname:      .text "PRALL.HS,P,R"
hs_rname_end:
hs_sname:      .text "S:PRALL.HS"
hs_sname_end:
hs_wname:      .text "PRALL.HS,P,W"
hs_wname_end:
hs_defname:    .text "SPIELER "
hs_defpts_lo:  .byte $00, $00, $00, $00, $00, $00, $00, $00, $00, $00   // Punkte sind BCD (Dezimalziffern): niedrige zwei Ziffern
hs_defpts_hi:  .byte $10, $09, $08, $07, $06, $05, $04, $03, $02, $01   // mittlere zwei Ziffern: 1000, 900, ... 100 Punkte
hs_points:     .byte 0, 0, 0
hs_level:      .byte 0
hs_name:       .text "SPIELER "
hs_buf:        .fill HS_SIZE, 0
hs_tmp:        .fill HS_SIZE + 1, 0

// ---- MEGA65: KERNAL-Umgebung.  Das Spiel schaltet beim Start auf "RAM ueberall" ($01 = $35, $D030 mit Palettenbit, map mit Nullen); das KERNAL
//   des C65-Modus ist dann unsichtbar: es liegt NICHT ueber $01 oder $D030 im Adressraum, sondern ueber die MAP-Abbildung des BASIC-65-Starts
//   (im Leerlauf von BASIC steht der Monitor von Xemu auf MAPL = $E300 / MAPH = $B300 - das gilt NICHT fuer ein SYS-Programm, dessen Code im RAM bei
//   $2001 liegt: dort ist nur $E000 auf das KERNAL abgebildet, MAPL = $0000, MAPH = $8300).  hs_env_in stellt fuer die Dauer eines Dateizugriffs den Zustand des BASIC-65-Starts wieder her; hs_env_out danach die
//   Spielumgebung.  Das Spiel muss beim Start, vor jeder Umschaltung, hs_k01 und hs_kd030 mit $01 und $D030 fuellen (lda $01 / sta hs_k01 /
//   lda $d030 / sta hs_kd030) und eine Marke hs_after bereitstellen, die nach dem Zugriff den Grafikchip neu einrichtet (das KERNAL kann ihn veraendern).
//   Interrupts bleiben waehrend des Zugriffs aus; der Rasterinterrupt des VIC wird abgeschaltet (das KERNAL kennt ihn nicht).
hs_env_in:
        php
        sei
        pla
        sta hs_sp               // Prozessorstatus (I-Flag) merken
        lda #0
        sta $d01a               // Rasterinterrupt des VIC aus
        lda #$ff
        sta $d019               // anstehende VIC-Interruptmeldungen quittieren
        lda $01
        sta hs_s01
        lda $d030
        sta hs_sd030
        lda hs_kd030
        sta $d030               // zuerst $D030 (die Ein-/Ausgabe ist noch sichtbar) ...
        lda hs_k01
        sta $01                 // ... dann $01
        lda #$00                // MAP im Zustand von SYS: unten nichts abgebildet (Programm im RAM), oben nur der Block $E000 = KERNAL (Versatz $30000)
        ldx #$00                //   (A = MAPL niedrig, X = MAPL hoch, Y = MAPH niedrig, Z = MAPH hoch: Bit 7 = Block $E000)
        ldy #$00
        ldz #$83
        map
        eom
        ldz #0
        rts
hs_env_out:
        lda #$00                // Abbildung aufheben: RAM ueberall (wie im Spiel)
        tax
        tay
        taz
        map
        eom
        lda hs_s01
        sta $01
        lda hs_sd030
        sta $d030
        jsr hs_after            // Spiel: Grafikchip und Rasterinterrupt neu einrichten
        lda hs_sp
        pha
        plp                     // I-Flag wie vorher
        rts
hs_k01:    .byte 0
hs_kd030:  .byte 0
hs_s01:    .byte 0
hs_sd030:  .byte 0
hs_sp:     .byte 0
hs_res:    .byte 0
