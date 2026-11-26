# SPDX-License-Identifier: GPL-3.0-or-later
# Kommandozeile vcc: Meldungen, Hilfe-Rahmentexte, Handbuchseiten, Vervollständigung (Etappe I2; Beschreibungen der Befehle aus data/cli folgen in I3)

# Kontext: main.rs, ok_or
cli-lang-missing-value = --lang erwartet de oder en

# Kontext: main.rs, Err
cli-lang-invalid = --lang: '{ $v }' ist keine unterstützte Sprache (de oder en)

# Kontext: main.rs, eprintln
cli-warning = Warnung: { $text }

# Kontext: main.rs, Text
cli-no-data-dir-hint = Hardware-Datenbank (data/) nicht gefunden; --data-dir oder VC_DATA setzen

# Kontext: main.rs, println
cli-asm-done = { $out_path }: { $bytes } Bytes, { $symbols } Symbole, { $warnings } Warnung(en)

# Kontext: main.rs, Text
cli-no-data-dir = Hardware-Datenbank (data/) nicht gefunden

# Kontext: main.rs, Text
cli-stage-needs-number = --stage erwartet eine Zahl

# Kontext: main.rs, Text
cli-style-invalid = --style erwartet compact oder inline

# Kontext: main.rs, Text
cli-runtime-invalid = --runtime erwartet include oder embed

# Kontext: main.rs, Text
cli-basic2asm-target-unsupported = basic2asm: Ziel '{ $target }' ist noch nicht umgesetzt (verfügbar: { $available })

# Kontext: main.rs, println
cli-basic2asm-check-ok = { $input }: in Ordnung (Stufe { $stage }, { $lines } Zeilen)

# Kontext: main.rs, println
cli-basic2asm-done = { $out_path }: Stufe { $stage }, { $lines } Zeilen, Module: { $modules }

# Kontext: main.rs, Text
cli-basic-sbas-disabled = Das plattformübergreifende strukturierte BASIC (.sbas) ist vorerst abgeschaltet. Bitte natives BASIC verwenden (.bas/.lst: Zeilennummern optional, Sprungmarken @name).

# Kontext: main.rs, Text
cli-basic-looks-like-atari = { $input }: { $e } (sieht nach Atari BASIC aus: --target atari8 angeben)

# Kontext: main.rs, Text
cli-file-error = { $input }: { $e }

# Kontext: main.rs, println
cli-basic-machine-code = Maschinencode: { $file } bei ${ $addr }, { $size } Bytes

# Kontext: main.rs, Text
cli-label-line = @{ $n } = Zeile { $l }

# Kontext: main.rs, println
cli-basic-machine-code-data = Maschinencode: { $file } bei ${ $addr }, { $size } Bytes (als DATA-Zeilen)

# Kontext: main.rs, Text
cli-disk-boot-add-conflict = --boot und --add lassen sich nicht kombinieren (die Boot-Diskette hat kein Dateisystem)

# Kontext: main.rs, println
cli-disk-written = { $out }: { $bytes } Bytes, { $free } Blöcke frei

# Kontext: main.rs, Text
cli-file-error-out = { $out }: { $e }

# Kontext: main.rs, println
cli-disk-blocks-free = { $blocks } Blöcke frei

# Kontext: main.rs, println
cli-disk-boot-no-filesystem = Boot-Diskette ohne Dateisystem (kein DOS-2-Verzeichnis): { $used } Sektoren mit Inhalt, keine Dateien

# Kontext: main.rs, eprintln
cli-disk-detected = { $file } ({ $len } Bytes): erkannt als { $l } [{ $status }]

# Kontext: main.rs, eprintln
cli-disk-not-detected = { $file } ({ $len } Bytes): nicht erkannt, Vorschläge nach der Größe:

# Kontext: main.rs, eprintln
cli-note = Hinweis: { $text }

# Kontext: main.rs, Text
cli-profile-missing = --profile fehlt

# Kontext: main.rs, eprintln
cli-note-w = Hinweis: { $text }

# Kontext: main.rs, Text
cli-mode-invalid = --mode: hires oder multicolor, nicht '{ $m }'

# Kontext: main.rs, println
cli-media-frames = { $path } ({ $count } Frames)

# Kontext: main.rs, println
cli-media-frame-imported = Frame { $frame } importiert: { $colors } Bildfarben, { $allowed } darstellbar{ $conflict }

# Kontext: main.rs, Text
cli-conflict-invalid = --conflict: rename, replace oder skip

# Kontext: main.rs, Text
cli-dir-invalid = --dir: nur relative Ordner ohne ..

# Kontext: main.rs, Text
cli-no-project-file = { $path }: keine .vcprj gefunden

# Kontext: main.rs, eprintln
cli-add-over-limit = Hinweis: { $over } weitere Datei(en) in { $dir } nicht aufgenommen (Obergrenze 5000)

# Kontext: main.rs, Text
cli-not-found = { $path }: nicht gefunden

# Kontext: main.rs, println
cli-add-copied = kopiert: { $r }

# Kontext: main.rs, println
cli-add-replaced = ersetzt: { $r }

