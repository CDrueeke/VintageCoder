; XEX-Bootloader für Atari-Disketten (128-Byte-Sektoren) – Teil von VintageCoder.
; Der Builder setzt per -D:  XEX_SECTOR = erster Datensektor, XEX_LEN = Länge der XEX-Daten in Bytes.
; Das OS lädt 3 Boot-Sektoren nach $0700 und springt nach $0706. Die XEX-Segmente liegen ab XEX_SECTOR fortlaufend auf der Diskette.
; Überlappe den Bereich $0700-$08FF (Loader und Sektorpuffer) nicht.
.target atari8
.cpu sally { illegals = deny }

.const BOOT = $0700
.const BUF = $0880              ; Sektorpuffer (128 Byte), direkt hinter den 3 Boot-Sektoren
.const DCB = $0300
.const SIOV = $e459
.const RUNAD = $02e0
.const INITAD = $02e2
.const COLBK = $d01a

.const zp_ptr = $e0             ; Zielzeiger (2)
.const zp_cnt = $e2             ; verbleibende Segmentbytes (2)
.const zp_sec = $e4             ; nächster Sektor (2)
.const zp_pos = $e6             ; Position im Puffer
.const zp_rem = $e7             ; verbleibende Bytes des Datenstroms (2)
.const zp_try = $e9             ; Wiederholungen

.org BOOT
        .byte 0                 ; Flags
        .byte 3                 ; Anzahl Boot-Sektoren
        .word BOOT              ; Ladeadresse
        .word boot_start        ; DOSINI (wird nie benutzt: der Loader kehrt nicht zurück)

boot_start:
        lda #<XEX_SECTOR
        sta zp_sec
        lda #>XEX_SECTOR
        sta zp_sec+1
        lda #<XEX_LEN
        sta zp_rem
        lda #>XEX_LEN
        sta zp_rem+1
        lda #128
        sta zp_pos              ; Puffer gilt als leer: das erste getbyte lädt einen Sektor
        lda #0
        sta INITAD
        sta INITAD+1
        sta RUNAD
        sta RUNAD+1

next_seg:
        jsr getbyte
        bcs finish
        sta zp_ptr
        jsr getbyte
        bcs finish
        sta zp_ptr+1
        lda zp_ptr
        and zp_ptr+1
        cmp #$ff
        beq next_seg            ; $FFFF-Kennung überspringen
        jsr getbyte             ; Endadresse
        bcs finish
        sta zp_cnt
        jsr getbyte
        bcs finish
        sta zp_cnt+1
        sec                     ; Länge = Ende - Start + 1
        lda zp_cnt
        sbc zp_ptr
        sta zp_cnt
        lda zp_cnt+1
        sbc zp_ptr+1
        sta zp_cnt+1
        inc zp_cnt
        bne copy
        inc zp_cnt+1
copy:   jsr getbyte
        bcs finish
        ldy #0
        sta (zp_ptr),y
        inc zp_ptr
        bne c2
        inc zp_ptr+1
c2:     lda zp_cnt
        bne c3
        dec zp_cnt+1
c3:     dec zp_cnt
        lda zp_cnt
        ora zp_cnt+1
        bne copy
        lda INITAD              ; Segment fertig: Init-Routine aufrufen?
        ora INITAD+1
        beq next_seg
        jsr call_init
        lda #0
        sta INITAD
        sta INITAD+1
        jmp next_seg
call_init:
        jmp (INITAD)

finish: lda RUNAD
        ora RUNAD+1
        beq hang
        jmp (RUNAD)
hang:   jmp hang

; liefert das nächste Byte des Datenstroms in A; C=1 am Ende
getbyte:
        lda zp_rem
        ora zp_rem+1
        beq eof
        lda zp_pos
        cmp #128
        bne have
        jsr readsec
        lda #0
        sta zp_pos
have:   ldx zp_pos
        lda BUF,x
        inc zp_pos
        lda zp_rem
        bne d1
        dec zp_rem+1
d1:     dec zp_rem
        lda BUF,x
        clc
        rts
eof:    sec
        rts

; liest Sektor zp_sec nach BUF (bis zu 4 Versuche), erhöht zp_sec
readsec:
        lda #4
        sta zp_try
retry:  lda #$31
        sta DCB                 ; DDEVIC: Laufwerk
        lda #1
        sta DCB+1               ; DUNIT
        lda #$52
        sta DCB+2               ; DCOMND: Sektor lesen
        lda #$40
        sta DCB+3               ; DSTATS: Daten empfangen
        lda #<BUF
        sta DCB+4
        lda #>BUF
        sta DCB+5
        lda #$0f
        sta DCB+6               ; DTIMLO
        lda #0
        sta DCB+7
        lda #128
        sta DCB+8               ; DBYTLO
        lda #0
        sta DCB+9
        lda zp_sec
        sta DCB+10              ; DAUX1/2: Sektornummer
        lda zp_sec+1
        sta DCB+11
        jsr SIOV
        lda DCB+3
        bpl ok
        dec zp_try
        bne retry
fail:   inc COLBK               ; dauerhafter Lesefehler: Rahmen flackert
        jmp fail
ok:     inc zp_sec
        bne done
        inc zp_sec+1
done:   rts
