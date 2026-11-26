# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

cli-lang-missing-value = --lang expects de or en

cli-lang-invalid = --lang: '{ $v }' is not a supported language (de or en)

cli-warning = Warning: { $text }

cli-no-data-dir-hint = Hardware database (data/) not found; set --data-dir or VC_DATA

cli-asm-done =
    { $out_path }: { $bytes } bytes, { $symbols } symbols, { $warnings ->
        [one] 1 warning
       *[other] { $warnings } warnings
     }

cli-no-data-dir = Hardware database (data/) not found

cli-stage-needs-number = --stage expects a number

cli-style-invalid = --style expects compact or inline

cli-runtime-invalid = --runtime expects include or embed

cli-basic2asm-target-unsupported = basic2asm: target '{ $target }' is not implemented yet (available: { $available })

cli-basic2asm-check-ok = { $input }: OK (stage { $stage }, { $lines } lines)

cli-basic2asm-done = { $out_path }: stage { $stage }, { $lines } lines, modules: { $modules }

cli-basic-sbas-disabled = The cross-platform structured BASIC (.sbas) is switched off for now. Please use native BASIC (.bas/.lst: line numbers optional, jump labels @name).

cli-basic-looks-like-atari = { $input }: { $e } (looks like Atari BASIC: give --target atari8)

cli-file-error = { $input }: { $e }

cli-basic-machine-code = Machine code: { $file } at ${ $addr }, { $size } bytes

cli-label-line = @{ $n } = line { $l }

cli-basic-machine-code-data = Machine code: { $file } at ${ $addr }, { $size } bytes (as DATA lines)

cli-disk-boot-add-conflict = --boot and --add cannot be combined (the boot disk has no file system)

cli-disk-written = { $out }: { $bytes } bytes, { $free } blocks free

cli-file-error-out = { $out }: { $e }

cli-disk-blocks-free = { $blocks } blocks free

cli-disk-boot-no-filesystem = Boot disk without file system (no DOS 2 directory): { $used } sectors with content, no files

cli-disk-detected = { $file } ({ $len } bytes): recognized as { $l } [{ $status }]

cli-disk-not-detected = { $file } ({ $len } bytes): not recognized, suggestions by size:

cli-note = Note: { $text }

cli-profile-missing = --profile is missing

cli-note-w = Note: { $text }

cli-mode-invalid = --mode: hires or multicolor, not '{ $m }'

cli-media-frames =
    { $path } ({ $count ->
        [one] 1 frame
       *[other] { $count } frames
     })

cli-media-frame-imported = Frame { $frame } imported: { $colors } image colors, { $allowed } displayable{ $conflict }

cli-conflict-invalid = --conflict: rename, replace or skip

cli-dir-invalid = --dir: relative folders only, without ..

cli-no-project-file = { $path }: no .vcprj found

cli-add-over-limit = Note: { $over } more files in { $dir } were not taken (limit 5000)

cli-not-found = { $path }: not found

cli-add-copied = copied: { $r }

cli-add-replaced = replaced: { $r }

cli-add-existing = exists: { $r }

cli-add-skipped = skipped: { $r }

cli-dry-run = (dry run, nothing changed)

cli-files-taken =
    { $n ->
        [one] 1 file
       *[other] { $n } files
     } taken over

cli-organize-nothing = Nothing to organize.

cli-organize-skipped-label = SKIPPED

cli-organize-move-label = move

cli-organize-target-taken = (target exists)

cli-organize-needs-yes = Nothing changed: without a terminal please give --yes (or --dry-run to look).

cli-organize-confirm =
    Move { $count ->
        [one] 1 file
       *[other] { $count } files
     }?

cli-cancelled = Cancelled.

cli-organize-no-check-build = Note: hardware database not found, no check build

cli-organize-done =
    { $count ->
        [one] 1 file
       *[other] { $count } files
     } moved, backup: { $backup }

cli-organize-check-same =
    Check build: { $n ->
        [one] 1 result file
       *[other] { $n } result files
     } unchanged

cli-organize-check-skipped = Check build skipped: { $why }

cli-organize-check-different = WARNING: result differs: { $files }

cli-organize-undo = To undo: copy the files back from { $backup } and move the files listed in moves.txt back.

cli-charset-invalid = --charset: auto, own or none expected

cli-test-none-found = No test files (*.vctest) found under { $dir }.

