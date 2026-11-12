# Parameter catalogs (`data/args/*.json`)

Catalogs describe the command-line parameters of the programs VintageCoder starts (emulators) or drives (the built-in assembler).
Settings → Targets and Settings → Toolchain show them as checkboxes and drop-downs; the same code resolves them for **Run** and **Debug**.
Key names, ids, placeholders and values in the files are English; only the display texts (`label`, `hint`, `verified`, …) are German.

## Where catalogs are read from

In the order of the rest of the database: `VC_DATA`, the `data` folder next to the program or the working directory, and last
`~/.config/vintagecoder/args/*.json` (Windows: `%APPDATA%\vintagecoder\args`). A file with the same `program` in the user folder **adds to** the
shipped one: parameters with the same `id` replace, new ones are appended. Broken files are reported and skipped.

## File format

```json
{
 "program": "vice-x64sc", "label": "VICE x64sc (C64)", "kinds": ["x64sc"],
 "verified": "where the entries were checked", "strict": false,
 "groups": [{"id": "video", "label": "Bild"}],
 "reserved": ["-autostart"], "debug_reserved": ["-binarymonitoraddress <value>"],
 "params": [
  {"id": "warp", "group": "run", "type": "switch", "flag": "-warp", "off_flag": "+warp", "label": "Warp-Modus beim Start", "hint": "…", "conflicts": ["speed"]},
  {"id": "filter", "group": "video", "type": "choice", "flag": "-VICIIfilter", "choices": [{"value": "1", "label": "CRT-Emulation"}]},
  {"id": "kernel", "group": "system", "type": "choice", "flag": "/kernel", "join": ":", "choices": [{"value": "xl", "label": "XL/XE"}]}
 ]}
```

- `type`: `switch`, `choice`, `text`, `number` (with `min`/`max`), `file`.
- `flag` is the switch; `off_flag` the opposite one (`+warp`); `join` puts the value in the same word (`/kernel:xl`), otherwise it is the next argument.
- `conflicts` / `requires` name other parameter ids; `default` and `example` are shown in the hover text.
- `reserved`: switches VintageCoder sets itself when starting; a free-field entry gets a warning.
  `debug_reserved`: switches the debug start needs for its connection; they are dropped when debugging (`<value>` marks a switch with a value).
- `strict`: unknown switches in the free field are errors (for programs VintageCoder evaluates itself, e.g. `vcc-asm`).
- `kinds` are the emulator kinds (`x64sc`, `atari800`, `xemu`, …) the catalog applies to.

## Placeholders

`{project}`, `{project_dir}`, `{output}`, `{target}`, `{variant}`, `{start}`, `{symbols}`, `{disk}` can be used in values and in the free field
(`{{` and `}}` give literal braces). Unknown or unavailable ones (for example `{start}` for a program without BASIC stub) are errors.
The settings page previews the command line with a sample project.

| Placeholder | Value |
|---|---|
| `{project}` | project name (folder name) |
| `{project_dir}` | absolute project folder |
| `{output}` | built program that is started |
| `{target}` | target system id (`c64`, `c128`, `plus4`, `vic20`, `atari8`, `mega65`) |
| `{variant}` | `pal` or `ntsc` |
| `{start}` | entry address from the BASIC stub (`SYS`), four hex digits |
| `{symbols}` | label file for the monitor (VICE: `.vs` next to the output), after a build |
| `{disk}` | the project's disk image, if it has one |

`tools/gen_args_catalog.py` generates the shipped files, `tools/check_args_catalog.py` compares them with the programs' help texts.
