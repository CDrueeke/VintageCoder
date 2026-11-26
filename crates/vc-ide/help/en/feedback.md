# Send Feedback

Send errors, wishes and questions by email to **vintagecoder@drueeke.de**. VintageCoder prepares the mail so that the subject and a block of text contain the important details about your installation. You only write what happened.

## How to do it

1. Menu **Help > Send feedback ...** or the command palette (Cmd/Ctrl+K, word "Feedback"). You can also reach the dialog through **Settings > Info and update** (button **Feedback**).

![The dialog 'Feedback senden' on top of the 'Info und Update' page. It shows a notice, the 'Betreff' field with version information, the 'Text' field with technical installation details, a switch for the debug log and the buttons 'Abbrechen', 'In die Zwischenablage' and 'Mail vorbereiten'.](bild:feedback-dialog "The 'Feedback senden' dialog with subject, text and the mail button.")

2. The dialog **Send feedback** shows the **Subject** and **Text** of the mail as they will appear later in the mail program. Read them through.
3. **Prepare mail** starts your default mail program with a new message. Write the description of the error on the **empty line above the details block**: what you did, what you expected and what happened instead.
4. **Attach screenshots and files yourself** (for example the project or the log from the tab **Output**). VintageCoder does not attach any of these automatically.
5. The mail is sent only when you click **Send** in the mail program.

If you have no mail program set up (for example only webmail), use **To the clipboard** and paste the text into a new mail to the address given.

## What is in the mail

In the subject: version, commit of the build, operating system with version and architecture, and the type of installation. Example:

`VintageCoder 0.3.0 (Commit abc1234) · macOS 27.2 arm64 · Package: macOS app package`

In the text under **Details about the installation**: version, commit, system, type of installation (Windows installer, macOS app package, Linux package, source build or unknown), installation folder, data folder, configuration folder, the target system of the open project and the release selection of this installation (which editors, connections and target systems are included).

## Privacy

- **Nothing is sent over the network** and nothing is sent without your click; VintageCoder does not collect usage data.
- The mail contains **no names of projects or files** and no source code, only the target system (for example `c64`).
- Paths in your personal folder appear as `~` (instead of `/Users/<name>/...`), so your user name is not in the mail.
- You can change or shorten the text in the mail program before sending.

## Long mails

Links to the mail program are limited to about 1900 characters (Windows handles about 2000). If the text does not fit, VintageCoder first leaves out the release selection, then the folders, and says in the text what is missing. **To the clipboard** always contains everything. In this case, the dialog shows a note.

## Debug log for errors that are hard to describe

If an error is hard to describe, turn on the **debug log** (Settings > General > Troubleshooting, switch **Write a debug log to a file**, or the same switch in the dialog "Send feedback"; for `vcc` and the IDE also the environment variable `VC_DEBUG_LOG=1`). After that, VintageCoder writes to a file on your computer (`logs/vintagecoder.log` in the configuration folder, time stamps in UTC):

- header with the same details as the feedback mail,
- menu commands, messages of the log (build, start, emulator, debugger), errors and warnings,
- for a crash, the message with a backtrace.

Repeat the error, close VintageCoder and attach the file to your mail (it is in `logs/vintagecoder.log` in the configuration folder). The log is **never sent or attached automatically**. Paths in the personal folder appear as `~`, passwords and tokens are blacked out; file names and error messages of your project can appear, so look at the file before you attach it. From 5 MB on, a new file begins, and the old one stays as `vintagecoder.log.1`.

The file consists of JSON lines (`ts`, `seq`, `kind`, `msg`) and can be analyzed automatically: `vcc debuglog file.log` summarizes it (program, kinds, crashes, errors, last events), `--json` gives the same in machine-readable form.

## On the command line

`vcc feedback` prints the subject and text, `vcc feedback --open` opens the mail program, `vcc feedback --json` gives everything in machine-readable form, `vcc debuglog` analyzes a debug log. See [Command line](help:cli).
