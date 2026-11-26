# Release selection (for developers)

This page is for everyone who puts together a release from the source tree. If you only use VintageCoder, you do not need it; for that reason it appears only in the source build (without `data/release.json`), not in a release. A release does not contain everything the source tree can do: the project owner chooses in a single file which parts are shipped. A script checks the selection, assembles the files, and the program hides at run time what was not chosen.

## The selection: `FEATURES.md`

`FEATURES.md` in the main folder is the **only place with check marks**. It has five fixed sections, each with a table:

| Section | Content | IDs |
|---|---|---|
| 1. Program | the development environment `vintagecoder` and the update check (a note about new versions, without downloading and installing; without it the package never makes such a connection) | `IDE-APP`, `IDE-UPDATE` |
| 2. Editors | editors for graphics, sound and media (only effective with `IDE-APP`) | `STU-SPRITE`, `STU-CHAR`, `STU-SCREEN`, `STU-TILES`, `STU-PALETTE`, `STU-TRACKER`, `STU-PLAY`, `STU-DISK`, `STU-MEMMAP`, `STU-EXTRACT` |
| 3. Connections | emulators and real devices | `EMU-VICE`, `EMU-ATARI800`, `EMU-ALTIRRA`, `EMU-FUJISAN`, `EMU-XEMU`, `HW-ULTIMATE`, `HW-MEGA65` |
| 4. Target systems | C64, Atari 8 bit, MEGA65, Plus/4, C16, C128, VIC-20 (plus one row for a system that is not chosen at the moment) | `SYS-C64`, `SYS-ATARI`, `SYS-MEGA65`, `SYS-PLUS4`, `SYS-C16`, `SYS-C128`, `SYS-VC20`, `SYS-PET` |
| 5. Example projects | packages of examples | `EX-HELLO`, `EX-JUMPRUN`, `EX-PRALLBALL`, `EX-TROXX`, `EX-DEMOS` |

Each row has four columns: **ID** (in backticks), description, **Stand** (the status word in FEATURES.md, in German: Stabil, Beta, In Arbeit, Spike, Konzept) and **Release**. A row is checked if the last field contains `[x]`; `[X]`, `[ x]`, `[x ]` and `[ X]` count as well, `[ ]` and an empty field do not. The script does not read text outside the tables and HTML comments. Each ID must appear exactly once in `FEATURES.md` and exactly once in the registry.

Always included, without a check mark: the command line `vcc` with assembler, BASIC, build and test, and the language server `vc-lsp`. The status of these fixed parts, the release criteria and the known gaps are in `STATUS.md`. The operating system is not a choice: a selection applies to Windows, macOS and Linux, only the packaging differs.

## Registry and `release.json`

Three files belong together:

1. **`FEATURES.md`** (repository): what is chosen.
2. **`release/registry.json`** (repository, not shipped): the technical details for each ID. It names the kind (`studio`, `link`, `target`, `examples`), the short ID for the program (`sprite`, `vice`, `atari8` ...), the dependencies (`needs`: all required, `needs_any`: at least one), soft hints (`soft`), the files that belong to the row (`files`, with `**` and `{a,b}`), the help topics (`help`) and the lists `core` (always included) and `never` (never included, for example `data/sys/vcs2600/**`).
3. **`data/release.json`** (only in the package, created by the script): the chosen allow list that the program reads at start.

```json
{ "format": 1, "ide": true,
  "studios": ["sprite", "char", "screen", "tiles", "palette", "tracker", "play", "disk", "memmap"],
  "links": ["vice", "atari800", "altirra", "fujisan", "xemu", "ultimate"],
  "targets": ["c64", "atari8", "mega65", "plus4", "c16", "c128", "vic20"],
  "examples": ["EX-HELLO", "EX-JUMPRUN", "EX-PRALLBALL", "EX-TROXX", "EX-DEMOS"] }
```

**Rules when reading** (module `vc_ide::features`, used the same way by `vintagecoder` and `vcc`):