# Kontext: main.rs, println
cli-add-existing = vorhanden: { $r }

# Kontext: main.rs, println
cli-add-skipped = übersprungen: { $r }

# Kontext: main.rs, println
cli-dry-run = (Trockenlauf, nichts geändert)

# Kontext: main.rs, println
cli-files-taken = { $n } Datei(en) übernommen

# Kontext: main.rs, println
cli-organize-nothing = Nichts zu ordnen.

# Kontext: main.rs, println
cli-organize-skipped-label = ÜBERSPRUNGEN

# Kontext: main.rs, println
cli-organize-move-label = verschieben

# Kontext: main.rs, println
cli-organize-target-taken = (Ziel belegt)

# Kontext: main.rs, eprintln
cli-organize-needs-yes = Nichts geändert: ohne Terminal bitte --yes angeben (oder --dry-run zum Ansehen).

# Kontext: main.rs, Text
cli-organize-confirm = { $count } Datei(en) verschieben?

# Kontext: main.rs, println
cli-cancelled = Abgebrochen.

# Kontext: main.rs, eprintln
cli-organize-no-check-build = Hinweis: Hardware-Datenbank nicht gefunden, kein Prüfbau

# Kontext: main.rs, println
cli-organize-done = { $count } Datei(en) verschoben, Sicherung: { $backup }

# Kontext: main.rs, println
cli-organize-check-same = Prüfbau: { $n } Ergebnisdatei(en) unverändert

# Kontext: main.rs, println
cli-organize-check-skipped = Prüfbau übersprungen: { $why }

# Kontext: main.rs, eprintln
cli-organize-check-different = ACHTUNG: Ergebnis weicht ab: { $files }

# Kontext: main.rs, eprintln
cli-organize-undo = Rückgängig: die Dateien aus { $backup } zurückkopieren und die in moves.txt genannten Dateien zurückverschieben.

# Kontext: main.rs, Text
cli-charset-invalid = --charset: auto, own oder none erwartet

# Kontext: main.rs, println
cli-test-none-found = Keine Testdateien (*.vctest) unter { $dir } gefunden.

# Kontext: main.rs, Text
cli-test-backend-invalid = --backend { $b }: vice, atari800, fujisan, xemu, u64:HOST[:PORT] oder m65:SCHNITTSTELLE[@BAUD]

# Kontext: main.rs, println
cli-test-broken = FEHLER  { $error }

# Kontext: main.rs, Text
cli-test-file = Testdatei

# Kontext: main.rs, Text
cli-test-trap-unavailable = Trap-Modus nicht verfügbar: die Hardware liefert keine CPU-Register und keine Haltepunkte. Als Postfach-Test ("mode": "mailbox", test.inc) neu bauen

# Kontext: main.rs, Text
cli-test-measure-needs-u64 = Zyklenmessung (mode measure) braucht die Ultimate 64: vcc test --backend u64:HOST

# Kontext: main.rs, Err
cli-test-atari800-target = atari800 kennt nur das Ziel atari8, nicht { $target }

# Kontext: main.rs, Err
cli-test-fujisan-target = Fujisan kennt nur das Ziel atari8, nicht { $target }

# Kontext: main.rs, Err
cli-test-xemu-target = Xemu kennt nur das Ziel mega65, nicht { $target }

# Kontext: main.rs, ok_or
cli-test-mega65-rom-missing = MEGA65-ROM nicht gefunden

# Kontext: main.rs, Err
cli-test-vice-target = VICE kennt das Ziel { $target } nicht

# Kontext: main.rs, Text
cli-test-skipped-label = ÜBERSPR.

# Kontext: main.rs, println
cli-test-summary = { $passed } bestanden, { $failed } fehlgeschlagen, { $timeouts } Zeitüberschreitung, { $hints } Hinweise, { $skipped } übersprungen{ $strict }

# Kontext: main.rs, Err
cli-test-u64-address-missing = --backend u64:HOST[:PORT]: Adresse fehlt

# Kontext: main.rs, Err
cli-test-m65-port-missing = --backend m65:SCHNITTSTELLE[@BAUD]: Schnittstelle fehlt

# Kontext: main.rs, Err
cli-test-hw-backend-invalid = --backend { $b }: u64:HOST[:PORT] oder m65:SCHNITTSTELLE[@BAUD] (Adresse angeben)

# Kontext: main.rs, Err
cli-test-hw-needs-yes = Hardware-Test abgebrochen: ohne Terminal bitte ausdrücklich --yes angeben

# Kontext: main.rs, eprint
cli-test-hw-ask = Auf dieser Hardware testen? [j/N]

# Kontext: main.rs, Text
cli-deny-license-invalid = --deny-license erwartet eine Liste von SPDX-Kennungen (z. B. GPL-3.0-only,GPL-3.0-or-later)

# Kontext: main.rs, Text
cli-json-needs-license-report = --json gilt nur zusammen mit --license-report

# Kontext: main.rs, Text
cli-license-no-sources = Das Projekt hat keine Quellen (Abschnitt "sources")

