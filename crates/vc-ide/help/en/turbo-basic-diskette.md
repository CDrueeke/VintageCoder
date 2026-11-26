# Turbo BASIC XL: disk

Turbo BASIC XL is a faster, extended Atari BASIC. On the Atari it is loaded from a **disk**. A Turbo BASIC XL project in VintageCoder therefore needs a disk that **you provide yourself**.

Turbo BASIC XL is not part of VintageCoder and must be provided by the user. Select a disk from a source you may use.

VintageCoder sends no data: there is **no telemetry**, the disk is not transmitted anywhere and stays only on your computer.

## How it works

1. The project gets a **copy** of your disk in the folder `turbo/` (under the original name, for example `turbo/Turbo_Basic_XL.atr`). Your original stays unchanged.
2. When **building**, this copy becomes `build/<name>.atr`: your translated program is written into it as `AUTORUN.BAS`. Turbo BASIC XL loads `AUTORUN.BAS` by itself after starting.
3. **Run** (F5) boots this disk in the Atari emulator (atari800), without the built-in BASIC. At the end of the program `READY` appears: Turbo BASIC XL then waits for input.

## Providing the disk

You can do this in three places: in the area **Project** on the page **Disk** through the card "Turbo BASIC XL" (button **Provide ...**, with an existing copy **Replace ...**), through the banner "This project needs a Turbo BASIC XL disk" that appears above the editor when you run without a disk (button **Provide ...**), and through the command palette ("Provide Turbo BASIC XL disk ..."). The card appears only for projects with the BASIC dialect "Turbo BASIC XL" for the Atari. On the command line: `vcc turbo provide`; there are also `vcc turbo check` (check), `vcc turbo reduce` (reduce) and `vcc turbo run` (build and run).

*[Bild folgt in E13: turbo-karte-diskette]*

The assistant "Provide Turbo BASIC XL disk" has three steps (choose file, check, copy). It checks the file (ATR, DOS 2, `AUTORUN.SYS` with the signature "TURBO-BASIC XL", version, free sectors) and shows the findings. If the file does not look like Turbo BASIC XL, it asks "Use it anyway?". Then you use it at your own risk. Images of 90 KB (720 sectors of 128 bytes) are supported; other sizes are reported as "not supported yet".

### Space and "Reduce"

On the original disk only about 103 sectors (about 12.9 KB) are free. The assistant therefore offers to reduce the **copy in the project** to the three system files `DOS.SYS`, `DUP.SYS` and `AUTORUN.SYS`. Then about 482 sectors (about 60 KB) are free. This happens only after your confirmation and only in the project copy. Sample programs as well as `COMPILER.COM`, `RUNTIME.COM` and `RAMDISK.COM` are removed. If you decline, the full copy stays; programs that are too large then report when building: "The program needs ... sectors, the disk has ... free."

## Settings in the project

The settings are stored in the field `turbo` of the project file (`.vcprj`):

| Field | Meaning |
|---|---|
| `disk` | Path of the copy in the project |
| `sha256` | Checksum of the copy when provided (detects a changed file) |
| `program` | Name of the program on the disk (default `AUTORUN.BAS`; only this name starts by itself) |
| `files` | Additional files on the disk: `source` or `source:NAME` (8.3 name, capital letters) |
| `reduced` | The copy was reduced |

You edit the same settings on the card "Turbo BASIC XL": the field "Program name on the disk" and the list "Additional files on the disk" (one file per line as `source` or `source:NAME`), then **Save**. The card also shows the state ("Disk ready" or "Disk missing"), the free sectors and the files on the disk, and offers **Check**, **Reduce ...** and **Open in the Disk Editor**; these commands are also in the command palette.

## Passing it on

The file `turbo/*.atr` is listed in the project's `.gitignore` and does not belong in version control. Whoever receives your project provides their own disk. When you copy a project that contains a Turbo disk, this note appears after copying: "The project contains a disk that not everyone may pass on."

## Limits

- **Debugging** is not yet possible for Turbo BASIC projects ("Turbo BASIC programs cannot be run step by step yet"); the button is greyed out.
- Only atari800 and Altirra are planned as run targets (Altirra start with ATR not checked yet); Ultimate and real hardware later.
- Compiling with `COMPILER.COM` on the disk (compiler stage) is not part of this version.

## Error messages

| Message | Meaning and remedy |
|---|---|
| "This project needs a Turbo BASIC XL disk ..." | No disk is provided: card or banner > Provide .... |
| "This does not look like Turbo BASIC XL ..." | Wrong file, or another version/packaging. Choose another file or "use it anyway". |
| "The copy ... has changed since it was provided" | The checksum differs. Usually harmless (you edited the copy yourself); otherwise provide it again. |
| Program does not fit on the disk | Reduce the copy (card > Reduce ...), or make the program smaller. |

See also: [BASIC: dialects and target systems](hilfe:basic-dialects), [Build and run](hilfe:build), [ROM files](hilfe:roms).
