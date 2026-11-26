# Check for updates

VintageCoder can check whether a new version is available. It then shows you **what has changed** and offers a **link to the release page**, where you download the new version yourself. **VintageCoder does not download or install anything itself.**

## Nothing happens without your consent

As long as you have not chosen anything under **Settings > General > Updates**, it says *Not asked yet* and nothing goes to the network. Your choices: *On request*, *Daily* or *Weekly* (with *Daily* and *Weekly*, VintageCoder checks at start). The choice applies after **Save**. The page **Info and update** has the button **Check for updates now**; it always sends exactly one request, without changing the setting. The field **Address of the version list** (empty = default) redirects the request to another list.

![The 'Updates' card with the text 'Noch nie nach Updates gesucht.' (never checked for updates), the button 'Jetzt nach Updates suchen' and a note that only the version list is requested and that the frequency is set under 'Allgemein'.](bild:einstellungen-update "The 'Updates' card with the button to check for updates.")

## What is sent

One **request for the version list** (`update.json`, a few kilobytes) to `github.com`. GitHub sees your **IP address**, the address of the file and the program name "VintageCoder-update", without a version number. Nothing else: no identifier, no account, no details about your computer, projects or settings, no counters. The project has no server of its own.

Every check is recorded in a check log (time, source, result). The result and the log are in the folder `updates` in the configuration folder (see [Settings](help:settings)); the result of the last check is also shown on the page **Info and update** ("Last check ..." or "Never checked for updates.").

## When a new version is available

**Update X available** appears in the status bar. A click opens **Settings > Info and update**. There, the section **Updates** shows:

- "New version X available" with the date,
- for a newer project format, a **warning**: after that, older versions can no longer read your projects,
- the **changes** since your version, as text (links and images appear only as text and cannot be clicked),
- **View changes**: opens the page of the new version in your browser. The link is fixed in the program (`github.com/CDrueeke/VintageCoder/releases/tag/v...`); an address from the version list is never used.
- **Remind me later**: the notice rests for three days.
- **Skip this version**: this number is not reported again; a newer one is reported again.

If the installed version is the newest, it says "You have the latest version."; if it was withdrawn, it asks you to update.

You update packages from your package manager (for example `.deb`, `.rpm`) with that package manager. The dialog itself never downloads or installs anything.

## If it does not work

Automatic checks fail **silently**: no network is not an event, nothing is shown, and the next attempt comes later (after an error at the earliest after six hours, then less often). Only **Check for updates now** shows a message with the cause, for example *no connection*, *Timeout* or *The version list is invalid*. Nothing on your installation is ever changed.

## Security

In this version the version list is **not signed**. It can therefore cause nothing except a display: the link comes from a fixed address in the program, the change texts are plain text, sizes and counts are limited, and nothing is downloaded or run. A forged list could at most show a wrong version number or a wrong text. Download new versions only from the release page in the repository `CDrueeke/VintageCoder`.

## On the command line

`vcc update` shows the same and ends with code 10 if a newer version is available; see [Command line](help:cli). Other `vcc` commands never ask for updates. With `VC_UPDATE_URL` you can change the source of the list (for example to a folder for testing); this never changes the target of the link.

## Packages without update check

A package can be built without the update check (selection `IDE-UPDATE`, see [Release selection](help:release-auswahl)). Then this section does not exist, and neither does such a connection.
