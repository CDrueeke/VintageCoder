# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

diag-ide-b114-file-not-found = #asm: file '{ $name }' not found
diag-ide-b114-read-error = { $path }: { $error }
diag-ide-b114-no-code = #asm '{ $name }' produces no code
diag-ide-b114-org-mismatch = code '{ $name }' must start with `.org asm_base` (starts at ${ $start }, expected ${ $expected })
diag-ide-b114-overlap = #asm '{ $name }': overlapping segments
diag-ide-b114-program-too-large = Program including machine code is too large (ends at ${ $end })
diag-ide-b114-address-unstable = The address of the machine code does not settle
diag-ide-b114-code-too-large = Machine code is too large ({ $bytes } bytes)
diag-ide-e330-target-unsupported = Target '{ $target }' is not supported yet (available: { $available })
diag-ide-e330-wrong-extension = Only .bas, .lst, or .tur files (native BASIC) can be converted
diag-ide-e330-io = { $path }: { $error }
diag-ide-e330-output-extension = Output file '{ $file }' must end in .asm
diag-ide-e330-not-generated = '{ $file }' exists and was not created by basic2asm – it will not be overwritten (choose another name)
