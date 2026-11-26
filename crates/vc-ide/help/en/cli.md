# Command Line: vcc

Everything the user interface does can also be done on the command line: `vcc` translates, builds projects, creates disks and runs tests. This is useful for scripts, automatic checks and computers without a user interface. The user interface itself is called `vintagecoder`, the language server for other editors is `vc-lsp`; both know `--help` and `--version`.

## Help in the terminal

```
vcc --help                  Overview of all commands
vcc help <command>          Help for a command (also: vcc <command> --help)
vcc help <command> <form>   only one form, e.g. vcc help basic ast
vcc help --all              all commands with all options
vcc help env                Environment variables
vcc help exit-codes         Exit codes
```

For each command, the help gives the call, all arguments and options with default values and allowed values, examples and exit codes. If you make a typing mistake (`vcc buidl`), `vcc` suggests the most similar command instead of showing the whole overview. Without arguments, `vcc` shows the short overview and ends with code 2.

## Commands

### Build programs

| Command | Purpose |
|---|---|
| `vcc asm` | Assemble assembler source |
| `vcc basic` | Tokenize BASIC, read it back, print the syntax tree |
| `vcc basic2asm` | Convert a BASIC program to assembler source |
| `vcc build` | Build a project from its .vcprj |

### Projects

| Command | Purpose |
|---|---|
| `vcc add` | Add files and folders to the project |
| `vcc migrate` | Convert a project from the old file scheme to the new one |
| `vcc new` | Create a new project from a template |
| `vcc organize` | Rearrange project files according to the project structure |

### Media and data

| Command | Purpose |
|---|---|
| `vcc dis` | Disassemble a machine program |
| `vcc disk` | Create and list disk images and extract files from them |
| `vcc import` | Bring programs and files from disks, or single files, into the project |
| `vcc inc` | Print an include file with hardware constants |
| `vcc media` (also `vcc asset`) | Work with sprites, character sets, songs and other media files |
| `vcc roms` | Show, check and import ROM files |

### Test and measure

| Command | Purpose |
|---|---|
| `vcc emulator` | Show, find and install emulators portably |
| `vcc streams` | Receive the video, audio and debug streams of an Ultimate 64 |
| `vcc test` | Check tests from *.vctest and run them in an emulator or on hardware |
| `vcc turbo` | Check and provide a Turbo BASIC XL disk and start the project |

### Help and feedback

| Command | Purpose |
|---|---|
| `vcc debuglog` | Analyze the debug log of the IDE |
| `vcc feedback` | Prepare feedback by email with details about the installation |
| `vcc update` | Check whether there is a new version (display only) |

Several commands have sub-forms, for example `vcc disk d64`, `vcc roms check` or `vcc basic ast`; `vcc help <command>` lists them.

## Examples

```
vcc new spiel --target c64
vcc build examples/c64/cart_demo
vcc asm examples/c64/border/border.asm -o border.prg
vcc test examples/c64/tests_demo
vcc roms check
```

`vcc help <command>` shows the exact options of each command; it also has more examples.

## Environment variables

| Variable | Meaning |
|---|---|
| `VC_DATA` | Folder of the hardware database (data); otherwise next to the program or in the working folder. |
| `VC_ROMS` | The user's ROM folder (instead of roms/ in the configuration folder). |
| `VC_XEMU` | Path to the Xemu program (MEGA65 emulator). |
| `VC_XEMU_SD` | Folder with MEGA65.ROM for Xemu. |
| `VC_XEMU_ARGS` | Extra arguments when starting Xemu, separated by spaces. |
| `VC_STRUCTURES` | File with your own project templates (otherwise structures.json in the configuration folder). |
| `VC_UPDATE_URL` | Source of the version list for `vcc update` (folder, file or https address; otherwise the fixed release update-feed). Never changes the target of the link. |
| `VC_UPDATE_DIR` | Folder for the result and check log of the update check (otherwise updates in the configuration folder). |
| `VC_DEBUG_LOG` | Turn on the debug log: 1 (default file logs/vintagecoder.log in the configuration folder) or a file path; 0 turns it off. Applies to vintagecoder and vcc. |
| `VC_WIN` | Window size of the IDE, e.g. 1400x860 (vintagecoder only). |
| `COLUMNS` | Width for line wrapping of the help (default: 100). |
| `NO_COLOR` | No colors in the help output. |

## Exit codes

| Value | Meaning |
|---|---|
| `0` | no errors |
| `1` | error (in the source, in a check or during execution) |
| `2` | wrong usage (unknown command or unknown option) |

## Manual pages, completion, manual

- **man pages (Linux):** `vcc man <command>` writes the page in man format to stdout; ready-made pages are in `installer/linux/man`, and you install them with `installer/linux/install-man.sh`. After that, `man vcc` and `man vcc-build` work.
- **Completion:** `vcc completion powershell|bash|zsh|fish` prints the script for the given shell; it completes commands, forms and options, and for fixed lists of values also the values.
- **Manual:** `vcc help --markdown` creates the whole manual as Markdown (in the source tree `docs/cli.md`).

All these outputs come from the same descriptions under `data/cli`; tests check that the description and the options of the program match.

## See also

[Building and running](help:build) · [Tests with vcc test](help:tests) · [ROM files](help:roms) · [Disk editor](help:disks) · [Linux](help:linux)
