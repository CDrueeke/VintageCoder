; =====================================================================================================================
; music_demo.asm – VC-20: spielt einen mit dem Tracker komponierten Song über den VIC-I (3 Töne + Rauschen)
; =====================================================================================================================
;
; ZWECK
;   Das Beispiel zeigt den kürzesten Weg von einem Tracker-Song zu Musik auf dem VC-20: Der Song music/tune.vcsong (Profil vic20.vic.song) wird beim
;   Bauen nach build/export/tune.asm exportiert (tune_order, tune_patterns, tune_ins und die Konstanten tune_speed, tune_rows, tune_patsize), die
;   Spielroutine lib/music.inc liest ihn, und ein IRQ-Einschub ruft sie einmal pro Bild auf. Man lernt daraus: wie ein Song-Export eingebunden wird,
;   wie man sich hinter den KERNAL-IRQ hängt und wie der VIC-I mit seinen vier Stimmen und der gemeinsamen Lautstärke angesprochen wird.
;
; ZU HÖREN UND ZU SEHEN
;   Eine kurze Schleife (32 Zeilen zu je 6 Bildern, in PAL knapp 4 Sekunden): Bass, Alt und Sopran mit den Instrumenten "Ton" und "Kurz" sowie
;   Rauschstöße ("Tick") als Schlagzeug. Der Bildschirm bleibt, wie BASIC ihn hinterlassen hat; das Programm gibt nichts aus, der Cursor blinkt
;   weiter, weil der KERNAL-IRQ weiterläuft.
;
; BAUEN UND STARTEN
;   Projekt music_demo.vcprj (VC-20 PAL, Speicherausbau "unexpanded", CPU 6502): "vcc build examples/vic20/music_demo" exportiert zuerst den Song
;   und baut dann build/music_demo.prg; ohne den Export fehlt build/export/tune.asm. Starten mit LOAD und RUN (die BASIC-Zeile "10 SYS" erzeugt
;   .basicstub); beendet wird mit Reset. Für NTSC die Tabelle wählen: "-D NTSC=1" (oder .const NTSC = 1 vor dem Einbinden von lib/vic_freq.inc); das
;   Tempo hängt außerdem am Fernsehstandard (siehe unten).
;
; WAS DER CODE AUF DER HARDWARE TUT
;   start sperrt die Interrupts, bereitet den Player vor (music_init: Selbstmodifikation der Adressen, Zustand löschen, Stimmen aus, Lautstärke 15),
;   biegt den IRQ-Vektor CINV ($0314/$0315) auf irq um und gibt die Interrupts frei. Danach tut die Hauptschleife nichts mehr. Jeder IRQ-Takt des
;   KERNALs (Timer 1 der VIA 2, $9124: PAL 50 Hz, NTSC 60 Hz) ruft irq: Dort läuft music_play und springt dann in die ursprüngliche KERNAL-Routine
;   weiter (Tastatur, Cursor, Uhr, Rückkehr mit RTI). Der KERNAL-IRQ-Einsprung hat A, X und Y schon gerettet, music_play darf sie also frei
;   verwenden.
;   Register: VIC-I $900A Bass, $900B Alt, $900C Sopran, $900D Rauschen (Bit 7 = ein, Bits 0..6 = Tonhöhe) und $900E (Lautstärke 0..15 in Bits 0..3,
;   Hilfsfarbe in Bits 4..7, bleibt erhalten). Der VIC-I hat keine Hüllkurven je Stimme; die Lautstärke gilt für alle Stimmen gemeinsam (siehe
;   lib/music.inc).
;
; SPEICHERPLAN (VC-20 ohne Erweiterung)
;   $1001  BASIC-Zeile "10 SYS" (.basicstub), Start bei $100D      Zeropage $FB-$FE: Zeiger des Players (music.inc)
;   danach Programm (start, irq, old_irq), music_runtime (Spielroutine und Variablen), Frequenztabellen (4 x 96 Byte, lib/vic_freq.inc),
;   Songdaten (tune_order, tune_patterns 256 Byte, tune_ins 96 Byte); zusammen derzeit $1001-$14C1 (1217 Byte). Bildschirm $1E00 und Farb-RAM $9600 bleiben unberührt.
;
; BEKANNTE GRENZEN
;   Ein Song, keine Steuerung (kein Stopp, keine Pause); die Songgeschwindigkeit hängt am Fernsehstandard, weil pro IRQ-Takt ein Bild gezählt wird;
;   die Frequenztabelle gilt nur für den gewählten Standard (PAL ohne -D NTSC=1).
; =====================================================================================================================
.target vic20
.cpu 6502

.org $1001                          ; BASIC-Start des Grundgeräts ($1001, Projekteinstellung "unexpanded")
.basicstub start                    ; erzeugt "10 SYS <start>", damit RUN das Programm startet

.include "lib/music.inc"

start:  sei                         ; Interrupts sperren, solange Player und IRQ-Vektor umgebaut werden
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize   ; Player mit den Exportdaten vorbereiten
        lda $0314                   ; KERNAL-IRQ merken und dahinter einhängen
        sta old_irq
        lda $0315
        sta old_irq+1
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315                   ; ab hier ruft der KERNAL-IRQ zuerst irq auf
        cli
loop:   jmp loop                    ; Endlosschleife: alles Weitere geschieht im IRQ

; irq: eingehängte IRQ-Routine, einmal pro KERNAL-Takt (PAL 50 Hz / NTSC 60 Hz). A, X, Y sind vom KERNAL-Einsprung schon gerettet.
irq:    jsr music_play              ; eine Spielstufe: Zeile weiterschalten, Hüllkurven, VIC-I-Register schreiben
        jmp (old_irq)               ; weiter in die ursprüngliche KERNAL-Routine (Tastatur, Cursor, Uhr, RTI)

old_irq: .word 0                    ; gemerkter alter IRQ-Vektor (von start gesetzt)

music_runtime                       ; Spielroutine und Variablen des Players, hier EINMAL hinter dem Hauptprogramm (nicht ausgeführt)
.include "lib/vic_freq.inc"         ; Frequenztabellen (Note 1..96 -> Registerwert) für Bass, Alt, Sopran und Rauschen, PAL oder mit NTSC=1 NTSC
.include "build/export/tune.asm"    ; Songdaten aus dem Tracker-Export (entsteht beim Bauen aus music/tune.vcsong, nicht im Quellbaum)