# Kontext: main.rs, Text
cli-license-ok = OK      { $f } -> { $artifact } ({ $size } Bytes){ $mark }

# Kontext: main.rs, Text
cli-license-error = FEHLER  { $f }

# Kontext: main.rs, println
cli-suggest-line = { $number }  Zeile { $line }: { $from } → { $to }{ $reg }   { $text }

# Kontext: main.rs, println
cli-suggest-count = { $count } Vorschlag/Vorschläge; mit --apply 1,3 -o datei auswählen (nichts wurde geändert)

# Kontext: main.rs, Text
cli-apply-invalid-number = --apply: '{ $part }' ist keine Nummer eines Vorschlags (1 bis { $max })

# Kontext: main.rs, Text
cli-apply-needs-output = --apply braucht -o <ausgabe> (die Quelldatei wird nie überschrieben)

# Kontext: main.rs, Text
cli-apply-would-overwrite = { $out }: Ausgabe würde die Quelldatei überschreiben (mit -o anderen Namen wählen)

# Kontext: main.rs, println
cli-apply-done = { $out }: { $count } Adresse(n) als Hex geschrieben

# Kontext: main.rs, Text
cli-output-would-overwrite = { $p }: Ausgabe würde die Quelldatei überschreiben (mit -o anderen Namen wählen)

# Kontext: main.rs, println
cli-migrate-nothing = nichts umzustellen

# Kontext: main.rs, println
cli-migrate-pair = { $a } -> { $b }

# Kontext: main.rs, println
cli-migrate-done = { $count } Datei(en) umgestellt; build/export/ entsteht beim nächsten Bauen neu

# Kontext: main.rs, Text
cli-debuglog-no-config-dir = vcc debuglog: kein Konfigurationsordner (HOME bzw. APPDATA fehlt)

# Kontext: main.rs, eprintln
cli-feedback-link-too-long = Hinweis: der Link ist auch gekürzt noch länger als { $max } Zeichen; manche Mailprogramme schneiden ihn ab (--json liefert den vollen Text).

# Kontext: main.rs, eprintln
cli-feedback-mail-opened = Mailprogramm geöffnet; gesendet wird erst dort.

# Kontext: main.rs, eprintln
cli-update-not-in-package = vcc update: Dieses Paket enthält keine Update-Prüfung (Auswahl IDE-UPDATE).

# Kontext: main.rs, eprintln
cli-update-checking = Prüfe { $source } …

# Kontext: main.rs, println
cli-update-no-check-yet = Noch keine Prüfung gespeichert (vcc update fragt nach).

# Kontext: main.rs, println
cli-update-yanked = Hinweis: Version { $installed } wurde zurückgezogen (fehlerhaft); bitte wechsle auf eine neuere, sobald es sie gibt.

# Kontext: main.rs, println
cli-update-available = Neu: Version { $version } verfügbar.

# Kontext: main.rs, println
cli-update-format-warning = Achtung: Die neue Version schreibt ein neueres Projektformat; ältere Versionen können diese Projekte danach nicht mehr lesen.

# Kontext: main.rs, println
cli-update-show-notes = Änderungen anzeigen: vcc update --notes

# Kontext: main.rs, println
cli-update-up-to-date = Aktuell: keine neuere Version bekannt (neueste in der Liste: { $latest }).

# Kontext: main.rs, Text
cli-import-basic = BASIC ({ $dialect }, { $lines } Zeilen)

# Kontext: main.rs, Text
cli-import-error = FEHLER: { $m }

# Kontext: main.rs, println
cli-import-hint = → übernehmen: vcc import { $input } NAME ...   oder   --all

# Kontext: main.rs, Text
cli-import-not-on-disk = »{ $n }« steht nicht auf der Diskette (vcc import { $input } zeigt das Verzeichnis)

# Kontext: emulator.rs, Text
cli-emu-no-package = kein automatisches Paket für dieses System

# Kontext: emulator.rs, Text
cli-emu-portable-versions = portabel ladbar: { $versions }

# Kontext: emulator.rs, println
cli-emu-found-none = nein

# Kontext: emulator.rs, println
cli-emu-found = { $label } { $path }

# Kontext: emulator.rs, println
cli-emu-installable-no = nein{ $hint }

# Kontext: emulator.rs, eprintln
cli-emu-version-unknown = { $name } { $v } gibt es nicht im Katalog{ $available }

# Kontext: emulator.rs, eprintln
cli-emu-no-package-for = Für { $name } gibt es auf diesem System kein automatisches Paket.{ $hint }
  Download-Seite: { $page }

# Kontext: emulator.rs, println
cli-emu-install-plan = Emulator:  { $name } { $version }
  Adresse:   { $url }
  SHA-256:   { $sha }
  Größe:     { $size }
  Zielordner: { $dir }

# Kontext: emulator.rs, eprintln
cli-emu-error = Fehler: { $err }

# Kontext: roms.rs, println
cli-roms-more-files = { $pad } +{ $count } weitere Datei(en) derselben Rolle

# Kontext: roms.rs, println
cli-roms-missing = { $id }: fehlt { $roles }

