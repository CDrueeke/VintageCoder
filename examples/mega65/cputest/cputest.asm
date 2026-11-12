; =====================================================================================================================
; cputest.asm - Selbsttest für den Befehlssatz der 45GS02 (MEGA65)
; =====================================================================================================================
; Der 45GS02 ist der Prozessor des MEGA65: ein 65CE02 mit Z-Register, 16-Bit-Befehlen, 32-Bit-Q-Register und 32-Bit-Zeigern.
; Dieses Programm führt von jeder Gruppe einen oder wenige Befehle aus, legt das Ergebnis in der Zeropage ab und bleibt dann in einer
; Endlosschleife (Marke `done`) stehen.  Es gibt nichts auf dem Bildschirm aus: ein Prüfprogramm (oder man selbst im Monitor) vergleicht
; danach die Zeropage mit den Erwartungswerten, die als Kommentar hinter jedem Speicherbefehl stehen.
;
; Bedienung / Ablauf
;   * Bauen:  vcc asm cputest.asm --target mega65 --cpu 45gs02 -o cputest.prg   (PRG mit Ladeadresse $2000, kein BASIC-Starter;
;     mit --format bin das reine Abbild ohne Ladeadresse, wie es crates/vc-ide/tests/debug_vice.rs für Xemu lädt).
;   * Starten: ohne BASIC/KERNAL, z. B. im Emulator Xemu oder auf der Hardware per UART-Monitor laden und mit `g 2000` starten.
;   * Danach den Speicher lesen: die Zeropage $60-$D4 (Liste unten) muss die genannten Werte enthalten.  Haltepunkt: Marke `done`.
;   * Voraussetzung: Basisseitenregister B = 0 (die "Zeropage" des 45GS02 liegt bei B * 256; dieses Programm setzt B nicht),
;     und bei $3001 sowie im Chip-RAM bei $10000/$10001 darf geschrieben werden (das ist ohne KERNAL/DOS unkritisch).
;
; Geprüfte Befehlsgruppen (Ergebnisse in der Zeropage)
;   Z-Register: LDZ, TZA, INZ, DEZ                       $60-$62        16 Bit: INW, DEW, ASW                         $63-$68
;   NEG, ASR (Akkumulator)                               $69, $6A       lange Sprünge LBRA, BSR/RTS                   $6B-$6D
;   Bitbefehle BBS/BBR, RMB/SMB                          $6E, $6F, $D3, $D4
;   Q-Register (32 Bit): LDQ, STQ, INQ, ADCQ, NEGQ, ASLQ Q, LSRQ Q    $84-$97
;   32-Bit-Zeiger [$nn],z und ($nn),z                    $A4, $D2       PHW                                           $C0, $C1
;   CPZ und Flags                                        $C2
; Eingaben/Operanden in der Zeropage: $70-$73 (16-Bit-Werte), $80-$83 (Q-Operand), $A0-$A3 (32-Bit-Zeiger), $D0-$D1 (16-Bit-Zeiger).
; Weitere beschriebene Stellen: $3001 (Zielbyte von ($D0),z), $10000/$10001 im Chip-RAM Bank 1 (Zielbytes von [$A0],z).
; Speicherplan: Programm ab $2000 (.org), etwa 290 Byte; sonst nur die genannten Stellen.
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
start:  sei                         ; keine Unterbrechungen: nach dem Start läuft nur dieses Programm
        cle                         ; CLE löscht das E-Flag = 16-Bit-Stapelzeiger (45GS02) ...
        see                         ; ... SEE setzt es wieder = 8-Bit-Stapelzeiger in Seite $01 wie beim 6502 (BSR und PHW benutzen den Stapel)
        ; --- Z-Register: zusätzliches Indexregister, auch Index für ($nn),z und [$nn],z ---
        ldz #$12
        tza                         ; Z in den Akkumulator holen, um es speichern zu können
        sta $60                     ; $12
        inz
        tza
        sta $61                     ; $13
        dez
        dez
        tza
        sta $62                     ; $11
        ; --- 16-Bit-Befehle: arbeiten auf zwei aufeinanderfolgenden Speicherbytes (Low-Byte zuerst) ---
        lda #$ff
        sta $70
        lda #$00
        sta $71
        inw $70                     ; $00FF + 1 = $0100 (Übertrag ins High-Byte)
        lda $70
        sta $63                     ; $00
        lda $71
        sta $64                     ; $01
        dew $70                     ; $0100 - 1 = $00FF
        lda $70
        sta $65                     ; $FF
        lda $71
        sta $66                     ; $00
        lda #$80
        sta $72
        lda #$01
        sta $73
        asw $0072                   ; $0180 << 1 = $0300 (ASW gibt es nur mit absoluter Adresse, daher $0072 statt $72)
        lda $72
        sta $67                     ; $00
        lda $73
        sta $68                     ; $03
        ; --- NEG / ASR auf dem Akkumulator (am 6502 nicht vorhanden) ---
        lda #$01
        neg                         ; Zweierkomplement: -1
        sta $69                     ; $FF
        lda #$81
        asr                         ; arithmetisches Rechtsschieben, das Vorzeichenbit bleibt: %10000001 -> %11000000
        sta $6a                     ; $C0
        ; --- lange Sprünge (16-Bit-Distanz), BSR (Unterprogrammaufruf mit relativer Distanz) ---
        lda #$01
        sta $6b
        lbra skip                   ; unbedingter langer Sprung über die nächsten beiden Befehle
        lda #$02
        sta $6b                     ; wird übersprungen
