// Allgemein: Bitmasken, Prozessor-Flags und Makros für Bytes, Wörter und Zeiger
// Einbinden mit: #include <common/types.asm>

// ---- Bitmasken
.const BIT0 = $01
.const BIT1 = $02
.const BIT2 = $04
.const BIT3 = $08
.const BIT4 = $10
.const BIT5 = $20
.const BIT6 = $40
.const BIT7 = $80

// ---- Prozessor-Statusbits (für Vergleiche mit dem gepushten Statuswert)
.const P_CARRY     = $01
.const P_ZERO      = $02
.const P_IRQ_OFF   = $04
.const P_DECIMAL   = $08
.const P_BREAK     = $10
.const P_OVERFLOW  = $40
.const P_NEGATIVE  = $80

.const FALSE = 0
.const TRUE  = 1

// ---- Bits in einer Speicherstelle (verändert A)
.macro set_bits(addr, mask)
 lda addr
 ora #mask
 sta addr
.endmacro

.macro clear_bits(addr, mask)
 lda addr
 and #(mask) ^ $ff
 sta addr
.endmacro

.macro toggle_bits(addr, mask)
 lda addr
 eor #mask
 sta addr
.endmacro

// ---- Wörter und Zeiger (niedriges Byte zuerst, wie beim 6502)
// 16-Bit-Konstante an addr, addr+1 schreiben (verändert A)
.macro ld16(addr, value)
 lda #<(value)
 sta addr
 lda #>(value)
 sta addr+1
.endmacro

// Zeiger auf eine Adresse setzen (gleich wie ld16, liest sich bei Nullseitenzeigern besser)
.macro ptr_set(zp, target)
 lda #<(target)
 sta zp
 lda #>(target)
 sta zp+1
.endmacro

// 16-Bit-Wert von src nach dst kopieren (verändert A)
.macro mov16(dst, src)
 lda src
 sta dst
 lda src+1
 sta dst+1
.endmacro

// A/X mit einer 16-Bit-Konstante laden (A = niedrig, X = hoch)
.macro ldax(value)
 lda #<(value)
 ldx #>(value)
.endmacro

// ---- Bytes (dst zuerst, wie bei mov16; src darf auch ein Wert mit # sein, z. B. mov8 $d020, #0)
// Byte von src nach dst kopieren (verändert A)
.macro mov8(dst, src)
 lda src
 sta dst
.endmacro

// wie mov8 über X bzw. Y (A bleibt erhalten)
.macro movx8(dst, src)
 ldx src
 stx dst
.endmacro

.macro movy8(dst, src)
 ldy src
 sty dst
.endmacro

// dst = dst + src bzw. dst - src (verändert A und das Carry-Flag)
.macro add8(dst, src)
 clc
 lda dst
 adc src
 sta dst
.endmacro

.macro sub8(dst, src)
 sec
 lda dst
 sbc src
 sta dst
.endmacro