- The file is missing: **everything is on**. This is how the source tree, `cargo run` and `cargo test` behave.
- A key is missing: everything is on for it. A key is present: only the listed entries. New editors in an old `release.json` therefore stay off.
- If the file is broken or its `format` is newer than the program knows, the **minimum selection** applies (nothing selectable is on) with a warning, never "everything on".
- `VC_RELEASE=all` forces "everything on", `VC_RELEASE=<path>` loads another `release.json`. This is the way to try out a release in the source tree without packaging.

```bash
VC_RELEASE=build/stage-linux/data/release.json cargo run -p vc-gui
VC_RELEASE=all vcc help                         # everything on, also in the package
```

## The script `tools/release.py`

Python standard library only, run from the main folder (the working folder does not matter otherwise).

| Subcommand | Purpose |
|---|---|
| `check` | Check `FEATURES.md` against the registry and the file system: IDs, status, dependencies, file assignment, examples, `#include` closure. Exit code 1 on errors. |
| `plan` | Show what would be shipped (rows, files, sizes). Writes nothing. |
| `stage` | Create a staging folder for an operating system that the installers only need to pack. |
| `package` | Build Linux packages (`.deb`, `.rpm`, `tar.gz`) from a Linux staging folder (`dpkg-deb`, `rpmbuild`; in the build container, see `Makefile`). |
| `bootstrap` | One-time draft of `registry.json` from the existing files (`data/targets`, `data/args`, `examples`); maintain it by hand afterwards. |

```bash
python3 tools/release.py check                           # check selection and registry
python3 tools/release.py plan -v                         # what would be shipped, with hints
python3 tools/release.py check --explain STU-TILES       # who needs the row, what depends on it
python3 tools/release.py stage --os macos --out build/stage-macos --build
python3 tools/release.py stage --os windows --out build/stage-windows --bin-dir target/release --force
python3 tools/release.py stage --os linux --out build/stage-linux --build --archive --reproducible
python3 tools/release.py package --os linux --format deb --distro debian --stage build/stage-linux --out build/pakete
python3 tools/release.py check --features tools/testdata/nur-cli-c64.md
python3 tools/release.py bootstrap --out /tmp/registry-entwurf.json
python3 tools/release.py manifest --out build/update-feed   # version list update.json for the update check (publishes nothing)
python3 tools/release.py pfadscan bin/vcc bin/vintagecoder   # check programs for build environment paths (staging does this itself during the build)
```

Important options: `--features FILE` (another selection file, for example a test selection), `--allow-unfinished` (allow chosen rows with a status below Beta, the report marks them), `--auto-deps` (add missing hard dependencies and report them, for experiments only), `--explain ID`, `-v`. For `stage`: `--os windows|macos|linux`, `--out FOLDER`, `--build` (run `cargo build --release` first, one build per operating system), `--bin-dir` (finished programs), `--force`, `--dry-run`, `--reproducible` (no timestamps, same checksums), `--archive` (Linux: additionally a `tar.gz`), `--verify-build` (build every shipped example with the staging `vcc`). The script reads the version number only from `Cargo.toml`; only the project owner assigns it.

Rows below Beta that you explicitly want to ship are listed in `release/registry.json` under `preview_ok` (currently no row). `check` and `stage` then ship them without `--allow-unfinished` and mark them as preview in the report.

**What `stage` does:** It checks the selection (it stops on errors), copies the programs (`vcc` and `vc-lsp` always, `vintagecoder` only with `IDE-APP`), `data/` according to the registry, the examples, writes `data/release.json` and the operating system part (Windows `installer.inc`, macOS folder structure, Linux an FHS tree with programs, data, man pages), runs the **prohibition scan** (no ROM images, no emulator programs, no names of other development environments in `data/` and `examples/`) and writes `release-report.md` with the selection, warnings and the **notes for the release notes**.

## Dependencies