skip:   bsr sub                     ; Aufruf wie JSR, aber mit relativer Distanz; Rücksprung mit normalem RTS
        lda #$09
        sta $6d                     ; $09 (nach dem Rücksprung)
        bra bits
sub:    lda #$07
        sta $6c                     ; $07
        rts
        ; --- Bitbefehle: BBS/BBR (Sprung, wenn Bit gesetzt/gelöscht), RMB/SMB (Bit löschen/setzen), alle auf Zeropage-Bytes ---
bits:   lda #$04                    ; %00000100: nur Bit 2 gesetzt
        sta $6e
        lda #$00
        sta $6f                     ; Zähler für falsche Sprungentscheidungen
        bbs2 $6e,t1                 ; Bit 2 gesetzt → Sprung
        inc $6f
t1:     bbr3 $6e,t2                 ; Bit 3 gelöscht → Sprung
        inc $6f
t2:     bbs0 $6e,bad                ; Bit 0 gelöscht → kein Sprung
        bra t3
bad:    inc $6f                     ; $6F bleibt $00
t3:     lda #$ff
        sta $d3
        rmb3 $d3                    ; Bit 3 löschen: $FF → $F7
        lda #$00
        sta $d4
        smb7 $d4                    ; Bit 7 setzen: $00 → $80
        ; --- Q-Register (32 Bit, A ist das niederwertigste Byte): Befehle mit dem Präfix NEG NEG ($42 $42) ---
        ; Q-Befehle laden/speichern vier aufeinanderfolgende Bytes, das niederwertigste zuerst.
        lda #$78
        sta $80
        lda #$56
        sta $81
        lda #$34
        sta $82
        lda #$12
        sta $83                     ; $12345678
        ldq $80
        inq
        stq $84                     ; $12345679 → 79 56 34 12
        ldq $80
        clc
        adcq $80
        stq $88                     ; $2468ACF0 → F0 AC 68 24
        ldq $80
        negq
        stq $8c                     ; $EDCBA988 → 88 A9 CB ED
        ldq $80
        aslq q
        stq $90                     ; $2468ACF0 → F0 AC 68 24
        ldq $80
        lsrq q
        stq $94                     ; $091A2B3C → 3C 2B 1A 09
        ; --- 32-Bit-Zeiger [$nn],z (Präfix $EA): vier Zeigerbytes in der Zeropage, hier auf $0001_0000 (Chip-RAM Bank 1) ---
        ; Bank 1 des Chip-RAMs liegt außerhalb des 64-KB-Adressraums der CPU und ist nur über so einen Zeiger erreichbar.
        lda #$00
        sta $a0
        sta $a1
        sta $a3
        lda #$01
        sta $a2                     ; Zeiger = $00010000
        ldz #$00
        lda #$5a
        sta [$a0],z                 ; $10000 = $5A
        ldz #$01
        lda #$5b
        sta [$a0],z                 ; $10001 = $5B (Z wird zum Zeiger addiert)
        lda [$a0],z
        sta $a4                     ; $5B
        ; --- ($nn),z: 16-Bit-Zeiger mit Z als Index (wie ($nn),y, aber mit dem Z-Register) ---
        lda #$99
        sta $3001
        lda #$00
        sta $d0
        lda #$30
        sta $d1                     ; Zeiger = $3000
        ldz #$01
        lda ($d0),z
        sta $d2                     ; $99 (gelesen von $3000 + Z = $3001)
        ; --- PHW: 16-Bit-Wert auf den Stapel legen ---
        phw #$1234
        pla
        sta $c0                     ; $12 (PHW legt das High-Byte oben auf den Stapel)
        pla
        sta $c1                     ; $34
        ; --- CPZ / Flags ---
        ldz #$05
        lda #$00
        sta $c2
        cpz #$05
        bne nz
        lda #$01
        sta $c2                     ; $01 (gleich)
nz:
done:   bra done                    ; Endlosschleife: hier Haltepunkt setzen und die Zeropage lesen