# Kontext: roms.rs, println
cli-roms-others-found = { $n } passende ROM-Datei(en) in anderen Programmen gefunden: vcc roms import --from-others (Vorschau), mit --yes kopieren

# Kontext: roms.rs, println
cli-roms-unknown-revision = (Symbole und Offsets können abweichen)

# Kontext: roms.rs, println
cli-roms-install-dir = ROM-Ordner (Schreibziel):  { $dir }

# Kontext: roms.rs, println
cli-roms-portable-active = Portabler Modus:           aktiv ({ $dir }), { $how }

# Kontext: roms.rs, println
cli-roms-portable-unavailable = Portabler Modus:           nicht verfügbar – { $why }

# Kontext: roms.rs, println
cli-roms-portable-off-hint = Portabler Modus:           aus (aktivieren: Ordner roms/ oder Datei portable in { $dir })

# Kontext: roms.rs, println
cli-roms-portable-off = Portabler Modus:           aus

# Kontext: roms.rs, println
cli-roms-env-set = VC_ROMS ist gesetzt: ersetzt Benutzer- und portablen Ordner

# Kontext: roms.rs, println
cli-roms-searched = Gesuchte Orte (in dieser Reihenfolge):

# Kontext: roms.rs, println
cli-roms-files = { $count } Datei(en)

# Kontext: roms.rs, println
cli-roms-absent = nicht vorhanden

# Kontext: roms.rs, println
cli-roms-foreign-off = (ROMs anderer Programme sind aus; mit --foreign mitsuchen)

# Kontext: roms.rs, println
cli-roms-none = Keine ROM-Dateien gefunden.

# Kontext: roms.rs, println
cli-roms-copied = kopiert  { $what } { $name }{ $src } → { $dest }

# Kontext: roms.rs, eprintln
cli-roms-copy-error = FEHLER   { $name }: { $e }

# Kontext: roms.rs, println
cli-roms-would-copy = würde kopieren { $what } { $name }{ $src } → { $dest }  { $label }

# Kontext: roms.rs, println
cli-roms-skipped = übersprungen { $name }: { $why }

# Kontext: roms.rs, println
cli-roms-nothing-copied = Nichts kopiert. Mit --yes ausführen.

# Kontext: roms.rs, eprintln
cli-roms-needs-file = { $a } gehört hinter --file

# Kontext: roms.rs, eprintln
cli-roms-as-missing = { $file }: --as system/rolle fehlt

# Kontext: roms.rs, Text
cli-file-error-f = { $f }: { $e }

# Kontext: roms.rs, println
cli-roms-table-written = { $count } Einträge in { $f }

cli-emu-label-device = Gerät:
cli-emu-device-yes = ja, kein Programm; Einstellungen ▸ Zielsysteme ▸ Hardware
cli-emu-label-systems = Systeme:
cli-emu-label-role = Rolle:
cli-emu-device-list = Gerät (kein Programm)
cli-emu-label-homepage = Homepage:
cli-emu-label-license = Lizenz:
cli-emu-label-found = Gefunden:
cli-emu-label-auto = Automatisch:
cli-emu-label-installable = Installierbar:
cli-emu-installable-yes = { $version } ({ $size }){ $default }  vcc emulator install { $id }@{ $version }
cli-emu-default-mark = [Vorgabe]
cli-emu-available-list = (verfügbar: { $versions })
cli-emu-program = Programm: { $path }
# $id: Kennung des Emulators | Beispiel: vice
# $known: bekannte Kennungen | Beispiel: vice, atari800
cli-emu-unknown = vcc emulator: unbekannter Emulator „{ $id }“ (bekannt: { $known })

cli-roms-expected = (erwartet { $size } Byte)

cli-roms-unknown-system = unbekanntes System { $id }

cli-roms-path-usage = vcc roms path <system/rolle>, z. B. c64/kernal

cli-roms-unknown = (unbekannt)

cli-roms-user-dir = Benutzerordner:            { $dir }

cli-roms-portable-by-marker = durch Markierungsdatei

cli-roms-portable-by-folder = Ordner vorhanden

cli-roms-cache-dir = Zwischenspeicher:          { $dir }

cli-roms-unknown-revision-label = (unbekannte Revision)

cli-roms-already = vorhanden { $what } { $name }  (= { $path })

cli-media-conflict-suffix = – FARBKONFLIKT, auf nächstliegende Farben reduziert

cli-test-tag-ok = OK

cli-test-tag-error = FEHLER

cli-test-tag-hint = HINWEIS

cli-test-tag-timeout = ZEIT

cli-test-hw-warning =
    ACHTUNG: Die Tests laufen auf echter Hardware: { $label }.
    { "  " }Der laufende Zustand des Geräts geht verloren (Reset, Programm laden, Speicher beschreiben); jeder Test wird nach seinem Zeitlimit abgebrochen, danach läuft das Gerät weiter.

cli-test-hw-cancelled = Hardware-Test abgebrochen

cli-license-main-mark = [Startprogramm]

