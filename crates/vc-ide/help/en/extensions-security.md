# Extensions: security and rights

*Note: Managing extensions comes in a later version. This page describes the rules planned for it.*

Extensions bring data (for example templates) or tools (separate programs) into VintageCoder. This page says openly what VintageCoder secures and what it does not.

## Tools run with your permissions

This tool runs as a separate program with your permissions. The declared permissions are shown but not enforced. Allow it only from a source you trust.

VintageCoder cannot lock up foreign programs.

The consent dialog shows name, publisher, version, license, programs, menu commands and the declared permissions (`run-programs`, `read-project`, `write-project`, `network-intended`). **Nothing** runs without your consent.

## Extensions in a project

When you open a project with `.vintagecoder/extensions/`, a banner appears. The project opens right away; **only data extensions** are loaded, **no tools**, until you choose "Trust". The banner says:

This project brings extensions. Tools are disabled until you trust the project. [Trust] [Do not trust] [What does this mean?]

Trust is stored **per project folder, locally** (not in the project). The default is "Do not trust".

See also: [ROM files](hilfe:roms), [License](hilfe:license).
