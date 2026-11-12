---
id: installation
titel: Installation und Ersteinrichtung
reihenfolge: 0
stufe: Einsteiger
zeit: 30 Minuten
siehe_auch: [willkommen, erstes-projekt, oberflaeche]
referenz: [start, settings, roms, mega65-hardware]
---

# Installation und Ersteinrichtung

**Ziel:** Du installierst VintageCoder auf macOS, Windows oder Linux (Debian, Ubuntu, Fedora, openSUSE), startest es zum ersten Mal, richtest es mit dem **Konfigurationsassistenten** ein (Emulator, Systeme, Projektordner) und prüfst mit einem Hello-World-Programm und **F5**, dass alles läuft.


## Voraussetzungen

- einen Rechner mit **macOS auf Apple-Chip (arm64)**, **Windows (64 Bit, x86_64)** oder **Linux (x86_64)**; Intel-Macs sowie ARM-Rechner unter Windows und Linux werden **nicht** unterstützt, dafür gibt es keine Pakete,
- das Installationspaket für dein System,
- einen **Emulator** für das System, das dich interessiert (siehe unten); den installierst du selbst, der Assistent zeigt dir nur, ob er gefunden wurde,
- optional die **ROM-Dateien** deiner Zielsysteme.

> **Info:** Die **Ersteinrichtung** übernimmt der Konfigurationsassistent. Er öffnet sich beim allerersten Start von selbst, prüft, was auf deinem Rechner schon da ist, und schreibt erst nach deiner Bestätigung etwas in deine Einstellungen. Dieses Kapitel führt dich der Reihe nach durch: Download, Installation, erster Start, Assistent, erstes Programm. Alles, was der Assistent einstellt, kannst du auch von Hand in den **Einstellungen** ändern (Abschnitt „Von Hand einstellen“).

## Download

