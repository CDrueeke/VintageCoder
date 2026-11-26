# SPDX-License-Identifier: GPL-3.0-or-later
# ROM library (vc-roms): status names, places, roles, messages

# Status of a ROM role
roms-status-recognized = Recognized
roms-status-unknown-revision = Unknown revision
roms-status-replacement = Replacement (free)
roms-status-missing = Missing
roms-status-unusable = Not usable

# Kind of search place
roms-stage-explicit = Setting
roms-stage-project = Project
roms-stage-portable = Program folder (portable)
roms-stage-user = User folder
roms-stage-legacy = Old location
roms-stage-foreign = Other program
roms-stage-bundled = Bundled (replacement)

roms-place-setting = Setting { $system }/{ $role }
roms-place-legacy-mega65 = Old location (mega65)
roms-place-xemu-appdata = Xemu (program data)
roms-place-vice-configured = VICE (configured)

# Short line for a resolved role
roms-describe = { $system }/{ $role }: { $status }
roms-describe-label = - { $label }
roms-describe-place = ({ $stage }: { $path })
roms-describe-expected = - expects { $sizes } bytes
roms-or = or

# Reasons why a file was not recognized
roms-ident-no-rom-size = not a ROM size
roms-ident-too-large = too large for a ROM image
roms-ident-unreadable = not readable
roms-ident-no-role = does not fit any role of { $h } (size { $size } bytes)

# Portable mode
roms-portable-translocation = App translocation: the app runs from a read-only random path; to enable, move it to the "Applications" folder and restart
roms-portable-bundle-no-parent = Bundle without a parent folder
roms-portable-no-program-dir = no program folder

# Table of known ROMs
roms-known-format = known.json: format { $format } is not supported
roms-known-no-uses = known.json: { $id } has no "uses"
roms-known-bad-parts = known.json: { $id } has invalid parts (offset or size)

# Roles
roms-role-kernal = KERNAL
roms-role-basic = BASIC
roms-role-chargen = Character set
roms-role-dos1541 = Drive 1541
roms-role-dos1571 = Drive 1571
roms-role-dos1581 = Drive 1581
roms-role-basiclo = BASIC (low)
roms-role-basichi = BASIC (high)
roms-role-kernal64 = KERNAL (C64 mode)
roms-role-basic64 = BASIC (C64 mode)
roms-role-functionlo = Function ROM (low)
roms-role-functionhi = Function ROM (high)
roms-role-os-xl = Operating system XL/XE
roms-role-os-b = Operating system 400/800 (B)
roms-role-os-a = Operating system 400/800 (A)
roms-role-rom = System ROM
roms-role-basic-b000 = BASIC ($B000)
roms-role-basic-c000 = BASIC ($C000)
roms-role-basic-d000 = BASIC ($D000)
roms-role-editor = Screen editor
roms-role-chip-c000 = Chip $C000 (2 KB)
roms-role-chip-c800 = Chip $C800 (2 KB)
roms-role-chip-d000 = Chip $D000 (2 KB)
roms-role-chip-d800 = Chip $D800 (2 KB)
roms-role-chip-f000 = Chip $F000 (2 KB)
roms-role-chip-f800 = Chip $F800 (2 KB)

# Systems
roms-system-c64 = C64
roms-system-c128 = C128
roms-system-vic20 = VIC-20
roms-system-plus4 = Plus/4, C16
roms-system-atari8 = Atari 8-bit
roms-system-mega65 = MEGA65
roms-system-pet = PET/CBM

# Character sets inside a ROM image
roms-set-upper = Uppercase and graphics
roms-set-lower = Upper/lower case
roms-set-vdc = 80 columns (VDC, 8x16)
roms-set-standard = Standard
roms-set-international = International
