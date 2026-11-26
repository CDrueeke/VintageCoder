# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.
# The legal texts (keys rechtlich-...) follow the approved wording; machine-translated, to be reviewed.

err-turbo-fehler = { $detail }

turboui-name-unbekannt = Turbo BASIC XL
turboui-stufe-titel = Turbo BASIC XL
turboui-datei-waehlen-titel = Choose Turbo BASIC disk
turboui-filter-diskabbild = Disk image

rechtlich-turbo-assistent = Turbo BASIC XL is not part of VintageCoder and must be provided by the user. Obtain a disk with Turbo BASIC XL from a source you are allowed to use, and select it here.
rechtlich-turbo-export-nachher = The project contains the Turbo BASIC XL disk.

turboui-projekt-titel = Project settings
turboui-projekt-ohne-turbo = This project has no special settings for Turbo BASIC XL. The card appears for projects with the BASIC dialect "Turbo BASIC XL" for the Atari.
turboui-karte-titel = Turbo BASIC XL
turboui-zustand-fehlt = Disk missing
turboui-zustand-bereit = Disk ready
turboui-zustand-datei-fehlt = The disk { $disk } is registered, but the file is missing.
turboui-zustand-name = { $name }, version { $version }
turboui-zustand-name-ohne-version = { $name }
turboui-zustand-frei = { $count } sectors free (about { $kb } KB)
turboui-zustand-reduziert-ja = Reduced to the system files
turboui-zustand-reduziert-nein = Not reduced (all files of the disk are still there)
turboui-zustand-veraendert = The copy has changed since it was provided (different checksum).
turboui-zustand-nicht-turbo = The copy does not look like Turbo BASIC XL (you are using it at your own risk).
turboui-zustand-kopie = Copy in the project: { $disk }
turboui-knopf-bereitstellen = Provide ...
turboui-knopf-ersetzen = Replace ...
turboui-knopf-pruefen = Check
turboui-knopf-verkleinern = Reduce ...
turboui-knopf-disk-editor = Open in the Disk Editor
turboui-feld-programm = Program name on the disk
turboui-feld-programm-hinweis = Turbo BASIC XL starts only AUTORUN.BAS by itself. Any other name does not start automatically.
turboui-feld-programm-fehler = "{ $name }" is not a valid name (1-8 characters A-Z, 0-9, _, first one a letter, extension up to 3 characters).
turboui-zusatz-titel = Additional files on the disk
turboui-zusatz-hinweis = One file per line as source or source:NAME (for example data/hello.dat:HELLO.DAT). The source is in the project, the name is an 8.3 name on the disk.
turboui-zusatz-leer = No additional files.
turboui-zusatz-neu = New line (source or source:NAME)
turboui-zusatz-hinzufuegen = Add
turboui-zusatz-entfernen = Remove: { $zeile }
turboui-zusatz-fehlt = The source is missing in the project: { $zeile }
turboui-speichern = Save
turboui-gespeichert = Turbo BASIC XL settings saved.
turboui-pruefung-titel = Result of the check
turboui-dateien-titel = Files on the disk ({ $count })
turboui-datei-zeile = { $name }, { $sectors } sectors
turboui-datei-gesperrt = write-protected
turboui-hinweis-kein-lieferumfang = Turbo BASIC XL is not part of VintageCoder and must be provided by the user. The disk stays on your computer and is not transmitted anywhere.
turboui-hinweis-gitignore = The file turbo/*.atr is listed in the project's .gitignore and does not belong in version control.
turboui-debuggen-hinweis = Debugging: Turbo BASIC programs cannot be run step by step yet.
turboui-verkleinern-fertig = Reduced: { $count } files removed, { $free } sectors free now.
turboui-geprueft = Disk checked: { $text }

turboui-dlg-titel = Provide Turbo BASIC disk
turboui-dlg-ersetzen-titel = Replace Turbo BASIC disk
turboui-dlg-schritt-datei = 1 · Choose file
turboui-dlg-schritt-pruefen = 2 · Check
turboui-dlg-schritt-kopieren = 3 · Copy
turboui-dlg-datei-waehlen = Choose file ...
turboui-dlg-keine-datei = No file chosen yet.
turboui-dlg-quelle = Chosen: { $path }
turboui-dlg-pruefung-laeuft = Checking ...
turboui-dlg-befund-erkannt = { $name } recognized. { $count } sectors free (about { $kb } KB). The disk is copied into the project ({ $dest }); your original stays unchanged.
turboui-dlg-befund-nicht-erkannt = This does not look like Turbo BASIC XL (no AUTORUN.SYS with the signature).
turboui-dlg-befund-kein-dos2 = There is no DOS 2 file system on the disk.
turboui-dlg-befund-fehler = Error in the disk: { $m }
turboui-dlg-trotzdem-frage = This does not look like Turbo BASIC XL. Use it anyway?
turboui-dlg-trotzdem-ja = Yes, use it anyway
turboui-dlg-trotzdem-nein = No, choose another file
turboui-dlg-kopie-existiert = The copy { $dest } already exists in the project. It will be replaced.
turboui-dlg-aktuelle-kopie = Current copy in the project: { $disk }. It will be replaced by the new one.
turboui-dlg-verkleinern = Reduce the disk to the system files
turboui-dlg-verkleinern-text = Only in the copy in the project, { $count } files are removed (for example sample programs). Then { $free } instead of { $now } sectors are free, enough for large programs. Your original stays untouched. You can also do this later on the card.
turboui-dlg-verkleinern-bestaetigt = Yes, reduce in the project copy
turboui-dlg-verkleinern-nein = Do not reduce (the full copy stays; programs that are too large report the error only when building)
turboui-dlg-kopieren = Copy
turboui-dlg-abbrechen = Cancel
turboui-dlg-schliessen = Close
turboui-dlg-fertig = Done: { $disk } is in the project, { $free } sectors free. The .gitignore was extended.
turboui-dlg-fertig-gitignore = The .gitignore now contains turbo/*.atr.
turboui-dlg-fehler = Providing the disk failed: { $detail }

turboui-banner-text = This project needs a Turbo BASIC XL disk. It is not part of VintageCoder.
turboui-banner-bereitstellen = Provide ...
turboui-banner-schliessen = Close banner
turboui-banner-label = Note on Turbo BASIC XL

turboui-marke-diskette = Disk (user)
turboui-marke-hinweis = Do not commit: this disk is yours and not everyone may pass it on.
turboui-status = Turbo disk: { $name }, { $free } sectors free
turboui-status-fehlt = Turbo disk missing
turboui-log-bereitgestellt = Turbo disk provided: { $disk } ({ $free } sectors free)
turboui-log-fehlt = Turbo BASIC XL: disk missing, provide it in the project settings.
turboui-nur-turbo = This command belongs to projects with Turbo BASIC XL for the Atari.
turboui-disk-editor-bald = The Disk Editor is built in a later stage. The disk is at { $disk }.
turboui-dlg-quelle-ist-kopie = The chosen file is already the copy in the project ({ $disk }). Choose your original.