Die Installationspakete gibt es unter [github.com/CDrueeke/VintageCoder/releases](https://github.com/CDrueeke/VintageCoder/releases) (Version 0.1.0: Tag `v0.1.0`, als Vorabversion gekennzeichnet) und auf der Webseite des Projekts, [vintagecoder.drueeke.de](https://vintagecoder.drueeke.de/). Du brauchst nur die Datei(en) für dein System:

| System | Datei(en) | Voraussetzung |
|---|---|---|
| **Windows** | `VintageCoder-<Version>-Setup.exe` | Windows 11 getestet, 64 Bit (x86_64) |
| **macOS** | `VintageCoder-<Version>-macos-arm64.dmg` | Mac mit Apple-Chip (arm64); Intel-Macs nicht unterstützt |
| **Debian, Ubuntu** | `vintagecoder-cli_<Version>_amd64.deb` und `vintagecoder_<Version>_amd64.deb` | Debian 13 oder neuer, Ubuntu 24.04 oder neuer |
| **Fedora, openSUSE** | `vintagecoder-cli-<Version>-1.x86_64.rpm` und `vintagecoder-<Version>-1.x86_64.rpm` | Fedora 45 oder neuer, openSUSE Tumbleweed (jeweils glibc 2.44 oder neuer) |
| **andere Linux-Systeme** | `vintagecoder-<Version>-linux-x86_64.tar.gz` | x86_64, glibc 2.39 oder neuer |

Auf den GitHub-Release-Seiten tragen die Dateien je nach Distribution einen Zusatz im Namen (zum Beispiel `…debian13.deb`, `…fedora45….rpm`), damit gleiche Namen nicht kollidieren; der Inhalt ist derselbe. Zu jedem Release gehört eine Datei `SHA256SUMS`. Prüfe damit den Download (Linux: `sha256sum -c SHA256SUMS`, macOS: `shasum -a 256 -c SHA256SUMS`, Windows: `Get-FileHash` in PowerShell und Vergleich der Summe).

> **Achtung:** Die Pakete sind **nicht signiert** (kein Entwicklerzertifikat, keine Beglaubigung durch Apple). Windows und macOS warnen deshalb beim ersten Start; wie du damit umgehst, steht in den Abschnitten zu deinem System. Lade Pakete nur aus dem offiziellen Release oder von der Webseite des Projekts.

Wie du die Datei öffnest, steht in den folgenden Abschnitten.

## Windows

> **Info:** Die Windows-Fassung ist noch **in Arbeit**. Das Setup-Programm gibt es, es ist aber noch nicht so gründlich erprobt wie die macOS-Fassung. Geprüft sind Installation, Start und Deinstallation unter Windows 11 (x86_64); Windows 10, 32-Bit-Windows und Windows auf ARM sind nicht geprüft beziehungsweise nicht unterstützt.

1. Starte das Setup-Programm `VintageCoder-<Version>-Setup.exe`. Weil es **nicht signiert** ist, kann Windows SmartScreen vorher „Der Computer wurde durch Windows geschützt“ (je nach Version „Windows hat den PC geschützt“) melden. Klicke dann auf **Weitere Informationen** und danach auf **Trotzdem ausführen**, wenn du die Datei aus dem offiziellen Release oder von der Webseite des Projekts geladen hast. Folge dann den Fragen. Das Setup braucht nur dann Administratorrechte, wenn du für alle Benutzer installieren willst.
2. Im Setup stehen drei Wahlmöglichkeiten:
   - **Symbol auf dem Desktop anlegen** (aus),
   - **Kommandozeilenprogramm vcc zum PATH hinzufügen** (aus; hake es an, wenn du `vcc` in jeder Eingabeaufforderung benutzen möchtest),
   - **Beispielprojekte nach „Dokumente\VintageCoder\Beispiele“ kopieren** (an).
3. Zum Schluss kannst du VintageCoder gleich starten. Das Setup verknüpft außerdem die Endung `.vcprj`: Ein Doppelklick auf eine Projektdatei öffnet sie in VintageCoder.

## macOS

1. Öffne die Datei `VintageCoder-<Version>-macos-<Architektur>.dmg`. Im Fenster liegen **VintageCoder.app**, eine Verknüpfung zum Ordner **Programme**, der Ordner **Beispiele** und die Datei **Kommandozeile installieren.command**.
2. Ziehe **VintageCoder.app** auf **Programme**. Kopiere auch den Ordner **Beispiele** an einen Ort, an dem du schreiben darfst (zum Beispiel in deine Dokumente), denn Bauen legt Dateien im Projektordner an.
3. Starte die App. Ist sie nicht von Apple beglaubigt, verweigert macOS (Gatekeeper) das Öffnen. Klicke dann im Ordner „Programme“ mit der **rechten Maustaste** auf VintageCoder und wähle **Öffnen**, danach im Dialog noch einmal **Öffnen**. Das musst du nur beim ersten Start tun. Auf neueren macOS-Versionen erscheint der Eintrag **Öffnen** im Dialog unter Umständen nicht; dann öffnest du **Systemeinstellungen ▸ Datenschutz & Sicherheit**, scrollst zum Hinweis zu VintageCoder und klickst auf **Trotzdem öffnen**.
4. Möchtest du das Kommandozeilenwerkzeug `vcc` im Terminal benutzen, starte (mit der App bereits in „Programme“) per Doppelklick **Kommandozeile installieren.command**. Es verlinkt `vcc` und `vc-lsp` nach `/usr/local/bin`; falls nötig, fragt es nach deinem Passwort. Öffne danach ein neues Terminalfenster.


## Linux (Debian, Ubuntu)

Die `.deb`-Pakete gibt es für **Debian 13** und **Ubuntu 24.04** (jeweils oder neuer), nur für x86_64. Ältere Versionen (Debian 12, Ubuntu 22.04) haben eine zu alte Systembibliothek (glibc) und werden nicht unterstützt. Lade beide Pakete in denselben Ordner. Wer nur die Kommandozeile braucht, installiert nur `vintagecoder-cli`; die Oberfläche (`vintagecoder`) braucht immer das Kommandozeilen-Paket in derselben Version.

1. Öffne ein Terminal in dem Ordner mit den Paketen und installiere beide mit `apt` (das `./` vor den Namen ist wichtig, sonst sucht `apt` in den Paketquellen; benötigte Bibliotheken holt `apt` selbst):

   ```bash
   sudo apt install ./vintagecoder-cli_<Version>_amd64.deb ./vintagecoder_<Version>_amd64.deb
   ```

2. Prüfe die Installation. Dieser Befehl zeigt die Version:

   ```bash
   vcc --version
   ```

3. Starte die Oberfläche aus dem Anwendungsmenü (Eintrag „VintageCoder“, Kategorie Entwicklung) oder im Terminal mit `vintagecoder`. Die Beispielprojekte liegen unter `/usr/share/vintagecoder/examples`; kopiere sie an einen Ort, an dem du schreiben darfst, denn Bauen legt Dateien im Projektordner an.

> **Info:** Am Ende der Installation kann `apt` die Zeile „N: Der Download wird als root und nicht Sandbox-geschützt durchgeführt, da auf die Datei … durch den Benutzer »_apt« nicht zugegriffen werden kann“ ausgeben. Das ist nur ein Hinweis, kein Fehler; die Installation ist vollständig. `apt` lädt Pakete sonst mit dem eingeschränkten Systembenutzer `_apt` und darf in deinen persönlichen Ordner nicht schauen. Möchtest du die Zeile vermeiden, kopiere die Pakete vorher nach `/tmp` und installiere von dort (`sudo apt install /tmp/vintagecoder-cli_<Version>_amd64.deb /tmp/vintagecoder_<Version>_amd64.deb`).

Zum Entfernen genügt `sudo apt remove vintagecoder vintagecoder-cli`.

## Linux (Fedora, openSUSE)

Die `.rpm`-Pakete gibt es für **Fedora 45** und **openSUSE Tumbleweed** (jeweils oder neuer), nur für x86_64. Sie verlangen glibc 2.44 und sind auf älteren Fedora-Versionen und auf openSUSE Leap nicht installierbar. Lade beide Pakete in denselben Ordner, öffne dort ein Terminal und installiere sie; die Paketverwaltung holt benötigte Bibliotheken selbst.

Fedora:

```bash
sudo dnf install ./vintagecoder-cli-<Version>-1.x86_64.rpm ./vintagecoder-<Version>-1.x86_64.rpm
```

openSUSE Tumbleweed (die Pakete sind nicht signiert; `zypper` fragt nach, ob es sie trotzdem installieren soll, oder du gibst `--allow-unsigned-rpm` an):

```bash
sudo zypper install --allow-unsigned-rpm ./vintagecoder-cli-<Version>-1.x86_64.rpm ./vintagecoder-<Version>-1.x86_64.rpm
```

Prüfe danach mit `vcc --version` und starte die Oberfläche wie oben beschrieben. Zum Entfernen: `sudo dnf remove vintagecoder vintagecoder-cli` beziehungsweise `sudo zypper remove vintagecoder vintagecoder-cli`.

## Linux (andere Systeme, Archiv)

Das Archiv `vintagecoder-<Version>-linux-x86_64.tar.gz` braucht keine Installation: Entpacke es (`tar xzf vintagecoder-<Version>-linux-x86_64.tar.gz`) und starte `usr/bin/vcc --version`; die Daten findet das Programm von selbst. Mit `sudo cp -a usr/. /usr/local/` installierst du es nach `/usr/local`. Für die Oberfläche (`usr/bin/vintagecoder`) müssen die üblichen Bibliotheken vorhanden sein (ALSA, xkbcommon-x11, GL/EGL, Wayland, Xcursor, Xrandr, Xi); die Liste steht in der Hilfeseite „Linux“. Voraussetzung ist glibc 2.39 oder neuer. Für das Archiv ist nur die Kommandozeile geprüft, nicht die Oberfläche.

> **Info:** Die Linux-Pakete wurden in frischen Containern der jeweiligen Distribution installiert und getestet (Installation, `vcc`, Start der Oberfläche mit Software-Darstellung). Auf einem echten Desktop mit Ton, Dateidialogen und Wayland-/X11-Sitzung sind sie noch nicht geprüft.

## Der erste Start

1. Starte VintageCoder. Als Erstes erscheint der **Startbildschirm**: der Name, die Version mit der Marke **ALPHA** und zwei Kästen.

![Der Startbildschirm: Name und Version, darunter der Alpha-Hinweis, der Haftungsausschluss, der Haken „Beim nächsten Start nicht mehr anzeigen“ und der Knopf „Verstanden, los geht’s“](bild:start-splash "Der Startbildschirm: erst lesen, dann bestätigen")

2. Lies die beiden Kästen:
   - **Alpha-Version** (orange): VintageCoder ist noch im Alpha-Stadium, enthält mit hoher Wahrscheinlichkeit noch viele Fehler, Funktionen können unvollständig sein oder ausfallen, das Programm kann abstürzen. Nimm es deshalb noch nicht für wichtige Arbeit.
   - **Haftungsausschluss** (blau): Die Software wird „wie besehen“ und ohne Gewährleistung bereitgestellt, die Nutzung erfolgt auf eigene Gefahr. Der Autor und die Mitwirkenden haften nicht für Datenverlust, beschädigte oder überschriebene Projekte, Disketten- und Speicherabbilder oder Schäden an angeschlossener Hardware und Emulatoren, soweit das Gesetz das zulässt. Sichere deine Projekte regelmäßig, zum Beispiel mit Git oder einer Datensicherung, besonders bevor du etwas auf echte Geräte überträgst.
3. Klicke auf **Verstanden, los geht’s**. Es geht auch mit **Enter**, der **Leertaste** oder **Esc**.
4. Mit dem Haken **Beim nächsten Start nicht mehr anzeigen** merkt sich VintageCoder deine Wahl, aber nur für die **installierte Version**. Sobald du eine neue Version installierst, erscheint der Startbildschirm wieder, damit du die aktuellen Hinweise noch einmal siehst. Ohne Haken erscheint er bei jedem Start.

> **Info:** Der Startbildschirm lädt nichts aus dem Netz. Dein Haken steht in der Datei `splash.json` im Konfigurationsordner (macOS `~/.config/vintagecoder`, Windows `%APPDATA%\vintagecoder`). Löschst du die Datei, erscheint der Startbildschirm wieder.

5. Beim **allerersten Start** (es gibt weder `settings.json` noch `wizard.json` im Konfigurationsordner) öffnet sich der **Konfigurationsassistent**; du siehst ihn, sobald du den Startbildschirm bestätigt hast. Im nächsten Abschnitt gehst du ihn durch. Danach siehst du die **Willkommensseite**, den Startpunkt, solange kein Projekt geöffnet ist.

![Die Willkommensseite: Knöpfe „Neues Projekt“, „Projekt öffnen“, „Beispiel öffnen“, die Liste der zuletzt geöffneten Projekte und rechts der Kasten „Toolchain und Hardware“](bild:willkommen-seite "Die Willkommensseite")

6. Oben stehen **Neues Projekt**, **Projekt öffnen** (Cmd/Strg+O) und **Beispiel öffnen**. Darunter folgen die Liste der **zuletzt geöffneten Projekte** (beim ersten Start leer) und die **Studios** für Grafik und Klang (gesperrt, solange kein Projekt offen ist).
7. Rechts zeigt der Kasten **Toolchain und Hardware** je Emulator einen grünen Punkt (gefunden) oder einen roten (fehlt), darunter die eingetragenen Maschinen. Der Knopf **Einstellungen öffnen** springt zu den Einstellungen, der Knopf **Konfigurationsassistent** startet den Assistenten erneut. Beim Start findet kein Netzwerkzugriff statt; Hardware steht deshalb auf „ungeprüft“, bis du **Testen** klickst.

> **Info:** Cmd bedeutet auf dem Mac, was unter Windows Strg ist. Im Handbuch steht deshalb **Cmd/Strg**.

> **Tipp:** Hattest du VintageCoder schon vor dem Assistenten benutzt (es gibt eine `settings.json`, aber noch keine `wizard.json`), öffnet er sich nicht von selbst. Dann bietet die Willkommensseite einmal einen Hinweis „Neu: Konfigurationsassistent“ an, den du annehmen oder wegklicken kannst.

## Der Konfigurationsassistent

Der Assistent bringt dich in wenigen Minuten von „frisch installiert“ bis „F5 startet mein Programm“. Er arbeitet nach drei Regeln:

- **Nichts wird gesendet.** Kein Netzwerkzugriff, keine Telemetrie. Er **installiert nichts** und lädt **nichts** herunter; für fehlende Programme zeigt er dir einen Befehl oder eine Webseite, aber ausführen musst du den Befehl selbst. Einen Browser öffnet er nur auf deinen Klick.
- **Er arbeitet auf einem Entwurf.** Alle Änderungen stehen zuerst im Speicher. Erst in der **Zusammenfassung** siehst du die Liste und bestätigst mit **Übernehmen**. Nur das **Design** (Farbschema, Größe, Schrift) wirkt sofort, damit du siehst, was du wählst.
- **Alles ist überspringbar und später änderbar.** **Überspringen** merkt sich den Schritt als offen, **Später fortsetzen** (oder **Esc**) beendet den Assistenten, ohne etwas zu schreiben. Er merkt sich dann nur, bei welchem Schritt du warst, nicht deine Eingaben.

Die Oberfläche des Assistenten ist **Deutsch**; eine Sprachwahl gibt es erst, wenn VintageCoder eine zweite Sprache hat.

### Den Weg wählen

Im ersten Schritt wählst du zwischen zwei Wegen:

![Der Assistent, Schritt 1: links die Schrittleiste, rechts die Karten „Schnelleinrichtung“ und „Alles selbst einstellen“](bild:assistent-weg-waehlen "Willkommen: zwei Wege zur Einrichtung")

| Weg | Schritte | Dauer | Wofür |
|---|---|---|---|
| **Schnelleinrichtung** (empfohlen) | 7 | etwa 2 Minuten | Design, Systeme, Toolchain und Projektordner; alles andere bleibt bei sinnvollen Voreinstellungen |
| **Alles selbst einstellen** | 12 | etwa 6 Minuten | zusätzlich Emulator-Parameter, ROMs, echte Hardware, Startziel für F5, Editor und erstes Projekt |

Links siehst du die **Schrittleiste** (1), darunter die **Fortschrittsleiste**. Mit **Weiter** (oder **Enter**) gehst du einen Schritt vor, mit **Zurück** (oder **Alt+Pfeil links**) einen zurück. Bereits erreichte Schritte kannst du in der Leiste direkt anklicken. In der Schnelleinrichtung steht links unten „Automatisch erledigt: Emulatoren, ROMs, Hardware, Startziel, Editor“; **Doch ansehen** wechselt zum langen Weg.

### Schnelleinrichtung, Schritt für Schritt

**Schritt 2: Design und Größe.** Wähle **Dunkel**, **Hell** oder **System** (folgt dem Betriebssystem, sonst dunkel). Zwei Regler stellen die Größe der **Oberfläche** und der **Editorschrift** ein. Eine Vorschau zeigt Assembler-Zeilen in deinem Design, und die **Lesbarkeitsprüfung** nennt für jede Farbe das Kontrastverhältnis. Die Schrift ist die eingebaute; weitere Schriften gibt es nicht.

![Der Schritt „Design und Größe“ mit den Karten Dunkel, Hell und System, den Reglern, der Vorschau und der Lesbarkeitsprüfung](bild:assistent-design "Design und Größe wirken sofort")

**Schritt 3: Zielsysteme.** Hake die Systeme an, mit denen du arbeiten willst: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE, MEGA65. Nur gewählte Systeme erscheinen später in der Oberfläche; abgewählte lassen sich wieder einblenden, ihre Einträge bleiben erhalten. Ein Chip zeigt je System, ob der passende Emulator gefunden wurde („VICE gefunden“) oder fehlt. Mindestens ein System muss gewählt sein, sonst sperrt der Assistent **Weiter**.

> **Info:** Die Pakete der Version 0.1.0 enthalten C64, C16/Plus/4, Atari und MEGA65. C128 und VC-20 sind in der Release-Auswahl abgewählt und erscheinen dort nicht (ein Bau aus dem Quelltext zeigt alles, siehe `docs/release-auswahl.md`).

![Der Schritt „Welche Systeme nutzt du?“ mit Kacheln je System und dem Chip „VICE gefunden“](bild:assistent-systeme "Zielsysteme wählen; die Chips zeigen die Emulator-Erkennung")

**Schritt 4: Toolchain.** Die Tabelle zeigt für deine Systeme, was gebraucht wird und was gefunden wurde: den eingebauten **Assembler** (kein Java, keine Zusatzinstallation) und den **Emulator** mit seinem Fundort. Zu jedem Programm gibt es **Details** (gefunden) oder einen **Hinweis** (fehlt) mit Reitern für macOS, Windows und Linux: einen Befehl zum Kopieren und eine Webseite, die sich auf Klick im Browser öffnet. **Pfad wählen …** trägt einen Emulator von Hand ein, **Erneut suchen** wiederholt die Suche, nachdem du etwas installiert hast. **Gesucht in** listet die Orte der Suche. Fehlt ein Emulator, kannst du mit **Ohne … fortfahren** weitermachen: Die betroffenen Systeme lassen sich dann bauen, aber nicht starten.

![Der Schritt „Toolchain“ mit der Programmtabelle, dem gefundenen VICE x64sc und dem Installationshinweis mit Reitern für macOS, Windows und Linux](bild:assistent-toolchain "Toolchain: was gefunden wurde, und Hilfe für das, was fehlt")

> **Info:** Auf dem Mac sucht die Erkennung im `PATH` und in den üblichen Ordnern (Homebrew, `/Applications`). Unter Windows und Linux sucht sie vor allem im `PATH`. Liegt dein Emulator woanders, wähle den Pfad mit **Pfad wählen …**. Unter macOS blockiert Gatekeeper nicht beglaubigte Programme beim ersten Start; dann hilft Rechtsklick ▸ **Öffnen**.

**Schritt 5: Projekte und Beispiele.** Das **Basisverzeichnis** ist der Ordner, in dem deine Projekte liegen; der Projekt-Assistent schlägt ihn als Speicherort vor. Ein Chip zeigt, ob der Ordner schon da ist oder neu angelegt wird. Das Ändern verschiebt nichts, es ändert nur die Voreinstellung für neue Projekte. Liegt der Ordner vermutlich auf einem Netzlaufwerk, warnt der Assistent, dass das langsam sein kann. Mit dem Haken **Mitgelieferte Beispiele dorthin kopieren** legt der Assistent die Beispielprojekte für deine Systeme nach `<Projektordner>/Beispiele`; kopiert wird erst bei **Übernehmen**, vorhandene Dateien bleiben unberührt.

![Der Schritt „Projekte und Beispiele“ mit Projektordner, Durchsuchen-Knopf und dem Haken zum Kopieren der Beispiele](bild:assistent-projekte "Projektordner und Beispiele")

**Schritt 6: Zusammenfassung.** Die **Änderungsliste** zeigt jede Änderung mit Haken: **Neu**, **Geändert**, **Entfernt**, darunter eingeklappt **Unverändert**. Jeden Eintrag kannst du abwählen; was nicht angehakt ist, wird nicht geschrieben. Rechts steht, **wohin** geschrieben wird (`settings.json` und `wizard.json` im Konfigurationsordner, bei Beispielen der Zielordner), und das **Sicherheitsnetz**: Vor dem Schreiben entsteht die Sicherung `settings.json.bak`, Passwörter bleiben verborgen, und das Schreiben ist atomar, bei einem Fehler bleibt alles wie vorher. Ohne Änderung heißt der Knopf **Ohne Änderung abschließen**.

![Die Zusammenfassung mit der Änderungsliste, den Kästen „Wird geschrieben nach“ und „Sicherheitsnetz“ und dem Knopf „Übernehmen“](bild:assistent-zusammenfassung "Die Änderungsliste: geschrieben wird erst nach „Übernehmen“")

**Schritt 7: Fertig.** Der Assistent zeigt die eingerichteten Systeme mit ihrem Stand („bereit“ oder „teilweise: Emulator fehlt, bauen geht“) und bietet vier Wege an: **Beispiel starten** (öffnet ein Beispiel, **F5** startet es), **Neues Projekt** (öffnet den Projekt-Assistenten), **Handbuch öffnen** und **Assistent erneut starten**. Der Kasten **Konfiguration sichern** exportiert deine Einstellungen (siehe unten). Mit **Rückgängig** stellst du Einstellungen und kopierte Beispiele wieder her, solange du VintageCoder nicht neu gestartet hast. **Fertig** schließt den Assistenten.

![Der letzte Schritt „Fertig“ mit den Karten Beispiel starten, Neues Projekt, Handbuch öffnen, Assistent erneut starten und dem Kasten „Konfiguration sichern“](bild:assistent-fertig "Fertig: wie es weitergeht")

> **Achtung:** Der Assistent baut und startet **nichts von selbst**. Ob die Kette aus Bauen und Emulator wirklich läuft, prüfst du im Abschnitt „Der Test: Hello World mit F5“.

### „Alles selbst einstellen“: die zusätzlichen Schritte

Der lange Weg hat zwölf Schritte. Die Schritte 1 bis 4 sowie Zusammenfassung und Fertig sind dieselben; dazwischen kommen:

| Schritt | Was du dort einstellst |
|---|---|
| **Emulatoren** | je System der Standard-Emulator, Pfad und Parameter (dieselbe Seite wie in den Einstellungen); **Test-Start** prüft, ob der Emulator läuft |
| **ROMs** | ROM-Dateien aus dem ROM-Ordner oder per Ablagezone; erkannt wird am Inhalt, ein Download findet nicht statt. Importierte Dateien werden **sofort** in den ROM-Ordner kopiert, nicht erst bei „Übernehmen“. Freie Ersatz-ROMs genügen oft |
| **Echte Hardware** | Ultimate 64 / II+ und MEGA65 eintragen (Haken „Ich habe echte Hardware anzuschließen“); Verbindungen werden nur auf Klick getestet. Unter Linux erklärt der Schritt, wie dein Benutzer in die Gruppe `dialout` kommt; den Befehl führst du selbst aus |
| **Startziel F5** | je System genau ein Ziel: der Standard-Emulator oder eine eingetragene Maschine. **F6** (Debuggen) startet immer im Emulator |
| **Editor** | Schrift und Adressspalte; bei der **Tastenbelegung** gibt es nur „Standard“, denn die **Tastenkürzel sind fest** und ein Anpassen ist noch nicht möglich |
| **Projekte** | wie in der Schnelleinrichtung, dazu das **erste Projekt**: ein Beispiel öffnen, ein leeres Projekt anlegen oder später |

> **Achtung:** Das Netzwerkpasswort einer Ultimate liegt in `settings.json` (nur für dich lesbar). Es erscheint nie im Klartext in der Änderungsliste und fehlt im Export, solange du es nicht ausdrücklich dazunimmst. Die Ablage im Schlüsselbund des Betriebssystems gibt es noch nicht.

### Den Assistenten später wieder aufrufen

Du kannst den Assistenten jederzeit erneut öffnen:

- Menü **Hilfe ▸ Konfigurationsassistent ▸ Alles einrichten …**,
- auf der Willkommensseite der Knopf **Konfigurationsassistent**,
- in den **Einstellungen** auf der Seite **Allgemein** im Abschnitt „Einrichtung“ der Knopf **Assistent starten**,
- in der Befehlspalette (Cmd/Strg+K) mit dem Wort „Konfigurationsassistent“,
- am Ende der Einrichtung mit **Assistent erneut starten**.

Der Wiederaufruf heißt **Konfiguration prüfen** und zeigt deinen gespeicherten Stand zuerst. Geändert wird nur, was du in der Zusammenfassung bestätigst.

![Der Assistent im Wiederaufruf: links Statusmarken in der Schrittleiste, rechts der gespeicherte Stand, die Liste der Probleme, die Teil-Assistenten und die Knöpfe zum Sichern](bild:assistent-wiederaufruf "Wiederaufruf: Statusmarken, Probleme und Teil-Assistenten")

- **Statusmarken** in der Schrittleiste: ein Haken heißt „unverändert“, ein Plus „neu“, eine Tilde „geändert“, ein Ausrufezeichen „Problem“, ein Strich „übersprungen“. Zusätzlich steht bei Schritten mit Neuem, Geändertem oder Problemen das Wort „neu“, „geändert“ oder „Problem“ am rechten Rand. Alle Schritte sind frei anklickbar.
- **Probleme im gespeicherten Stand** findet der Assistent selbst: ein Emulator-Pfad, den es nicht mehr gibt, ein fehlender ROM-Ort, eine Maschine ohne Adresse, ein Projektordner, der nicht existiert. **Schritt öffnen** springt zum passenden Schritt.
- **Neu gefunden** erscheint, wenn die Suche einen Emulator findet, den du noch nicht eingetragen hast. Nichts wird still übernommen: **Übernehmen** legt den Fund in den Entwurf, **Ignorieren** blendet ihn dauerhaft aus.
- Die **Teil-Assistenten** richten nur einen Teil ein und lassen alles andere nachweislich unverändert: **Nur ROMs**, **Nur Emulatoren**, **Nur echte Hardware**, **Nur Design und Editor**. Du erreichst sie auch direkt im Menü **Hilfe ▸ Konfigurationsassistent**.
- **Konfiguration exportieren …** schreibt die Einstellungen (Systeme, Emulatoren, Hardware, Darstellung) in eine Datei `vintagecoder-config.json`. Pfade im Projektordner stehen relativ darin; Passwörter nur, wenn du sie ausdrücklich dazunimmst.
- **Konfiguration importieren …** liest so eine Datei. Du siehst danach dieselbe Änderungsliste; geschrieben wird erst nach **Übernehmen**. Pfade, die es auf diesem Rechner nicht gibt, werden gemeldet.
- **Auf Standard zurücksetzen …** setzt einen Bereich zurück (Design, ROMs, Emulatoren, Hardware oder alles). Hardware-Einträge und Passwörter löschst du nur, wenn du es ausdrücklich ankreuzt.

Zwei Hinweise auf der Willkommensseite erinnern dich an offene Dinge: das Band „Einrichtung unvollständig: ein paar Schritte sind noch offen“ (Knopf **Fertig einrichten**), wenn du Schritte übersprungen oder mit **Später fortsetzen** verlassen hast, und „Neues erkannt“ (Knopf **Einrichtung prüfen**), wenn seit dem letzten Lauf ein Programm gefunden wurde. Mit ✕ blendest du sie aus.

### Was der Assistent nicht kann

- Er **installiert keine Programme** und lädt nichts herunter (auch keine ROMs).
- Er stellt **keine Tastenbelegung** ein; die Kürzel sind fest.
- Die Oberfläche ist **nur auf Deutsch**.
- Die Passwortablage im **Schlüsselbund** des Betriebssystems fehlt noch.
- Die automatische Emulator-Suche ist auf dem Mac am gründlichsten; unter Windows und Linux hilft **Pfad wählen …**.

> **Tipp:** Für Skripte und Testläufe unterdrückt die Umgebungsvariable `VC_NO_WIZARD=1` den automatischen Start des Assistenten.

## Von Hand einstellen: Emulator, ROMs und Hardware

Alles, was der Assistent einstellt, findest du auch in den **Einstellungen**. Die folgenden Abschnitte zeigen den manuellen Weg, zum Beispiel für Feinheiten oder wenn du die Einstellungen später ändern willst.

### Schritt für Schritt: Einen Emulator finden und einstellen

VintageCoder startet dein Programm in einem **Emulator**, einem eigenen Programm, das du selbst installierst. VintageCoder **lädt nichts herunter**.

| Zielsystem | Emulator | Hinweis |
|---|---|---|
| C64 | VICE `x64sc` (taktgenau) oder `x64` (schnell) | |
| C128 | VICE `x128` | |
| C16 / Plus/4 | VICE `xplus4` | |
| VC-20 | VICE `xvic` | |
| Atari 800 XL/XE | `atari800`, unter Windows auch Altirra | Altirra verwaltet seine Firmware selbst |
| MEGA65 | Xemu (`xmega65`) | braucht eine MEGA65-ROM, siehe unten |
| C64 (Hardware) | Ultimate 64 / II+ | kein Emulator, siehe unten |

1. **Emulator installieren.** Für den C64, C128, Plus/4 und VC-20 brauchst du VICE, für Atari `atari800` oder (unter Windows) Altirra, für den MEGA65 Xemu. Für den Debugger braucht VICE **Version 3.9 oder neuer**; unter Windows besser 3.10 oder neuer (VICE 3.9 stürzt dort beim Einzelschritt ab, Starten geht auch mit 3.9).
2. Öffne die **Einstellungen**: Menü **Datei ▸ Einstellungen …** (macOS: Menü „VintageCoder ▸ Einstellungen …“) oder **Cmd/Strg+Komma**.
3. Wechsle zur Seite **Zielsysteme und Hardware** und wähle links dein System. Ein grüner Punkt bei „Standard-Emulator“ heißt: gefunden. Rot heißt: nicht gefunden.
4. Rechts steht bei **Programm** der Pfad des Emulators. **Leer** heißt: VintageCoder sucht selbst im `PATH` und in den üblichen Ordnern. Der Knopf **Automatisch suchen** trägt den gefundenen Pfad ein. Findet er nichts, trägst du den Pfad von Hand ein. Ein Chip zeigt „vorhanden“ oder „nicht gefunden“.
5. Mit **▶ Test-Start** startest du den Emulator kurz zur Probe. Die Meldung zeigt, ob es geklappt hat.
6. Klicke unten auf **Änderungen anwenden**.

![Einstellungen, Seite „Zielsysteme und Hardware“: links die Systemliste mit grünen und roten Punkten, rechts der Standard-Emulator VICE x64sc mit Programm-Pfad, „Automatisch suchen“, „Test-Start“ und dem Chip „vorhanden“](bild:einstellungen-emulator "Einstellungen: Emulator-Pfad prüfen und mit „Test-Start“ ausprobieren")

**Wo VintageCoder automatisch sucht:** im `PATH` und zusätzlich zum Beispiel unter macOS in `/Applications/vice-arm64-gtk3-3.10/bin`, `/Applications/vice-arm64-gtk3-3.9/bin`, `/opt/homebrew/bin` und `/usr/local/bin` (Xemu auch unter `/Applications/xmega65.app/Contents/MacOS`). Liegt dein Emulator woanders, trage den Pfad ein. Für Xemu geht auch die Umgebungsvariable `VC_XEMU`.

Einige Besonderheiten:

- Ein System kann **mehrere Emulatoren** haben; einer ist der **Standard** („Als Standard setzen“). **Weiteren Emulator hinzufügen** bietet die Sorten des Systems an.
- **Parameter** hängt VintageCoder an die Kommandozeile an (siehe [Bauen und Starten](kapitel:bauen-starten)).
- **atari800** kann ein eigenes **OS-ROM** bekommen, **Xemu** den **Ordner mit `MEGA65.ROM`**. Im Normalfall legst du die ROMs aber im ROM-Ordner ab (nächster Abschnitt).
- **Startziel (F5)** bestimmt, wohin F5 und F6 starten: der Standard-Emulator oder eine eingetragene Maschine.

#### Ultimate 64 und MEGA65 (echte Hardware)

Unter **Physische Zielhardware & Netzwerkgeräte** trägst du mit **+ Hardware hinzufügen** eine **Ultimate 64 / II+** (Adresse wie `192.168.1.50`, optional Netzwerkpasswort) oder einen **MEGA65** (USB oder Ethernet) ein. **Verbindung testen** prüft, ob die Maschine antwortet. Einzelheiten und der Ablauf stehen im Kapitel [Bauen und Starten](kapitel:bauen-starten).

### ROM-Dateien: brauchst du sie?

Ein **ROM** ist der fest eingebaute Speicher eines alten Computers: Betriebssystem, BASIC, Zeichensatz. VintageCoder liefert **keine Original-ROMs** mit und lädt nichts aus dem Netz. Wer sie braucht, legt seine **eigenen Abbilder** (Dateien) in den ROM-Ordner. VintageCoder erkennt sie am **Inhalt** (Größe und Prüfsumme), der Dateiname ist egal. Ob du ein Abbild benutzen darfst, klärst du selbst.

| Wofür | Ohne ROM | Mit ROM |
|---|---|---|
| VICE, atari800 | laufen mit eigenen Ersatz-ROMs | nutzen deine Abbilder |
| Xemu (MEGA65) | startet **nicht**: braucht eine `MEGA65.ROM` von genau 131072 Byte | läuft |
| Zeichensatz-Editor, Zeichenwähler | eingebaute Ersatzschrift ohne Grafikzeichen, mit Hinweis | Original-Zeichensatz deines Systems |

Für das Hello-World-Programm mit VICE brauchst du **keine** ROMs.

#### Schritt für Schritt: ROMs einrichten

1. Klicke auf der Willkommensseite im Kasten für die ROMs auf **ROMs einrichten …**. Den Assistenten findest du auch im Menü **Hilfe ▸ ROMs einrichten …** und in der Befehlspalette (Cmd/Strg+K).
2. Der Assistent hat drei Schritte: **1 Systeme wählen**, **2 Dateien ablegen** (ziehe deine ROM-Dateien oder ZIP-Archive in das Fenster, oder lege sie in den ROM-Ordner; **Ordner öffnen** zeigt ihn) und **3 Ergebnis**.
3. Unter **Einstellungen ▸ ROMs** siehst du je System und Rolle (zum Beispiel `kernal`, `basic`, `chargen` beim C64), ob eine passende Datei gefunden wurde. Dort bieten **Ordner öffnen**, **Neu prüfen** und **Auswählen …** weitere Wege. Haken bei **ROMs anderer Programme mitbenutzen** (Vorgabe aus) lässt VintageCoder auch die ROM-Ordner von VICE, Xemu und atari800 lesen.

![Der Assistent „ROMs einrichten“ mit der Schrittleiste (1 Systeme wählen, 2 Dateien ablegen, 3 Ergebnis) und der Ablagezone für ROM-Dateien](bild:roms-einrichten "Der ROM-Assistent: Dateien ablegen, Erkennung am Inhalt")

![Einstellungen, Seite „ROMs“: je System und Rolle der Status der gefundenen Dateien, Knöpfe „Ordner öffnen“, „Neu prüfen“ und „Auswählen …“](bild:einstellungen-roms "Einstellungen ▸ ROMs: welche Rollen sind gefunden?")

**Wo die ROMs liegen:** im Benutzerordner `roms` (macOS `~/.config/vintagecoder/roms`, Windows `%APPDATA%\vintagecoder\roms`), je System in einem Unterordner (`c64`, `c128`, `vic20`, `plus4`, `atari8`, `mega65`). Auf Wunsch gilt auch ein Ordner `roms/` im Projekt oder neben dem Programm (portabler Modus). Die vollständige Reihenfolge steht in der Hilfeseite [ROM-Dateien](hilfe:roms). Auf der Kommandozeile zeigt `vcc roms list`, welche Rollen fehlen.

## Das Kommandozeilenwerkzeug vcc

Neben der Oberfläche gehört das Werkzeug **`vcc`** zum Paket. Du brauchst es nicht für den Alltag, aber es baut Projekte ohne Oberfläche (zum Beispiel für Skripte). Wichtige Befehle:

```bash
vcc new hallo --target c64 --template asm-hello   # neues Projekt anlegen
vcc build hallo                                   # Projekt bauen
vcc new --list                                    # Vorlagen und Projektstrukturen
vcc roms list                                     # ROM-Rollen und ihr Status
```

Unter Windows legt das Setup `vcc` nur in den `PATH`, wenn du es dort anhakst; unter macOS verlinkt es **Kommandozeile installieren.command**.

## Beispielprojekte

VintageCoder bringt fertige Beispiele mit. Auf der Willkommensseite öffnet **Beispiel öffnen** den Ordner `examples/` neben der Hardware-Datenbank; wähle darin die Projektdatei (`.vcprj`) eines Beispiels. Unter Windows kopiert das Setup die Beispiele nach „Dokumente\VintageCoder\Beispiele“, auf dem Mac liegt der Ordner **Beispiele** im .dmg. Es gibt Beispiele für alle Systeme, zum Beispiel `c64/border`, `atari/colorbars` und `mega65/hello`.

## Der Test: Hello World mit F5

Zum Schluss prüfst du, dass die ganze Kette läuft.

1. Klicke auf der Willkommensseite auf **Neues Projekt** (oder drücke Cmd/Strg+Umschalt+N). Hast du im Assistenten „Neues Projekt“ gewählt, öffnet sich der Projekt-Assistent schon.
2. Wähle das Zielsystem **Commodore 64** und die Vorlage **Assembler (vcasm)** ▸ **Assembler-Programm**. Gib dem Projekt einen Namen wie `hallo` und wähle einen Speicherort. Klicke auf **Projekt erstellen und öffnen**.

![Der Projekt-Assistent, Spalte 3 „Projekt-Details“ mit Projektname, Speicherort, Videonorm, Emulator, Startziel und Diskette](bild:wizard-details "Der Projekt-Assistent: Zielsystem, Vorlage und Details")

3. Drücke **F5**. VintageCoder baut das Programm und startet VICE. Im Emulator läuft der Rahmen in bunten Streifen durch die Farben.
4. Schließe den Emulator.

Klappt das, ist die Einrichtung fertig. Hast du im Assistenten die Beispiele kopieren lassen, geht es auch mit einem Beispiel: **Beispiel öffnen**, ein `.vcprj` wählen, **F5**. Eine ausführliche Anleitung mit Bildern steht in [Dein erstes Projekt](kapitel:erstes-projekt).

## Häufige Stolpersteine

- **Der Assistent ist verschwunden oder hat sich nicht geöffnet.** Er öffnet sich nur beim allerersten Start von selbst. Ruf ihn über **Hilfe ▸ Konfigurationsassistent** oder den Knopf auf der Willkommensseite auf.
- **Der Startbildschirm kommt immer wieder.** Hake **Beim nächsten Start nicht mehr anzeigen** an; nach einem Versionswechsel erscheint er einmal wieder.
- **Der Assistent meldet „Die Einstellungsdatei ist nicht lesbar“.** Er überschreibt sie nie ohne Sicherung; **Neu beginnen** legt zuerst eine Sicherung der alten Datei an.
- **Emulator rot in den Einstellungen.** Er ist nicht installiert oder liegt an einem ungewöhnlichen Ort. Installiere ihn und klicke **Automatisch suchen**, oder trage den Pfad ein. 
- **macOS verweigert das Öffnen.** Rechtsklick auf die App und **Öffnen** (Gatekeeper-Hinweis oben).
- **`vcc` wird im Terminal nicht gefunden.** Windows: Haken „zum PATH hinzufügen“ im Setup nachholen (Setup erneut ausführen) und eine neue Eingabeaufforderung öffnen. macOS: **Kommandozeile installieren.command** ausführen.
- **Der MEGA65-Emulator startet nicht.** Xemu braucht eine `MEGA65.ROM` von genau 131072 Byte.
- **Der Zeichensatz-Editor zeigt eine Ersatzschrift.** Es fehlt das Zeichensatz-ROM deines Systems; richte es mit dem ROM-Assistenten ein.
- **Windows-Debugger stürzt ab.** VICE 3.9 stürzt beim Einzelschritt ab; nimm 3.10 oder neuer.

## Wie geht es weiter?

Weiter geht es mit [Willkommen bei VintageCoder](kapitel:willkommen), falls du einen Überblick möchtest, oder direkt mit [Dein erstes Projekt](kapitel:erstes-projekt). Die Oberfläche erklärt [Die Oberfläche im Überblick](kapitel:oberflaeche).