cli-feedback-mail-failed = vcc feedback: Mailprogramm nicht gestartet ({ $e })

cli-help-data-invalid = Hilfe: data/cli/{ $name }.json ist ungültig: { $e }

cli-help-usage-label = Aufruf:

cli-help-overview-usage = vcc <befehl> [optionen]

cli-help-overview-hint = Hilfe zu einem Befehl: vcc help <befehl>  oder  vcc <befehl> --help

cli-help-general = Allgemein

cli-help-row-help = Hilfe zu einem Befehl oder Thema (env, exit-codes)

cli-help-row-man = man-Seite eines Befehls (roff) ausgeben

cli-help-row-completion = Vervollständigung für powershell, bash, zsh oder fish ausgeben

cli-help-row-version = Version anzeigen

cli-help-row-license = Lizenzhinweis anzeigen (Einzelheiten: LICENSING.md)

cli-help-row-lang = Sprache de oder en (Umgebungsvariable VC_LANG überstimmt)

cli-help-examples = Beispiele

cli-help-more = Ausführlicher: vcc help --all   ·   Umgebungsvariablen: vcc help env   ·   Rückgabewerte: vcc help exit-codes

cli-help-values = Werte: { $values }.

cli-help-default = (Standard: { $value })

cli-help-repeatable = Mehrfach möglich.

cli-help-required = Erforderlich.

cli-help-form-base = Grundform

cli-help-form-named = Form „{ $id }“

cli-help-arguments = Argumente

cli-help-optional = (optional)

cli-help-options = Optionen

cli-help-more-options = Weitere Optionen

cli-help-exit = Rückgabe:

cli-help-see-also = Siehe auch:

cli-help-aliases = Auch als: { $names }

cli-help-env = Umgebungsvariablen

cli-help-exit-codes = Rückgabewerte

cli-usage-incomplete = vcc { $cmd }: Aufruf unvollständig oder ungültig

cli-usage-help-hint = Hilfe:   vcc { $cmd } --help

cli-unknown-command = vcc: unbekannter Befehl „{ $word }“

cli-did-you-mean = Meinten Sie { $name }?

cli-all-commands-hint = Alle Befehle: vcc --help

cli-unknown-option = vcc { $cmd }: unbekannte Option „{ $opt }“

cli-completion-header-bash = # Vervollständigung für vcc (bash). Einmalig: eval "$(vcc completion bash)"

cli-completion-header-powershell = # Vervollständigung für vcc (PowerShell). Einmalig: vcc completion powershell | Out-String | Invoke-Expression

cli-completion-header-zsh = # Vervollständigung für vcc (zsh). Datei als _vcc in einen Ordner des fpath legen oder: eval "$(vcc completion zsh)"

cli-completion-header-fish = # Vervollständigung für vcc (fish). Einmalig: vcc completion fish | source

cli-completion-desc-help = Hilfe zu einem Befehl

cli-completion-desc-man = man-Seite ausgeben

cli-completion-desc-completion = Vervollstaendigung ausgeben

cli-completion-desc-version = Version anzeigen

cli-completion-zsh-command = befehl

cli-completion-zsh-file = datei

cli-completion-fish-form = Form

cli-completion-help =
    vcc completion - Vervollständigung (Tab) für die Shell ausgeben

    Aufruf:  vcc completion <shell>
             shell: powershell | bash | zsh | fish

    Beispiele:
      vcc completion powershell | Out-String | Invoke-Expression
      eval "$(vcc completion bash)"
      vcc completion fish | source

cli-completion-unknown-shell =
    vcc completion: unbekannte Shell „{ $shell }“
    Erlaubt: powershell, bash, zsh, fish

cli-completion-shell-missing =
    vcc completion: Shell fehlt
    Aufruf:  vcc completion <powershell|bash|zsh|fish>

cli-man-generated-comment = .\" Erzeugt von "vcc man"; nicht von Hand ändern (Quelle: data/cli/*.json).

cli-man-manual = VintageCoder Handbuch

cli-man-default = Standard: { $value }.

cli-man-form-named = Form { $id }

cli-man-sh-examples = BEISPIELE

cli-man-sh-synopsis = ÜBERSICHT

cli-man-sh-description = BESCHREIBUNG

cli-man-sh-options = OPTIONEN

cli-man-sh-notes = HINWEISE

cli-man-sh-exit = RÜCKGABEWERT

cli-man-sh-see-also = SIEHE AUCH

cli-man-sh-commands = BEFEHLE

cli-man-sh-environment = UMGEBUNG

cli-man-aliases = Auch als: { $names }.

cli-man-arg-command = befehl

cli-man-arg-options = optionen

cli-man-command-line = { $summary } (siehe \fBvcc\-{ $name }\fR(1))

cli-man-general-help = Hilfe zu einem Befehl oder Thema (env, exit-codes).

cli-man-general-man = man-Seite ausgeben.

cli-man-general-completion = Vervollständigung für powershell, bash, zsh oder fish ausgeben.

cli-man-env-data = Ordner der Hardware-Datenbank (data).

cli-man-env-roms = ROM-Ordner des Benutzers.