An editor needs `IDE-APP`. `EMU-ATARI800`, `EMU-ALTIRRA` and `EMU-FUJISAN` need `SYS-ATARI`, `EMU-XEMU` and `HW-MEGA65` need `SYS-MEGA65`, `HW-ULTIMATE` needs `SYS-C64`, `EMU-VICE` needs at least one of the VICE systems. `STU-TRACKER` and `STU-PLAY` need a target with a sound chip. An example is only shipped if its **package and the target system of its folder** are chosen. Exact rules: `check --explain <ID>`.

If a **hard** dependency is violated, `check` (and with it `stage`) stops with a message like "`HW-MEGA65` needs `SYS-MEGA65`" and names what to check or uncheck. The script never changes the selection silently (exception: `--auto-deps`). **Soft** dependencies only give a warning in the report, for example "Target system without emulator or device: build only, no run". A chosen row with a status below Beta also stops the script (exception: `--allow-unfinished`).

## What unchosen parts do

**Hidden, not removed.** The code stays in the program; the user interface shows unchosen parts nowhere and cannot be led there.

- **Editors:** no menu items, no commands in the command palette, no buttons (start page, the "Kind" choice in the "New file" dialog of the file tree), no tab. The file of an unchosen editor (for example `*.sprite.json`) opens in the text editor with a note. **File formats, `vcc media` and `vcc import` always stay**; projects stay buildable. Saved window layouts with unchosen editors stay in `layout.json` but have no effect.
- **Connections:** detection, F5, debugger, `vcc test --backend`, `vcc emulator` and `vcc streams` report "not included in the package" (exit code 1). Settings show only cards of the chosen connections; entries that were already saved stay in the file. Debugger commands, the "Image" panel and the help page "Emulators" follow the chosen connections. Emulators and ROMs are never shipped.
- **Target systems:** the project wizard, start page, settings, templates and `vcc new` offer only shipped systems. A project with a missing target can be opened and edited; build and run report "Target system 'x' is not included in this release", and `vcc` ends with exit code 2. The project file is never changed. The default target is the C64 if it is shipped, otherwise the first shipped one.
- **Help:** topics that belong only to unchosen parts are missing in the list, in search and in references (a link becomes text). The assignment is in `vc_ide::help::needs` and in the `help` field of the registry; a test keeps both the same.
- **Examples:** without example packages, "Open example" on the start page is gray. References in the help to `examples/...` stay text and can point to examples that are not shipped.

## What the selection cannot do

- It **does not remove code**. Whoever changes or deletes `data/release.json` or sets `VC_RELEASE=all` gets the hidden parts back. Whoever wants to be sure not to ship unfinished work would need Cargo features; that is not planned. The purpose is a manageable offer without hints of unfinished work, not saving space.
- **Shared data files** (`data/templates/*.json`, `data/profile/profiles.json`, `data/roms/known.json`, `data/palettes/index.json`) are shipped unchanged and also name systems that are not chosen; the program ignores these entries.
- The **fixed parts** (assembler, BASIC, `basic2asm`, modules, `vcc`) cannot be unchosen. Unfinished work in them (`BAS-B2A`, `ASM-CART`, `ASM-45GS02`) appears as a note in the release report.
- **No named presets** and no separate selection per operating system. The test selections under `tools/testdata/` are only meant for checking.
- It does **not replace an installer run**: the Windows installer and the macOS package must be started by hand once per release.
- **Linux packages** (`.deb`, `.rpm`): `package` builds them from the Linux staging folder (two packages `vintagecoder-cli` and `vintagecoder`); they are built and checked in Docker containers per distribution, the status is in `STATUS.md`. `stage --os linux --archive` still delivers a `tar.gz` archive.

## Further reading

The assembly step by step with a checklist is described in `docs/release-auswahl.md`. Checking the packages: `python3 tools/ci_staging.py` (one staging folder per test selection, built examples, prohibition scan, check with the staging `vcc`). For the command line see [Command line (vcc)](help:cli), for emulators [here](help:emulators), for target systems see the [project wizard](help:wizard).
