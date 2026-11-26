# License notice when assembling

When you assemble an assembler program, VintageCoder reads the line `SPDX-License-Identifier:` in the header of each included source file (and, if present, the copyright line). This tells it under which licenses the source code that goes into your program is released.

```
; SPDX-License-Identifier: MIT-0
; SPDX-FileCopyrightText: 2026 Erika Muster
```

If the program contains source code under a **copyleft license** (GPL, LGPL, AGPL), a factual notice appears in the log (and as information under **Problems**):

> Note (L001): The program contains source code under GPL-3.0-only (lib/rmtplayr.asm, ...). If you pass the program on, you must comply with the terms of this license (among other things, provide the source code). With --license-report you get the list.

The notice is **not an error**; the build succeeds as usual. The well-known example is the RMT player in the Atari example `music_demo`; the example `prallball` (only MIT-0) triggers nothing.

## Report
`vcc build --license-report` lists all contributing files with license, author and source of the information (header line or folder note), also those under MIT-0, CC0 or their own license, and writes the same list to `build/LICENSES.txt`. With `--json` the report comes as JSON. The system files of the target systems appear as `data/sys/...`.

## Allowing and blocking
In the project file (`.vcprj`), the section `licenses` is for this:

```json
"licenses": { "deny": ["GPL-3.0-only", "GPL-3.0-or-later"], "allow": ["LGPL-2.1-only"] }
```

- **`deny`**: The build fails with error `L002` if a contributing source text is under one of these licenses, and no result is created. A `*` at the end works as a wildcard (`GPL-*`).
- **`allow`**: For these licenses, no notice `L001` appears. They still appear in the report.
- On the command line, `vcc build --deny-license GPL-3.0-only,GPL-3.0-or-later` blocks them for one run.

## Limits
- The result is **only as good as the header lines**. Files without `SPDX-License-Identifier:` stay neutral (they are missing from the report and trigger nothing); a wrong header line leads to a wrong result.
- A **folder note** replaces a missing header line: a file `LICENSE` or `LICENSE.txt` in the folder of the file or above it that contains `SPDX-License-Identifier: ...` or `(SPDX: ...)` (this is how the supplied system files, templates and examples carry MIT-0).
- **Binary data** (music, image and program files that you include with `.incbin`) is not recognized, and neither are BASIC programs.
- The check applies to assembler source texts; it does not replace a check of the third-party parts of your program.

More about the license of VintageCoder and about your outputs: [License and acknowledgements](help:license).
