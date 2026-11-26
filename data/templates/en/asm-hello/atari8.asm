; {{name}} - Atari 8-bit: background color = raster line (WSYNC)
.target atari8
.cpu sally { illegals = deny }
.org $2000
start:  lda #0
        sta antic.nmien         ; VBI/DLI off
.line:  ldx antic.vcount
        stx antic.wsync
        stx gtia.colbk
        jmp .line
.run start
