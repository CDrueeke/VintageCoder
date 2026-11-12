; hello.asm – PET Hello (Commodore PET/CBM): Text mit der KERNAL-Routine CHROUT ausgeben
;
; Zweck: das kleinste Assemblerprogramm für die PET/CBM-Reihe. Es zeigt .target pet, die BASIC-Startzeile bei $0401 (.basicstub), eine
;   Textschleife mit Terminatorbyte und den Aufruf der KERNAL-Sprungtabelle (CHROUT).
;
; Bedienung: bauen mit `vcc build examples/pet/hello` (oder `vcc asm hello.asm --target pet --cpu nmos6502 -o hello.prg`), PRG laden
;   (VICE: `xpet -model 4032 -autostart hello.prg`; laut Hilfeseite „PET/CBM“ geprüft auf 2001, 4032 und 8032 mit VICE 3.10) und RUN
;   eingeben. Zu sehen ist der Text HELLO FROM VINTAGECODER, danach meldet sich BASIC wieder mit READY, weil das Programm mit RTS endet.
;
; Ablauf: Das Programm gibt Zeichen für Zeichen aus `text` über CHROUT aus, bis das Endebyte 0 erreicht ist. Im Text steht vor der 0 ein
;   Zeilenvorschub (RETURN, Code 13), damit READY in der nächsten Zeile erscheint. Die Schleife verlässt sich darauf, dass CHROUT das
;   X-Register erhält (auf den geprüften Modellen funktioniert das); der Text darf höchstens 256 Bytes lang sein, weil X als Index dient
;   und beim Überlauf auf 0 die Schleife endet.
;
; Text: `.petscii` wandelt in PETSCII-Codes um. Kleingeschriebener Quelltext ergibt die Codes $41-$5A, die der PET der Modelle 2001 bis
;   4032 im Grundzustand (Großbuchstaben/Grafikzeichen) als Großbuchstaben zeigt; der 8032 startet laut Hilfeseite im Groß-/Klein-
;   schreibmodus, dort erscheinen dieselben Codes vermutlich als Kleinbuchstaben (nicht nachgeprüft).
;
; Hardware und Speicher: KERNAL-Sprungtabelle CHROUT = $FFD2 (Zeichen ausgeben, A = Zeichencode; Konstante auch in pet/kernal.asm). Bildschirm
;   ab $8000 (nur von CHROUT angesprochen). BASIC-Programme beginnen auf dem PET bei $0401: Dort liegt die Zeile `10 SYS 1037`; das
;   Maschinenprogramm beginnt bei $040D (= 1037). Zeropage: unbenutzt. Der PET hat keinen 6510-Port bei $00/$01
;   und keine Farbe; daher .cpu nmos6502.
; Grenzen: Spike-Stand der PET-Zieldaten (laut Kopf von pet/hardware.asm sind nur Bildschirmspeicher und CHROUT im Emulator bestätigt).
.target pet
.cpu nmos6502 { illegals = deny }
.org $0401                      ; BASIC-Programmanfang des PET (BASIC 2 und 4)
.basicstub start                ; erzeugt `10 SYS 1037` (Adresse von start)

.const CHROUT = $ffd2           ; KERNAL: Zeichen in A ausgeben (Sprungtabelle, gleiche Adresse wie auf dem C64)

start:  ldx #0                  ; X = Index in den Text
.loop:  lda text,x
        beq .done               ; Endebyte 0: fertig
        jsr CHROUT              ; Zeichen ausgeben (verändert A; X wird erhalten, siehe oben)
        inx
        bne .loop               ; X läuft 0..255; beim Überlauf auf 0 endet die Schleife (höchstens 256 Zeichen)
.done:  rts                     ; zurück ins BASIC (READY.)

text:   .petscii "hello from vintagecoder"      ; PETSCII-Codes ($41-$5A für Buchstaben)
        .byte 13, 0                             ; Zeilenumbruch (RETURN) und Endebyte