cli-test-backend-invalid = --backend { $b }: vice, atari800, fujisan, xemu, u64:HOST[:PORT] or m65:INTERFACE[@BAUD]

cli-test-broken = ERROR  { $error }

cli-test-file = Test file

cli-test-trap-unavailable = Trap mode not available: the hardware provides no CPU registers and no breakpoints. Rebuild as a mailbox test ("mode": "mailbox", test.inc)

cli-test-measure-needs-u64 = Cycle measurement (mode measure) needs the Ultimate 64: vcc test --backend u64:HOST

cli-test-atari800-target = atari800 only knows the target atari8, not { $target }

cli-test-fujisan-target = Fujisan only knows the target atari8, not { $target }

cli-test-xemu-target = Xemu only knows the target mega65, not { $target }

cli-test-mega65-rom-missing = MEGA65 ROM not found

cli-test-vice-target = VICE does not know the target { $target }

cli-test-skipped-label = SKIPPED

cli-test-summary =
    { $passed } passed, { $failed } failed, { $timeouts } timed out, { $hints ->
        [one] 1 hint
       *[other] { $hints } hints
     }, { $skipped } skipped{ $strict }

cli-test-u64-address-missing = --backend u64:HOST[:PORT]: address is missing

cli-test-m65-port-missing = --backend m65:INTERFACE[@BAUD]: interface is missing

cli-test-hw-backend-invalid = --backend { $b }: u64:HOST[:PORT] or m65:INTERFACE[@BAUD] (give an address)

cli-test-hw-needs-yes = Hardware test cancelled: without a terminal please give --yes explicitly

cli-test-hw-ask = Test on this hardware? [y/N]

cli-deny-license-invalid = --deny-license expects a list of SPDX identifiers (e.g. GPL-3.0-only,GPL-3.0-or-later)

cli-json-needs-license-report = --json only works together with --license-report

cli-license-no-sources = The project has no sources (section "sources")

cli-license-ok = OK      { $f } -> { $artifact } ({ $size } bytes){ $mark }

cli-license-error = ERROR  { $f }

cli-suggest-line = { $number }  line { $line }: { $from } → { $to }{ $reg }   { $text }

cli-suggest-count =
    { $count ->
        [one] 1 suggestion
       *[other] { $count } suggestions
     }; choose with --apply 1,3 -o file (nothing was changed)

cli-apply-invalid-number = --apply: '{ $part }' is not the number of a suggestion (1 to { $max })

cli-apply-needs-output = --apply needs -o <output> (the source file is never overwritten)

cli-apply-would-overwrite = { $out }: output would overwrite the source file (choose another name with -o)

cli-apply-done =
    { $out }: { $count ->
        [one] 1 address
       *[other] { $count } addresses
     } written as hex

cli-output-would-overwrite = { $p }: output would overwrite the source file (choose another name with -o)

cli-migrate-nothing = nothing to convert

cli-migrate-pair = { $a } -> { $b }

cli-migrate-done =
    { $count ->
        [one] 1 file
       *[other] { $count } files
     } converted; build/export/ is created again at the next build

cli-debuglog-no-config-dir = vcc debuglog: no configuration folder (HOME or APPDATA is missing)

cli-feedback-link-too-long = Note: even shortened the link is longer than { $max } characters; some mail programs cut it off (--json gives the full text).

cli-feedback-mail-opened = Mail program opened; nothing is sent until you send it there.

cli-update-not-in-package = vcc update: this package contains no update check (selection IDE-UPDATE).

cli-update-checking = Checking { $source } ...

cli-update-no-check-yet = No check saved yet (vcc update will ask).

cli-update-yanked = Note: version { $installed } was withdrawn (faulty); please switch to a newer one as soon as there is one.

cli-update-available = New: version { $version } available.

cli-update-format-warning = Attention: the new version writes a newer project format; older versions can no longer read these projects afterwards.

cli-update-show-notes = Show changes: vcc update --notes

cli-update-up-to-date = Current: no newer version known (newest in the list: { $latest }).

cli-import-basic = BASIC ({ $dialect }, { $lines } lines)

cli-import-error = ERROR: { $m }

cli-import-hint = → take over: vcc import { $input } NAME ...   or   --all

cli-import-not-on-disk = "{ $n }" is not on the disk (vcc import { $input } shows the directory)

cli-emu-no-package = no automatic package for this system

cli-emu-portable-versions = portable download: { $versions }

