; music_demo.asm – spielt einen mit dem Tracker komponierten Song über den SID (C64), getaktet von einem Rasterzeilen-IRQ
;
; Zweck: zeigt, wie ein Tracker-Song (music/tune.vcsong) exportiert, eingebunden und abgespielt wird. Die Spielroutine steht in
;   lib/music.inc (Makros music_init, music_play, music_stop, music_runtime); dieses Programm sorgt nur dafür, dass `music_play`
;   genau einmal je Bild aufgerufen wird.
;
; Vorbereitung und Bedienung: `vcc build examples/c64/music_demo` exportiert den Song selbst (von Hand: `vcc media export
;   music/tune.vcsong`; erzeugt build/export/tune.asm). PRG laden, RUN: der Song läuft, bis man RESET drückt. Das Programm hat keine
;   Bedienung; die Tastatur wird nicht mehr abgefragt (der KERNAL-Teil des IRQ mit der Tastaturabfrage wird übergangen).
;
; Der Song (music/tune.vcsong): 3 Stimmen, Tempo 6 Bilder je Zeile, 32 Zeilen je Pattern, 2 Patterns in der Reihenfolge 0, 1, 0, 1 und
;   danach wieder von vorn (loop 0). Stimme 1 spielt mit dem Instrument „Puls“ (Pulswelle) Dreiklangsbrechungen, Stimme 2 mit „Bass“
;   (Sägezahn) einen Basston je acht Zeilen, Stimme 3 mit „Snare“ (Rauschen) einen Schlag je acht Zeilen. Pattern 0 klingt nach C-Dur über C,
;   Pattern 1 nach D-Dur über G. Bei 50 Bildern je Sekunde (PAL) ist eine Zeile 0,12 s lang, ein Pattern 3,84 s, der Song etwa 15 s.
;
; Zeitgeber: Die Spielroutine rechnet in Bildern. Deshalb läuft der Song nicht über den KERNAL-IRQ (CIA-Timer, 60 Hz auch am PAL-C64:
;   der Song wäre 20 % zu schnell), sondern über einen Rasterzeilen-IRQ des VIC-II in Zeile 0: genau ein Aufruf je Bild (PAL 50 Hz,
;   NTSC 60 Hz). Im IRQ wird der KERNAL-Einsprung $EA81 verwendet (nur Register zurückholen und RTI), also ohne Tastatur und Uhr.
;
; Hardware: VIC-II $D011 (Bit 7 = Rasterzeile Bit 8), $D012 (Vergleichszeile), $D019 (Interrupt-Status, quittieren), $D01A
;   (Interrupt-Freigabe); CIA 1 und 2 $DC0D/$DD0D (Interrupt-Steuerregister); SID $D400-$D418 (über music.inc); IRQ-Vektor $0314/$0315.
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061`, Programm ab $080D, danach Spielroutine und Songdaten; der ganze Block endet bei $0C64
;   (1126 Bytes mit Ladeadresse). Zeropage $F7-$FA (Spielroutine, siehe music.inc). Pal/NTSC: Standard ist die PAL-Frequenztabelle,
;   mit -D NTSC=1 die NTSC-Tabelle.
; Grenzen: kein Rücksprung ins BASIC, Ende nur mit RESET; die Zeropage $F7-$FA enthält Zustand der Spielroutine und darf zwischen den
;   Aufrufen von music_play nicht anderweitig benutzt werden.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/music.inc"

start:  sei                         ; während der Einrichtung keine Unterbrechung
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; Spielroutine mit den Songtabellen füllen
        ; Rasterzeilen-IRQ statt KERNAL-IRQ: genau ein Takt je Bild (PAL 50 Hz, NTSC 60 Hz).
        ; Der KERNAL-IRQ läuft auch am PAL-C64 mit 60 Hz und spielte den Song 20 % zu schnell.
        lda #$7f
        sta $dc0d                   ; CIA-Interrupts aus (Bit 7 = 0: die gesetzten Bits 0-6 werden gelöscht)
        sta $dd0d
        lda $dc0d                   ; ausstehende Meldungen quittieren
        lda $dd0d
        lda #$01
        sta $d01a                   ; Raster-IRQ an
        lda #$00
        sta $d012                   ; Rasterzeile 0 (untere acht Bit der Vergleichszeile)
        lda $d011
        and #$7f
        sta $d011                   ; Bit 8 der Vergleichszeile = 0 (Bit 7 von $D011 ist beim Schreiben das Bit 8 des Vergleichswerts)
        lda #<irq
        sta $0314                   ; KERNAL-IRQ-Vektor auf die eigene Routine umbiegen
        lda #>irq
        sta $0315
        cli                         ; Interrupts frei: ab jetzt läuft die Musik
loop:   jmp loop                    ; Hauptprogramm hat nichts zu tun

; irq – Rasterzeilen-Interrupt, einmal je Bild bei Zeile 0 (über den KERNAL-Einsprung $FF48/$0314; Register sind dort schon gerettet).
;   Quittiert die Unterbrechung, spielt einen Tick der Musik und verlässt den Interrupt über das KERNAL-Ende bei $EA81.
irq:    lda #$01
        sta $d019                   ; Raster-IRQ quittieren
        jsr music_play
        jmp $ea81                   ; Register zurückholen, RTI

music_runtime                       ; Spielroutine (music_play, mus_step, Zustand, Frequenztabelle) – hinter dem Hauptprogramm
.include "build/export/tune.asm"    ; Songdaten: tune_order, tune_patterns, tune_ins und die Konstanten tune_speed, tune_rows, tune_patsize
