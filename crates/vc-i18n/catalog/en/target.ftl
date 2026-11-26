# SPDX-License-Identifier: GPL-3.0-or-later
# User interface texts: vc-target (addressing modes, banking check, target system)

target-missing = Target system "{ $id }" is not included in this release

target-bank-invalid-number = invalid number "{ $s }"
target-bank-out-of-range-u16 = value { $n } is outside 0..65535
target-bank-out-of-range-u8 = value { $n } is outside 0..255
target-bank-default-preset-missing = default preset "{ $d }" is missing
target-bank-bit-out-of-range = bit { $bit } is outside 0..7

target-mode-name-imp = Implied
target-mode-name-acc = Accumulator
target-mode-name-imm = Immediate
target-mode-name-imw = Immediate, 16-bit
target-mode-name-zp = Zero page
target-mode-name-zpx = Zero page, X-indexed
target-mode-name-zpy = Zero page, Y-indexed
target-mode-name-abs = Absolute
target-mode-name-abx = Absolute, X-indexed
target-mode-name-aby = Absolute, Y-indexed
target-mode-name-ind = Indirect
target-mode-name-izx = Indirect, X pre-indexed
target-mode-name-izy = Indirect, Y post-indexed
target-mode-name-rel = Relative
target-mode-name-rl = Relative, 16-bit
target-mode-name-izz = Indirect, Z post-indexed
target-mode-name-isy = Stack-relative indirect, Y post-indexed
target-mode-name-iax = Absolute indirect, X-indexed
target-mode-name-zpr = Zero page + relative branch

target-mode-explain-imp = no operand
target-mode-explain-acc = acts on the accumulator
target-mode-explain-imm = the value is part of the instruction
target-mode-explain-imw = the 16-bit value is part of the instruction
target-mode-explain-zp = address within the first 256 bytes
target-mode-explain-zpx = zero page address plus X
target-mode-explain-zpy = zero page address plus Y
target-mode-explain-abs = full 16-bit address
target-mode-explain-abx = 16-bit address plus X
target-mode-explain-aby = 16-bit address plus Y
target-mode-explain-ind = address is stored in a pointer
target-mode-explain-izx = pointer in the zero page, X is added to the pointer address first
target-mode-explain-izy = pointer in the zero page, Y is added to the target address afterwards
target-mode-explain-rel = branch distance relative to the instruction
target-mode-explain-rl = 16-bit branch distance relative to the instruction
target-mode-explain-izz = pointer in the zero page, Z is added afterwards
target-mode-explain-isy = pointer on the stack, Y is added afterwards
target-mode-explain-iax = pointer at 16-bit address plus X
target-mode-explain-zpr = test a zero page address and branch relative

target-row-column-missing = column { $i } is missing
target-row-unknown-mode = unknown mode { $mode }
target-row-opcode-invalid = opcode is invalid
target-unknown-memory = unknown memory configuration "{ $want }" (possible: { $options })
target-json-id-missing = target.id is missing
target-json-cpu-missing = target.cpu is missing
target-chip-regs-missing = regs is missing
target-chip-reg-name-missing = reg.name is missing
target-chip-reg-addr-invalid = reg.addr is invalid
target-chip-inc-header = ; Generated from data/chips/ for target '{ $id }' - do not edit by hand.
target-bank-start-after-end = { $what }: start ${ $start } is after end ${ $end }
target-bank-duplicate-preset = preset id "{ $id }" appears twice
target-bank-preset-label = Preset { $id }

target-cbm2-entfernt = Target system "{ $id }" (CBM-II) no longer exists: it has been removed from VintageCoder. The project cannot be built; PET, C64 and the other target systems remain.