cli-man-env-xemu = Pfad zum Xemu-Programm.

cli-man-env-xemu-sd = Ordner mit MEGA65.ROM für Xemu.

cli-man-env-win = Fenstergröße, z. B. 1400x860.

cli-man-opt-help = Hilfe anzeigen.

cli-man-opt-version = Version anzeigen.

cli-man-opt-version-short = Version anzeigen

cli-man-ide-summary = grafische Entwicklungsumgebung für Retro-Systeme

cli-man-ide-synopsis = vintagecoder [projekt-ordner | datei.vcprj | datei.asm | datei.bas]

cli-man-ide-description = Öffnet die IDE. Ohne Argument erscheint der Startbildschirm, mit einem Pfad das Projekt oder die Datei.

cli-man-ide-example-start = Startbildschirm

cli-man-ide-example-open-project = Projekt öffnen

cli-man-ide-example-open-source = Quelltext öffnen

cli-man-ide-exit = 0 nach normalem Beenden.

cli-man-lsp-summary = Language Server für VintageCoder-Assembler

cli-man-lsp-description-1 = Spricht das Language Server Protocol über Standardeingabe und Standardausgabe. Er wird normalerweise vom Editor gestartet, nicht von Hand: ohne Eingabe wartet er auf Nachrichten und gibt nichts aus.

cli-man-lsp-description-2 = Im Editor als Server mit dem Befehl vc-lsp ohne Argumente für Dateien mit der Endung .asm einbinden.

cli-man-lsp-exit = 0 nach geordnetem Beenden (shutdown, exit), sonst 1; 2 bei unbekannter Option.

cli-man-all-missing-dir =
    vcc man --all: Zielordner fehlt
    Aufruf:  vcc man --all <ordner>

cli-man-all-written = { $count } Seiten nach { $dir } geschrieben.

cli-man-help =
    vcc man - man-Seite (roff) ausgeben

    Aufruf:  vcc man [befehl]
             vcc man --all <ordner>

    Beispiele:
      vcc man asm | man -l -
      vcc man --all installer/linux/man

cli-md-form-named = Form `{ $id }`

cli-md-table-arguments =
    | Argument | Bedeutung |
    |---|---|

cli-md-table-options =
    | Option | Bedeutung | Standard |
    |---|---|---|

cli-md-examples = Beispiele:

cli-md-note = > **Hinweis:** { $text }

cli-md-exit = Rückgabe: { $codes }

cli-md-aliases = Auch als: { $names }.

