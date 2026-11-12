; {{name}} – Atari 8-Bit: Hintergrundfarbe = Rasterzeile (WSYNC)
.target atari8
.cpu sally { illegals = deny }
.org $2000
start:  lda #0
        sta antic.nmien         ; VBI/DLI aus
.line:  ldx antic.vcount
        stx antic.wsync
        stx gtia.colbk
        jmp .line
.run start
