; =====================================================================================================================
; disk.asm - Diskette: Bestenliste laden und speichern (Laufwerk 8, Datei TROXX.HS) ueber das KERNAL
; =====================================================================================================================
; ZWECK     disk.load liest die Datei nach HS_TMP, disk.save schreibt HS_BUF und prueft das Ergebnis durch Zurueckladen und Vergleichen (verify).  Beide
;           melden das Ergebnis in A: DK_OK, DK_NODEV (Laufwerk fehlt), DK_NOFILE (Datei nicht gefunden), DK_ERR (anderer Fehler, zum Beispiel Schreibschutz).  Das Spiel laeuft auch ohne Laufwerk: jeder Fehler
;           wird abgefangen, es bleibt bei der Standardliste.
; HARDWARE  KERNAL-Sprungtabelle (Plus/4 wie C64): SETMSG ($FF90, Meldungen aus), SETLFS ($FFBA, Datei 1, Geraet 8, Sekundaeradresse 0 = an die uebergebene Adresse
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
load:   jsr raster.pause
        jsr load_
        jmp raster.resume
save:   jsr raster.pause
        jsr save_
        jmp raster.resume

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
        jmp verify                      ; Schreibfehler (zum Beispiel Schreibschutz) meldet das KERNAL nicht: die Datei zurueckladen und vergleichen

; verify: Datei TROXX.HS nach HS_TMP laden und mit HS_BUF vergleichen.  A = DK_OK, bei Fehler oder Abweichung DK_ERR (auch wenn die Datei fehlt).
;   (Der Fehlerkanal 15 des Laufwerks waere die uebliche Pruefung; ein OPEN darauf hat im Emulator nach dem Speichern den Rechner abstuerzen lassen,
;   Ursache nicht geklaert; das Zurueckladen benutzt dieselben KERNAL-Aufrufe wie das Laden beim Start und lief in allen Versuchen sauber.)
verify: jsr load_
        cmp #DK_OK
        bne .fail
        ldx #HS_SIZE - 1
.c:     lda HS_TMP,x
        cmp HS_BUF,x
        bne .fail
        dex
        bpl .c
        lda #DK_OK
        rts
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
