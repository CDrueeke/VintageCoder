; =====================================================================================================================
; disk.asm - Plattenzugriff der Bestenliste ueber das CIO des Betriebssystems (Datei TROXX.HS)
; =====================================================================================================================
; Herkunft: examples/atari/prallball/lib/hiscore.asm (CIO-Aufrufe), fuer Troxx neu geordnet.
; ZWECK     load liest die Datei (HS_SIZE Byte) nach HS_TMP, save schreibt HS_BUF.  Ergebnis in A (DK_*): DK_OK, DK_NODRIVE (kein Laufwerk oder kein DOS:
;           das Geraet D: fehlt), DK_NOFILE (Datei fehlt), DK_ERR (sonstiger Fehler, z. B. Diskette voll oder schreibgeschuetzt).
; WAHL DER LOESUNG  Ein XEX aus einem Disketten-Image laeuft meist unter einem DOS (DOS 2.5, MyDOS, SpartaDOS ...), das die Datei-Verwaltung und das Geraet D:
;           bereitstellt.  Direkte SIO-Sektoren wuerden auf einer fremden Diskette Dateien zerstoeren; darum arbeitet das Programm ueber CIO mit einer
;           richtigen Datei.  Laeuft das Spiel ohne DOS (Emulator mit -run, Kassette, Modul), meldet CIO einen Fehler (Geraet fehlt): das Spiel laeuft weiter,
;           die Bestenliste bleibt im RAM (Standardliste, in der Sitzung gepflegt).  Das Laufwerk ist das, von dem das Programm geladen wurde (DUNIT).
;           Testbetrieb im Emulator ohne DOS: -D HS_HOSTTEST=1 benutzt das Host-Geraet H1: (atari800 -H1 <Ordner> -hreadwrite).
; WAEHREND DES ZUGRIFFS  Anzeige und DLI sind aus (SIO ist zeitkritisch), nur der Bildruecklauf bleibt (Zeitgeber des SIO); danach stellt end die Anzeige und
;           den Klang wieder her.
; =====================================================================================================================

.const ICCOM  = $0342
.const ICBAL  = $0344
.const ICBAH  = $0345
.const ICBLL  = $0348
.const ICBLH  = $0349
.const ICAX1  = $034a
.const ICAX2  = $034b
.const HS_IOCB = $10                    ; Kanal 1
.const DUNIT_OS = $0301                 ; SIO-Geraetenummer des zuletzt benutzten Laufwerks (1..8)

.const DK_OK      = 0
.const DK_NODRIVE = 1
.const DK_NOFILE  = 2
.const DK_ERR     = 3

disk: {

; init: das Laufwerk des Dateinamens = Ladelaufwerk (Ziffer aus DUNIT, soweit 1..8).  Veraendert A.
init:
.ifndef HS_HOSTTEST
        lda DUNIT_OS
        beq .out
        cmp #9
        bcs .out
        clc
        adc #'0'
        sta FNAME + 1
.endif
.out:   rts

; begin: Anzeige und DLI aus, Klang still.  Veraendert A.
begin:  jsr sound.silence
        lda #0
        sta SDMCTL
        lda #$40
        sta NMIEN
        jmp antic.wait_frame

; end: Anzeige, DLI und Klang wieder an.  Veraendert A.
end:    lda #$3e
        sta SDMCTL
        lda #$c0
        sta NMIEN
        jmp sound.init

; cio: IOCB 1: A = Befehl, Puffer (PTR_C), Laenge X (niedrig) / Y (hoch).  Ergebnis: Status in Y (>= 128 Fehler) und im Z/N-Flag.  Veraendert A, X, Y.
cio:    sta ICCOM + HS_IOCB
        stx ICBLL + HS_IOCB
        sty ICBLH + HS_IOCB
        lda PTR_C
        sta ICBAL + HS_IOCB
        lda PTR_C + 1
        sta ICBAH + HS_IOCB
        ldx #HS_IOCB
        jsr CIOV
        tya
        rts

; open: A = Modus (4 lesen, 8 schreiben) fuer FNAME.  Ergebnis wie cio.
open:   pha
        lda #<FNAME
        sta PTR_C
        lda #>FNAME
        sta PTR_C + 1
        pla
        sta ICAX1 + HS_IOCB
        lda #0
        sta ICAX2 + HS_IOCB
        lda #3                          ; OPEN
        ldx #0
        ldy #0
        jmp cio

; close: Kanal schliessen.  Veraendert A, X, Y.
close:  lda #$0c
        ldx #0
        ldy #0
        jmp cio

; err: A = CIO-Fehlercode -> DK_*.
err:    cmp #170                        ; Datei nicht gefunden
        beq .nf
        cmp #130                        ; Geraet nicht vorhanden (kein DOS)
        beq .nd
        cmp #138                        ; Geraet antwortet nicht
        beq .nd
        cmp #139
        beq .nd
        lda #DK_ERR
        rts
.nf:    lda #DK_NOFILE
        rts
.nd:    lda #DK_NODRIVE
        rts

; load: Datei lesen (HS_SIZE Byte nach HS_TMP).  A = DK_*.  Veraendert A, X, Y, PTR_C.
load:   jsr begin
        lda #4
        jsr open
        bmi .e1
        lda #<HS_TMP
        sta PTR_C
        lda #>HS_TMP
        sta PTR_C + 1
        lda #7                          ; Block lesen
        ldx #HS_SIZE
        ldy #0
        jsr cio
        pha
        jsr close
        pla
        bmi .e2
        jsr end
        lda #DK_OK
        rts
.e1:    pha
        jsr close
        pla
.e2:    cmp #136                        ; Dateiende: zu kurze Datei
        bne .e3
        lda #DK_ERR
        pha
        jmp .fin
.e3:    jsr err
        pha
.fin:   jsr end
        pla
        rts

; save: HS_BUF als Datei schreiben (eine vorhandene Datei wird ersetzt).  A = DK_*.  Veraendert A, X, Y, PTR_C.
save:   jsr begin
        lda #8
        jsr open
        bmi .e1
        lda #<HS_BUF
        sta PTR_C
        lda #>HS_BUF
        sta PTR_C + 1
        lda #$0b                        ; Block schreiben
        ldx #HS_SIZE
        ldy #0
        jsr cio
        pha
        jsr close
        pla
        bmi .e2
        jsr end
        lda #DK_OK
        rts
.e1:    pha
        jsr close
        pla
.e2:    jsr err
        pha
        jsr end
        pla
        rts

.ifdef HS_HOSTTEST
FNAME:  .byte "H1:TROXX.HS", $9b
.else
FNAME:  .byte "D1:TROXX.HS", $9b
.endif
}