cli-md-generated-comment = <!-- Erzeugt von "vcc help --markdown" aus data/cli/*.json; nicht von Hand ändern. -->

cli-md-title = # Kommandozeile: vcc

cli-md-help-line = Hilfe im Programm: `vcc --help`, `vcc help <befehl>`, `vcc <befehl> --help`. Handbuchseiten: `vcc man <befehl>`; Vervollständigung: `vcc completion <shell>`.

cli-md-env-table =
    ## Umgebungsvariablen

    | Variable | Bedeutung |
    |---|---|

cli-md-exit-table =
    ## Rückgabewerte

    | Wert | Bedeutung |
    |---|---|

cli-test-default-name = Test { $n }

cli-test-default-measure-name = Messung { $n }

# Kontext: main.rs, println
cli-dis-segment = ; Segment { $n }: ${ $addr }-${ $end } ({ $size } Bytes)

# Kontext: main.rs, println
cli-dis-undocumented = ; undokumentiert

# Kontext: main.rs, println
cli-inc-written = { $o } geschrieben

# Kontext: main.rs, Text
cli-file-error-o = { $o }: { $e }

# Kontext: main.rs, eprintln
cli-basic-roundtrip-ok = { $input }: Rundreise in Ordnung

# Kontext: main.rs, eprintln
cli-disk-problem = Problem: { $m }

# Kontext: main.rs, println
cli-disk-entry = { $name } { $sectors } Sektoren  ab { $start }{ $locked }

# Kontext: main.rs, println
cli-disk-free-sectors = { $count } freie Sektoren

# Kontext: main.rs, println
cli-media-color-replaced = Farbe { $color } ({ $pixels } Pixel) ersetzt

# Kontext: main.rs, println
cli-organize-adjust-paths = Pfade anpassen: { $f } ({ $n })

# Kontext: main.rs, println
cli-structure-heading = Projektstrukturen (--structure):

# Kontext: main.rs, println
cli-test-junit-written = JUnit-XML: { $j }

# Kontext: main.rs, Text
cli-file-error-j = { $j }: { $e }

# Kontext: main.rs, println
cli-test-strict-mark = (--strict)

# Kontext: main.rs, eprintln
cli-build-disk-error = Diskette: { $e }

# Kontext: main.rs, eprintln
cli-license-report-written = Lizenzbericht: build/LICENSES.txt

# Kontext: main.rs, eprintln
cli-license-report-error = build/LICENSES.txt: { $e }

# Kontext: main.rs, println
cli-migrate-long-names = Lange Namen → Kurznamen:

# Kontext: main.rs, println
cli-feedback-mail-preview = An: { $to }
  Betreff: { $subject }
  { $body }

# Kontext: main.rs, println
cli-update-release-page = Releaseseite: { $url }

# Kontext: main.rs, Text
cli-secs-needs-number = --secs: Zahl erwartet

# Kontext: main.rs, Text
cli-streams-listen-usage = vcc streams listen --host HOST LIED.vcsong

# Kontext: main.rs, Text
cli-file-error-file = { $file }: { $e }

# Kontext: main.rs, println
cli-streams-player-size = Abspielprogramm: { $size } Byte

# Kontext: main.rs, println
cli-streams-error = FEHLER { $e }

# Kontext: main.rs, println
cli-streams-buffer = Puffer: Füllstand { $fill } ms, { $filled } Lücken gefüllt, { $underruns } Unterläufe, { $overruns } Überläufe

# Kontext: main.rs, println
cli-streams-wav = WAV: { $seconds } s → { $path }

# Kontext: main.rs, println
cli-streams-stopped = Strom gestoppt{ $suffix }.

# Kontext: main.rs, eprintln
cli-streams-usage = vcc streams probe|trace --host HOST [--video|--audio|--debug] [--secs N] [--password PW] [--multicast] [--record ORDNER [--count N]]

# Kontext: main.rs, Text
cli-count-needs-number = --count: Zahl erwartet

# Kontext: main.rs, Text
cli-host-missing = --host fehlt

# Kontext: main.rs, Text
cli-streams-video-debug-conflict = Bild und Debug-Strom laufen nie zugleich (--video oder --debug)

# Kontext: main.rs, println
cli-streams-target = { $label }: Ziel { $target }

# Kontext: main.rs, println
cli-streams-kind-error = { $label }: FEHLER { $e }

# Kontext: main.rs, Text
cli-dir-error = { $dir }: { $e }

# Kontext: main.rs, println
cli-streams-recorded = { $n } Pakete → { $path }

# Kontext: main.rs, println
cli-streams-all-stopped = Ströme gestoppt.

# Kontext: main.rs, Text
cli-streams-shot-usage = vcc streams shot --host HOST --out DATEI.png

# Kontext: main.rs, Text
cli-address-invalid = Adresse „{ $s }“ ungültig

# Kontext: main.rs, ok_or_else
cli-address-unknown = Adresse oder Symbol „{ $s }“ unbekannt (Symbole nur mit --debug datei.vcdbg)

# Kontext: main.rs, Text
cli-address-out-of-range = Adresse „{ $s }“ liegt außerhalb von $0000..$FFFF

# Kontext: main.rs, Text
cli-cpu-needs-id = --cpu: ID erwartet

# Kontext: main.rs, Text
cli-measure-needs-pair = --measure: A:B erwartet

# Kontext: main.rs, Text
cli-tail-needs-number = --tail: Zahl erwartet

# Kontext: main.rs, Text
cli-no-data-dir-short = Datenverzeichnis nicht gefunden (--data-dir)

# Kontext: main.rs, Text
cli-trace-45gs02 = Der 45GS02 wird nicht ausgewertet (nur 6502/6510).

# Kontext: main.rs, Text
cli-measure-invalid = --measure { $m }: Form A:B erwartet

# Kontext: main.rs, Text
cli-trace-failed = Ablaufprotokoll: { $e }

# Kontext: main.rs, println
cli-trace-started = Debug-Strom gestartet (Bildstrom gestoppt), { $secs } s ...

# Kontext: main.rs, println
cli-trace-heading = Ablaufprotokoll (6510 Only), { $secs } s

# Kontext: main.rs, println
cli-trace-cycles = Zyklen: { $cycles } (davon { $stalls } BA-Pausen), gemessen { $rate } Mio/s; Befehle: { $insns } (IRQ { $irqs }, NMI { $nmis }), Reset-Vektor gelesen: { $resets }

# Kontext: main.rs, println
cli-trace-sync = Befehlserkennung: { $state }; Widersprüche { $desyncs }, Einstiege { $syncs }

# Kontext: main.rs, println
cli-trace-packets = Pakete: { $packets }, Verluste: { $lost } ({ $percent } %), Neuanfänge durch Verlust: { $epochs }

# Kontext: main.rs, println
cli-trace-coverage = Abdeckung: { $covered } Adressen mit Opcode-Abruf; SID/VIC-Schreibzugriffe: { $io_writes }

# Kontext: main.rs, println
cli-trace-unreached = Nicht erreichte Zeilen: { $count }{ $hint }

# Kontext: main.rs, println
cli-trace-last-instructions = Letzte { $count } Befehle (Zyklus, PC, Zyklen, Quelle):

# Kontext: main.rs, println
cli-trace-recorded = { $n } Befehle → { $path }

# Kontext: main.rs, println
cli-trace-dropped = ACHTUNG: { $count } Befehle nicht aufgezeichnet (Datenträger zu langsam)

cli-stream-stats = { $label }: { $state }  { $packets } Pakete ({ $rate }/s, { $mbit } Mbit/s), { $lost } fehlen ({ $percent } %), { $rejected } verworfen, { $restarts } Neustarts

cli-stream-audio-stats = Ton: { $state }  { $packets } Pakete, { $lost } fehlen ({ $percent } %)

cli-stream-ultimate-reset = Ultimate zurückgesetzt

cli-trace-synced = synchron

cli-trace-not-synced = nicht synchron

cli-trace-examples-below = (Beispiele unten)

cli-error-text = { $text }

cli-license-report-title = Lizenzen der übersetzten Quelltexte (erzeugt von VintageCoder)

cli-license-report-how = Gelesen werden die Zeilen SPDX-License-Identifier in den Kopfzeilen der Dateien (sonst ein Ordnervermerk LICENSE).

cli-license-report-caveat = Das ist nur so gut wie diese Kopfzeilen; Binärdaten und Dateien ohne Kennung werden nicht erkannt. Kein Rechtsurteil.

# I3b
cli-define-expected = -D erwartet NAME=Zahl
# $choices: erlaubte Kategorien | Beispiel: a, b
cli-category-unknown = --category: unbekannt ({ $choices })
# $id: Kennung | Beispiel: x
cli-structure-unknown = Projektstruktur '{ $id }' unbekannt (vcc new --list)

# Fehler (Kommandozeile): Wert von --range fehlt oder ist ungültig
cli-convert-range-expected = --range erwartet einen Bereich der alten Zeilennummern, z. B. 100-200

# Hinweis (Kommandozeile): Ergebnis einer Umwandlung
# $changed: Zahl geänderter Zeilen | Beispiel: 12
# $new: Zahl neu erzeugter Labels | Beispiel: 3
# $kept: Zahl unverändert gebliebener Labels | Beispiel: 2
cli-convert-summary = { $changed } Zeilen geändert, { $new } Labels neu, { $kept } Labels unverändert

# Hinweis (Kommandozeile): ein automatisch erzeugtes Label
# $name: Name des Labels | Beispiel: z100
# $file: Datei | Beispiel: spiel.bas
# $line: Zeile der Quelle | Beispiel: 20
cli-convert-new-label = neues Label @{ $name } ({ $file }, Zeile { $line })

# Ausgabe (Kommandozeile): Vorschau einer geänderten Zeile
# $file: Datei | Beispiel: spiel.bas
# $line: Zeile der Quelle | Beispiel: 20
# $old: alter Text | Beispiel: 20 GOTO 10
# $new: neuer Text | Beispiel: GOTO @start
cli-convert-preview-line = { $file }:{ $line }: { $old } → { $new }

# Ausgabe (Kommandozeile): Datei geschrieben
# $file: Datei | Beispiel: spiel.bas
cli-convert-written = { $file }: geschrieben

# Hinweis (Kommandozeile): auch eingebundene Dateien ändern sich, werden aber ohne --write nicht geschrieben
# $count: Zahl der Dateien | Beispiel: 1
cli-convert-includes-hint = { $count } eingebundene Datei(en) ändern sich ebenfalls; mit --write werden alle Dateien geschrieben, mit --preview sieht man alle Änderungen

# Kontext: main.rs, vcc migrate includes, eine umgestellte Zeile
# $file: Quelltext | Beispiel: main.asm
# $line: Zeile | Beispiel: 12
# $old: alte Zeile | Beispiel: .include "build/export/hero.asm"
# $new: neue Zeile | Beispiel: .include "graphics/hero.vcsprite"
cli-migrate-includes-change = { $file }:{ $line }: { $old }  →  { $new }

# Kontext: main.rs, vcc migrate includes, eine entfallende Zeile
# $file: Quelltext | Beispiel: main.asm
# $line: Zeile | Beispiel: 13
# $old: alte Zeile | Beispiel: .include "build/export/hero.anim.asm"
cli-migrate-includes-removed = { $file }:{ $line }: { $old }  →  entfällt (steckt im Include der Mediendatei)

# Kontext: main.rs, vcc migrate includes, eine Zeile, die bleibt
# $file: Quelltext | Beispiel: main.asm
# $line: Zeile | Beispiel: 14
# $old: alte Zeile | Beispiel: .include "build/export/x.asm"
# $why: Grund | Beispiel: Kein eindeutiges Medium
cli-migrate-includes-skipped = { $file }:{ $line }: { $old }  –  { $why }

# Kontext: main.rs, vcc migrate includes, Abschluss nach dem Umstellen
# $count: Zahl der umgestellten oder entfallenen Zeilen | Beispiel: 3
# $skipped: Zahl der Zeilen, die bleiben | Beispiel: 0
cli-migrate-includes-done = { $count } Zeile(n) umgestellt, { $skipped } unverändert.

# Kontext: main.rs, vcc migrate includes, Abschluss im Trockenlauf
# $count: Zahl der Zeilen, die sich ändern würden | Beispiel: 3
# $skipped: Zahl der Zeilen, die bleiben | Beispiel: 0
cli-migrate-includes-dry = { $count } Zeile(n) würden umgestellt, { $skipped } blieben unverändert. Mit --apply umstellen.