cli-emu-found-none = no

cli-emu-found = { $label } { $path }

cli-emu-installable-no = no{ $hint }

cli-emu-version-unknown = { $name } { $v } is not in the catalog{ $available }

cli-emu-no-package-for =
    There is no automatic package for { $name } on this system.{ $hint }
    Download page: { $page }

cli-emu-install-plan =
    Emulator:  { $name } { $version }
    Address:   { $url }
    SHA-256:   { $sha }
    Size:      { $size }
    Target folder: { $dir }

cli-emu-error = Error: { $err }

cli-roms-more-files = { $pad } +{ $count } more files of the same role

cli-roms-missing = { $id }: missing { $roles }

cli-roms-others-found =
    { $n ->
        [one] 1 matching ROM file
       *[other] { $n } matching ROM files
     } found in other programs: vcc roms import --from-others (preview), copy with --yes

cli-roms-unknown-revision = (symbols and offsets may differ)

cli-roms-install-dir = ROM folder (write target):  { $dir }

cli-roms-portable-active = Portable mode:             active ({ $dir }), { $how }

cli-roms-portable-unavailable = Portable mode:             not available – { $why }

cli-roms-portable-off-hint = Portable mode:             off (enable: folder roms/ or file portable in { $dir })

cli-roms-portable-off = Portable mode:             off

cli-roms-env-set = VC_ROMS is set: replaces user and portable folder

cli-roms-searched = Places searched (in this order):

cli-roms-files =
    { $count ->
        [one] 1 file
       *[other] { $count } files
     }

cli-roms-absent = does not exist

cli-roms-foreign-off = (ROMs of other programs are off; add --foreign to search them too)

cli-roms-none = No ROM files found.

cli-roms-copied = copied  { $what } { $name }{ $src } → { $dest }

cli-roms-copy-error = ERROR   { $name }: { $e }

cli-roms-would-copy = would copy { $what } { $name }{ $src } → { $dest }  { $label }

cli-roms-skipped = skipped { $name }: { $why }

cli-roms-nothing-copied = Nothing copied. Run with --yes.

cli-roms-needs-file = { $a } belongs after --file

cli-roms-as-missing = { $file }: --as system/role is missing

cli-file-error-f = { $f }: { $e }

cli-roms-table-written =
    { $count ->
        [one] 1 entry
       *[other] { $count } entries
     } in { $f }

cli-emu-label-device = Device:

cli-emu-device-yes = yes, not a program; Settings ▸ Target systems ▸ Hardware

cli-emu-label-systems = Systems:

cli-emu-label-role = Role:

cli-emu-device-list = Device (not a program)

cli-emu-label-homepage = Homepage:

cli-emu-label-license = License:

cli-emu-label-found = Found:

cli-emu-label-auto = Automatic:

cli-emu-label-installable = Installable:

cli-emu-installable-yes = { $version } ({ $size }){ $default }  vcc emulator install { $id }@{ $version }

cli-emu-default-mark = [default]

cli-emu-available-list = (available: { $versions })

cli-emu-program = Program: { $path }

cli-emu-unknown = vcc emulator: unknown emulator "{ $id }" (known: { $known })

cli-roms-expected = (expected { $size } bytes)

cli-roms-unknown-system = unknown system { $id }

cli-roms-path-usage = vcc roms path <system/role>, e.g. c64/kernal

cli-roms-unknown = (unknown)

cli-roms-user-dir = User folder:                { $dir }

cli-roms-portable-by-marker = by marker file

cli-roms-portable-by-folder = folder exists

cli-roms-cache-dir = Cache:                      { $dir }

cli-roms-unknown-revision-label = (unknown revision)

cli-roms-already = exists { $what } { $name }  (= { $path })

cli-media-conflict-suffix = – COLOR CLASH, reduced to the nearest colors

cli-test-tag-ok = OK

cli-test-tag-error = FAIL

cli-test-tag-hint = HINT

cli-test-tag-timeout = TIME

cli-test-hw-warning =
    WARNING: The tests run on real hardware: { $label }.
    { "  " }The running state of the device is lost (reset, load program, write memory); each test is cancelled after its time limit, then the device keeps running.

cli-test-hw-cancelled = Hardware test cancelled

cli-license-main-mark = [start program]

cli-feedback-mail-failed = vcc feedback: mail program not started ({ $e })

