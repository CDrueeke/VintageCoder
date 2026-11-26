# SPDX-License-Identifier: GPL-3.0-or-later
# ROM import, ZIP reader and entry tool (vc-roms)

romsio-import-not-recognized = not recognized
romsio-import-role-ambiguous = role unclear (matches { $candidates })
romsio-import-checksum-unknown = checksum unknown (size matches { $candidates })
romsio-import-wrong-size = wrong size (expected { $sizes } bytes)
romsio-import-or = { " or " }
romsio-import-unsupported-system = belongs to a system that VintageCoder does not support (e.g. C65)
romsio-import-in-app-bundle = target folder is inside an app bundle
romsio-import-no-rom-dir = no ROM folder known (no configuration folder)
romsio-import-zip-no-roms = archive contains no recognized ROM files
romsio-import-part-label = { $label }: { $role }
romsio-import-zip-too-large = archive too large ({ $mb } MB), not read
romsio-import-never-bundle = never write into an app bundle
romsio-import-no-target = no target
romsio-import-archive-label = archive { $name }

romsio-zip-not-zip = not a ZIP archive
romsio-zip-dir-damaged = ZIP: directory damaged
romsio-zip-too-many = ZIP: too many entries ({ $n })
romsio-zip-name-missing = ZIP: name missing
romsio-zip-data-missing = ZIP: data missing

romsio-emit-unknown-system = unknown system { $system }
romsio-emit-unknown-role = unknown role { $system }/{ $role }
romsio-emit-bad-size = { $path }: { $size } bytes, expected { $expected } for { $system }/{ $role }
romsio-emit-bad-number = number '{ $t }' is invalid
romsio-emit-set-format = --set { $spec }: expected name:offset:size:profile[:text]
romsio-emit-set-beyond-end = --set { $spec }: lies beyond the end of the file ({ $len } bytes)
