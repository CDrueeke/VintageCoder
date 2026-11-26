; =====================================================================================================================
; disk.asm - Diskette: Bestenliste laden und speichern (Laufwerk 8, Datei TROXX.HS) ueber das KERNAL
; =====================================================================================================================
; ZWECK     disk.load liest die Datei nach HS_TMP, disk.save schreibt HS_BUF.  Beide melden das Ergebnis in A: DK_OK, DK_NODEV (Laufwerk fehlt),
;           DK_NOFILE (Datei nicht gefunden), DK_ERR (anderer Fehler, zum Beispiel Schreibschutz).  Das Spiel laeuft auch ohne Laufwerk: jeder Fehler
;           wird abgefangen, es bleibt bei der Standardliste.
; HARDWARE  KERNAL-Sprungtabelle: SETMSG ($FF90, Meldungen aus), SETLFS ($FFBA, Datei 1, Geraet 8, Sekundaeradresse 0 = an die uebergebene Adresse
;           laden / 1 = mit der Adresse aus dem Dateikopf speichern), SETNAM ($FFBD), LOAD ($FFD5), SAVE ($FFD8; A = Zeropage-Adresse des
;           Anfangszeigers, X/Y = Endadresse + 1).  Fehler kommen im Uebertrag (C = 1) mit der KERNAL-Fehlernummer in A (4 Datei nicht gefunden,
;           5 Geraet fehlt).  Der Interrupt bleibt gesperrt (SEI): der serielle Bus arbeitet auch so; nach jedem Aufruf wird SEI wiederholt.
;           "@0:" vor dem Namen ersetzt eine vorhandene Datei.
; =====================================================================================================================

.const DK_OK     = 0
.const DK_NODEV  = 1
.const DK_NOFILE = 2
.const DK_ERR    = 3
.const DEVICE    = 8

disk: {

; load / save: Rasterinterrupt waehrend des Zugriffs sperren
load:
.if FX_RASTER
        jsr raster.pause
.endif
        jsr load_
.if FX_RASTER
        jmp raster.resume
.else
        rts
.endif
save:
.if FX_RASTER
        jsr raster.pause
.endif
        jsr save_
.if FX_RASTER
        jmp raster.resume
.else
        rts
.endif

; load_: Datei TROXX.HS nach HS_TMP laden.  A = DK_*, Z passend.  Veraendert A, X, Y.
load_:
        lda #0
        jsr SETMSG
        lda #FNAME_END - FNAME
        ldx #<FNAME
        ldy #>FNAME
        jsr SETNAM
        lda #1
        ldx #DEVICE
        ldy #0
        jsr SETLFS
        lda #0                          ; 0 = laden
        ldx #<HS_TMP
        ldy #>HS_TMP
        jsr LOAD
        sei
        bcs err
        lda #DK_OK
        rts

; save: HS_BUF (HS_SIZE Byte) in die Datei schreiben, eine vorhandene Datei ersetzend.  A = DK_*.  Veraendert A, X, Y, T0, PTR_C.
save_:
        lda #0
        jsr SETMSG
        lda #WNAME_END - WNAME
        ldx #<WNAME
        ldy #>WNAME
        jsr SETNAM
        lda #1
        ldx #DEVICE
        ldy #1
        jsr SETLFS
        lda #<HS_BUF
        sta PTR_C
        lda #>HS_BUF
        sta PTR_C + 1
        lda #PTR_C
        ldx #<(HS_BUF + HS_SIZE)
        ldy #>(HS_BUF + HS_SIZE)
        jsr SAVE
        sei
        bcs err
        jmp dos_status                  ; Schreibfehler (zum Beispiel Schreibschutz) meldet nur der Fehlerkanal des Laufwerks, nicht das KERNAL

; dos_status: Fehlerkanal 15 des Laufwerks lesen: "00" am Anfang = alles in Ordnung.  A = DK_OK oder DK_ERR.  Veraendert A, X, Y, T0.
dos_status:
        lda #0
        tax
        tay
        jsr SETNAM                      ; ohne Namen
        lda #15
        ldx #DEVICE
        ldy #15
        jsr SETLFS
        jsr OPEN
        bcs .bad
        ldx #15
        jsr CHKIN
        bcs .bad
        jsr CHRIN
        sta T0                          ; erste Ziffer
        jsr CHRIN                       ; zweite Ziffer in A
        pha
        jsr CLRCHN
        lda #15
        jsr CLOSE
        sei
        pla
        cmp #$30                        ; "0"
        bne .fail
        lda T0
        cmp #$30
        bne .fail
        lda #DK_OK
        rts
.bad:   jsr CLRCHN
        lda #15
        jsr CLOSE
        sei
.fail:  lda #DK_ERR
        rts

; err: Fehlernummer in A (KERNAL) -> DK_*.
err:    cmp #5
        beq .nodev
        cmp #4
        beq .nofile
        lda #DK_ERR
        rts
.nodev: lda #DK_NODEV
        rts
.nofile:
        lda #DK_NOFILE
        rts

FNAME:  .text "TROXX.HS"
FNAME_END:
WNAME:  .text "@0:TROXX.HS"
WNAME_END:
}