cli-help-data-invalid = Help: data/cli/{ $name }.json is invalid: { $e }

cli-help-usage-label = Usage:

cli-help-overview-usage = vcc <command> [options]

cli-help-overview-hint = Help for a command: vcc help <command>  or  vcc <command> --help

cli-help-general = General

cli-help-row-help = Help for a command or topic (env, exit-codes)

cli-help-row-man = Print the man page of a command (roff)

cli-help-row-completion = Print completion for powershell, bash, zsh or fish

cli-help-row-version = Show the version

cli-help-row-license = Show the license notice (details: LICENSING.md)

cli-help-row-lang = Language de or en (the environment variable VC_LANG overrides it)

cli-help-examples = Examples

cli-help-more = More detail: vcc help --all   ·   Environment variables: vcc help env   ·   Exit codes: vcc help exit-codes

cli-help-values = Values: { $values }.

cli-help-default = (default: { $value })

cli-help-repeatable = Can be repeated.

cli-help-required = Required.

cli-help-form-base = Basic form

cli-help-form-named = Form "{ $id }"

cli-help-arguments = Arguments

cli-help-optional = (optional)

cli-help-options = Options

cli-help-more-options = More options

cli-help-exit = Exit codes:

cli-help-see-also = See also:

cli-help-aliases = Also as: { $names }

cli-help-env = Environment variables

cli-help-exit-codes = Exit codes

cli-usage-incomplete = vcc { $cmd }: incomplete or invalid call

cli-usage-help-hint = Help:    vcc { $cmd } --help

cli-unknown-command = vcc: unknown command "{ $word }"

cli-did-you-mean = Did you mean { $name }?

cli-all-commands-hint = All commands: vcc --help

cli-unknown-option = vcc { $cmd }: unknown option "{ $opt }"

cli-completion-header-bash = # Completion for vcc (bash). Once: eval "$(vcc completion bash)"

cli-completion-header-powershell = # Completion for vcc (PowerShell). Once: vcc completion powershell | Out-String | Invoke-Expression

cli-completion-header-zsh = # Completion for vcc (zsh). Put the file as _vcc in a folder of the fpath or: eval "$(vcc completion zsh)"

cli-completion-header-fish = # Completion for vcc (fish). Once: vcc completion fish | source

cli-completion-desc-help = Help for a command

cli-completion-desc-man = Print man page

cli-completion-desc-completion = Print completion

cli-completion-desc-version = Show version

cli-completion-zsh-command = command

cli-completion-zsh-file = file

cli-completion-fish-form = Form

cli-completion-help =
    vcc completion - print completion (Tab) for the shell

    Usage:  vcc completion <shell>
            shell: powershell | bash | zsh | fish

    Examples:
      vcc completion powershell | Out-String | Invoke-Expression
      eval "$(vcc completion bash)"
      vcc completion fish | source

cli-completion-unknown-shell =
    vcc completion: unknown shell "{ $shell }"
    Allowed: powershell, bash, zsh, fish

cli-completion-shell-missing =
    vcc completion: shell is missing
    Usage:  vcc completion <powershell|bash|zsh|fish>

cli-man-generated-comment = .\" Generated by "vcc man"; do not edit by hand (source: data/cli/*.json).

cli-man-manual = VintageCoder Manual

cli-man-default = Default: { $value }.

cli-man-form-named = Form { $id }

cli-man-sh-examples = EXAMPLES

cli-man-sh-synopsis = SYNOPSIS

cli-man-sh-description = DESCRIPTION

cli-man-sh-options = OPTIONS

cli-man-sh-notes = NOTES

cli-man-sh-exit = EXIT STATUS

cli-man-sh-see-also = SEE ALSO

cli-man-sh-commands = COMMANDS

cli-man-sh-environment = ENVIRONMENT

cli-man-aliases = Also as: { $names }.

cli-man-arg-command = command

cli-man-arg-options = options

cli-man-command-line = { $summary } (see \fBvcc\-{ $name }\fR(1))

cli-man-general-help = Help for a command or topic (env, exit-codes).

cli-man-general-man = Print the man page.

cli-man-general-completion = Print completion for powershell, bash, zsh or fish.

cli-man-env-data = Folder of the hardware database (data).

cli-man-env-roms = ROM folder of the user.

cli-man-env-xemu = Path to the Xemu program.

cli-man-env-xemu-sd = Folder with MEGA65.ROM for Xemu.

