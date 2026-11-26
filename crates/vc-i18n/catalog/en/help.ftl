# SPDX-License-Identifier: GPL-3.0-or-later
# Help: generated pages (emulators, manual header, instructions, registers, lookup)
help-emu-arch-64bit = 64-bit
help-emu-portable-download = portable download: { $pakete }
help-emu-col-emulator = Emulator
help-emu-col-systems = Systems
help-emu-col-role = Role
help-emu-col-portable = Portable installation
help-emu-homepage = Homepage
help-emu-download-page = Download page
help-emu-license = License
help-emu-role-in-vc = Role in VintageCoder
help-chapter-meta = Level: { $stufe } · Time: { $zeit }
help-chapter-see-also = See also in the manual:
help-chapter-reference = For reference:
help-mn-quad = 45GS02 Q instruction: prefix `NEG NEG` (`$42 $42`) before the opcode of `{ $base }`; it works on the 32-bit register Q (A, X, Y, Z). With `[$nn],z` the prefix `$EA` is added as well (32-bit pointer).
help-mn-izz = With `[$nn],z` (prefix `$EA`) the pointer becomes 32 bits wide.
help-mn-table-head = | Addressing | Syntax | Opcode | Bytes | Cycles | Flags |
help-mn-illegal = Some forms are **undocumented** (`illegals = allow|warn` is needed in the `.cpu` header).
help-mn-cycles-flags = Cycle counts: `+1` on a page crossing, `+1/+2` when a branch is taken. Flags: N=Negative, V=Overflow, Z=Zero, C=Carry.
help-see-basic = See [BASIC](help:basic).
help-see-topic = See [{ $title }](help:{ $rel }).
help-reg-title = Register { $name }
help-reg-line = **{ $chip }.{ $name }** - address `{ $addr }`, access { $access }
help-reg-use = Use it in source code as `chip.register` (for example `lda vic2.border`).
help-none = I have no entry for "{ $word }". The search finds topics, instructions, directives and registers.
help-sym-address = Address `{ $addr }`
help-sym-same-address = Same address: { $names }
help-sym-include = Include: `#include <{ $file }>`