cli-man-env-win = Window size, e.g. 1400x860.

cli-man-opt-help = Show help.

cli-man-opt-version = Show version.

cli-man-opt-version-short = Show version

cli-man-ide-summary = graphical development environment for retro systems

cli-man-ide-synopsis = vintagecoder [project-folder | file.vcprj | file.asm | file.bas]

cli-man-ide-description = Opens the IDE. Without an argument the start screen appears, with a path the project or the file.

cli-man-ide-example-start = Start screen

cli-man-ide-example-open-project = Open a project

cli-man-ide-example-open-source = Open a source file

cli-man-ide-exit = 0 after a normal exit.

cli-man-lsp-summary = language server for VintageCoder assembler

cli-man-lsp-description-1 = Speaks the Language Server Protocol over standard input and standard output. It is normally started by the editor, not by hand: without input it waits for messages and prints nothing.

cli-man-lsp-description-2 = Set it up in the editor as a server with the command vc-lsp without arguments for files with the extension .asm.

cli-man-lsp-exit = 0 after an orderly exit (shutdown, exit), otherwise 1; 2 for an unknown option.

cli-man-all-missing-dir =
    vcc man --all: target folder is missing
    Usage:  vcc man --all <folder>

cli-man-all-written = { $count } pages written to { $dir }.

cli-man-help =
    vcc man - print a man page (roff)

    Usage:  vcc man [command]
            vcc man --all <folder>

    Examples:
      vcc man asm | man -l -
      vcc man --all installer/linux/man

cli-md-form-named = Form `{ $id }`

cli-md-table-arguments =
    | Argument | Meaning |
    |---|---|

cli-md-table-options =
    | Option | Meaning | Default |
    |---|---|---|

cli-md-examples = Examples:

cli-md-note = > **Note:** { $text }

cli-md-exit = Exit codes: { $codes }

cli-md-aliases = Also as: { $names }.

cli-md-generated-comment = <!-- Generated by "vcc help --markdown" from data/cli/*.json; do not edit by hand. -->

cli-md-title = # Command line: vcc

cli-md-help-line = Help in the program: `vcc --help`, `vcc help <command>`, `vcc <command> --help`. Manual pages: `vcc man <command>`; completion: `vcc completion <shell>`.

cli-md-env-table =
    ## Environment variables

    | Variable | Meaning |
    |---|---|

cli-md-exit-table =
    ## Exit codes

    | Value | Meaning |
    |---|---|

cli-test-default-name = Test { $n }

cli-test-default-measure-name = Measurement { $n }

cli-dis-segment = ; segment { $n }: ${ $addr }-${ $end } ({ $size } bytes)

cli-dis-undocumented = ; undocumented

cli-inc-written = { $o } written

cli-file-error-o = { $o }: { $e }

cli-basic-roundtrip-ok = { $input }: round trip OK

cli-disk-problem = Problem: { $m }

cli-disk-entry = { $name } { $sectors } sectors  from { $start }{ $locked }

cli-disk-free-sectors = { $count } free sectors

cli-media-color-replaced = Color { $color } ({ $pixels } pixels) replaced

cli-organize-adjust-paths = Adjusting paths: { $f } ({ $n })

cli-structure-heading = Project structures (--structure):

cli-test-junit-written = JUnit XML: { $j }

cli-file-error-j = { $j }: { $e }

cli-test-strict-mark = (--strict)

cli-build-disk-error = Disk: { $e }

cli-license-report-written = License report: build/LICENSES.txt

cli-license-report-error = build/LICENSES.txt: { $e }

cli-migrate-long-names = Long names → short names:

cli-feedback-mail-preview =
    To: { $to }
    Subject: { $subject }
    { $body }

cli-update-release-page = Release page: { $url }

cli-secs-needs-number = --secs: number expected

cli-streams-listen-usage = vcc streams listen --host HOST SONG.vcsong

cli-file-error-file = { $file }: { $e }

cli-streams-player-size = Player program: { $size } bytes

cli-streams-error = ERROR { $e }

cli-streams-buffer = Buffer: fill level { $fill } ms, { $filled } gaps filled, { $underruns } underruns, { $overruns } overruns

cli-streams-wav = WAV: { $seconds } s → { $path }

cli-streams-stopped = Stream stopped{ $suffix }.

cli-streams-usage = vcc streams probe|trace --host HOST [--video|--audio|--debug] [--secs N] [--password PW] [--multicast] [--record FOLDER [--count N]]

cli-count-needs-number = --count: number expected

cli-host-missing = --host is missing

cli-streams-video-debug-conflict = Video and debug stream never run at the same time (--video or --debug)

cli-streams-target = { $label }: target { $target }

cli-streams-kind-error = { $label }: ERROR { $e }

cli-dir-error = { $dir }: { $e }

cli-streams-recorded = { $n } packets → { $path }

cli-streams-all-stopped = Streams stopped.

cli-streams-shot-usage = vcc streams shot --host HOST --out FILE.png

cli-address-invalid = Address "{ $s }" is invalid

cli-address-unknown = Address or symbol "{ $s }" unknown (symbols only with --debug file.vcdbg)

cli-address-out-of-range = Address "{ $s }" is outside $0000..$FFFF

cli-cpu-needs-id = --cpu: ID expected

cli-measure-needs-pair = --measure: A:B expected

cli-tail-needs-number = --tail: number expected

cli-no-data-dir-short = Data folder not found (--data-dir)

cli-trace-45gs02 = The 45GS02 is not analyzed (6502/6510 only).

cli-measure-invalid = --measure { $m }: form A:B expected

cli-trace-failed = Trace: { $e }

cli-trace-started = Debug stream started (video stream stopped), { $secs } s ...

cli-trace-heading = Trace (6510 only), { $secs } s

cli-trace-cycles = Cycles: { $cycles } (of which { $stalls } BA stalls), measured { $rate } M/s; instructions: { $insns } (IRQ { $irqs }, NMI { $nmis }), reset vector read: { $resets }

cli-trace-sync = Instruction detection: { $state }; contradictions { $desyncs }, entry points { $syncs }

cli-trace-packets = Packets: { $packets }, losses: { $lost } ({ $percent } %), restarts after loss: { $epochs }

cli-trace-coverage = Coverage: { $covered } addresses with opcode fetch; SID/VIC writes: { $io_writes }

cli-trace-unreached = Lines not reached: { $count }{ $hint }

cli-trace-last-instructions = Last { $count } instructions (cycle, PC, cycles, source):

cli-trace-recorded = { $n } instructions → { $path }

cli-trace-dropped = WARNING: { $count } instructions not recorded (disk too slow)

cli-stream-stats = { $label }: { $state }  { $packets } packets ({ $rate }/s, { $mbit } Mbit/s), { $lost } missing ({ $percent } %), { $rejected } rejected, { $restarts } restarts

cli-stream-audio-stats = Audio: { $state }  { $packets } packets, { $lost } missing ({ $percent } %)

cli-stream-ultimate-reset = Ultimate reset

cli-trace-synced = synchronized

cli-trace-not-synced = not synchronized

cli-trace-examples-below = (examples below)

cli-error-text = { $text }

cli-license-report-title = Licenses of the assembled source code (generated by VintageCoder)

cli-license-report-how = The lines SPDX-License-Identifier in the header lines of the files are read (otherwise a folder notice LICENSE).

cli-license-report-caveat = This is only as good as these header lines; binary data and files without an identifier are not detected. Not a legal opinion.


# I3b
cli-define-expected = -D expects NAME=number
cli-category-unknown = --category: unknown ({ $choices })
cli-structure-unknown = project structure '{ $id }' is unknown (vcc new --list)

cli-convert-range-expected = --range expects a range of old line numbers, e.g. 100-200

cli-convert-summary = { $changed } lines changed, { $new } labels new, { $kept } labels unchanged

cli-convert-new-label = new label @{ $name } ({ $file }, line { $line })

cli-convert-preview-line = { $file }:{ $line }: { $old } → { $new }

cli-convert-written = { $file }: written

cli-convert-includes-hint = { $count } included file(s) change as well; use --write to write all files, --preview to see all changes
cli-migrate-includes-change = { $file }:{ $line }: { $old }  →  { $new }
cli-migrate-includes-removed = { $file }:{ $line }: { $old }  →  removed (now part of the media file include)
cli-migrate-includes-skipped = { $file }:{ $line }: { $old }  –  { $why }
cli-migrate-includes-done = { $count } line(s) converted, { $skipped } left unchanged.
cli-migrate-includes-dry = { $count } line(s) would be converted, { $skipped } would stay unchanged. Use --apply to convert.
