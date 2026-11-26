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

> **Info:** Das Setup-Programm ist unter Windows 11 (x86_64) geprüft: Installation, Start und Deinstallation. Windows 10 ist nicht geprüft; 32-Bit-Windows und Windows auf ARM werden nicht unterstützt.

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

> **Info:** Die Linux-Pakete wurden in frischen Containern der jeweiligen Distribution (Installation, `vcc`, Start der Oberfläche mit Software-Darstellung) und zusätzlich auf einer Ubuntu-24.04-Testmaschine mit Desktop (virtuelle Maschine) geprüft; dort liefen die Menüleiste, die Alt-Kürzel, der Dateidialog und der Papierkorb. **Ton** und eine **Wayland-Sitzung** sind noch nicht geprüft.

## Der erste Start

1. Starte VintageCoder. Als Erstes erscheint kurz das **Startbild**: das Logo mit dem Namen und dem Satz „Assembler-Entwicklung für 8-Bit-Computer“. Ein Klick oder eine beliebige Taste schließt es; sonst verschwindet es nach etwa einer Sekunde von selbst.
2. **Hinweis:** VintageCoder enthält wahrscheinlich noch viele Fehler. Es gibt keine Gewährleistung (siehe Lizenz). Sichere deine Projekte regelmäßig, zum Beispiel mit Git oder einer Datensicherung, besonders bevor du etwas auf echte Geräte überträgst.
3. Das Startbild lässt sich abschalten: **Einstellungen ▸ Allgemein ▸ Startbild**, Schalter **Startbild beim Programmstart zeigen**. Die Wahl gilt ab dem nächsten Start.
4. Beim **allerersten Start** (es gibt weder `settings.json` noch `wizard.json` im Konfigurationsordner) öffnet sich der **Konfigurationsassistent** über der Oberfläche. Im nächsten Abschnitt gehst du ihn durch. Danach siehst du die **Startseite**, den Startpunkt, solange kein Projekt offen ist.

![Die Startseite ohne offenes Projekt: der Titel VintageCoder, die Karten Neues Projekt, Projekt öffnen und Beispiel öffnen, der Link Projektdatei (.vcprj) wählen, der Abschnitt Zuletzt geöffnete Projekte (noch leer) und die Beispielprojekte mit Filtern je System und Karten mit Öffnen und Kopieren.](bild:willkommen-seite "Die Startseite erscheint, solange kein Projekt offen ist.")

5. Oben stehen die Karten **Neues Projekt**, **Projekt öffnen** (Cmd/Strg+Umschalt+O) und **Beispiel öffnen** (öffnet einen Dialog mit den mitgelieferten Beispielen und einem Suchfeld). Darunter folgen der Link **Projektdatei (.vcprj) wählen …**, die Liste der **zuletzt geöffneten Projekte** (beim ersten Start leer) und die **Beispielprojekte** mit Filter je System.
6. Unten zeigt der Kasten **Toolchain und Hardware** je Emulator einen grünen Punkt mit „bereit“ (gefunden) oder einen grauen mit „nicht gefunden“. Daneben steht die **Schnellreferenz** mit den wichtigsten Tastenkürzeln. Solange du noch nichts eingerichtet hast, steht an der Stelle des Kastens die Karte **Willkommen bei VintageCoder** mit **Einrichtung starten** und **Später**. Beim Start findet kein Netzwerkzugriff statt.

> **Info:** Cmd bedeutet auf dem Mac, was unter Windows Strg ist. Im Handbuch steht deshalb **Cmd/Strg**. Die Startseite erreichst du später mit dem Befehl **Startseite** (Menü **Datei**, auch in der Befehlspalette).

> **Info:** Die Menüleiste mit **Datei**, **Bearbeiten**, **Ansicht**, **Projekt** und **Hilfe** gibt es auf allen Systemen: auf dem Mac oben am Bildschirmrand, sonst im Fenster. Alle Befehle erreichst du außerdem über die Befehlspalette (**Cmd/Strg+K**), die Kürzel, die Kopfzeile und die Kontextmenüs. Den Bereich **Einstellungen** öffnest du mit dem Zahnrad ⚙ in der Kopfzeile oder mit **Cmd/Strg+Komma**.

> **Tipp:** Hast du die Karte **Willkommen bei VintageCoder** mit **Später** ausgeblendet, bleibt sie weg. Die Einrichtung findest du jederzeit unter **Einstellungen ▸ Allgemein ▸ Einrichtung** und im Menü **Hilfe**.

## Der Konfigurationsassistent

Der Assistent bringt dich in wenigen Minuten von „frisch installiert“ bis „F5 startet mein Programm“. Er arbeitet nach drei Regeln:

- **Nichts wird gesendet.** Kein Netzwerkzugriff, keine Telemetrie. Er **installiert nichts** und lädt **nichts** herunter; er zeigt nur, was auf deinem Rechner gefunden wurde. Eine Ausnahme gibt es: Im Schritt **Emulatoren** (langer Weg) zeigt der Assistent dieselbe Seite wie die Einstellungen. Fehlt dort ein Emulator, kann sie eine portable Installation anbieten; geladen wird erst nach deinem Klick auf **Jetzt laden**.
- **Er arbeitet auf einem Entwurf.** Alle Änderungen stehen zuerst im Speicher. Erst in der **Zusammenfassung** siehst du die Liste und bestätigst mit **Übernehmen**. Nur das **Design** (Farbschema, Zoom, Schrift) wirkt sofort, damit du siehst, was du wählst; **Abbrechen** stellt das Alte wieder her.
- **Du kannst jederzeit aufhören.** **Abbrechen** beendet den Assistenten, ohne Einstellungen zu ändern. **Später fortsetzen** tut dasselbe und merkt sich, bei welchem Schritt du warst, nicht deine Eingaben.

Links stehen die **Schritte** mit ihren Gruppen. Unten stehen **Abbrechen**, **Später fortsetzen**, **Zurück** und **Weiter**. Mit **Esc** brichst du ab.

### Den Weg wählen

Der erste Schritt, **Willkommen**, hat oben die **Sprache** (*Automatisch*, *Deutsch*, *English*; Englisch ist maschinell übersetzt und noch nicht geprüft) und bietet zwei Wege:

| Weg | Schritte | Wofür |
|---|---|---|
| **Schnelleinrichtung (7 Schritte)** | 7 | Design, Zielsysteme, Toolchain und Projekte; Emulatoren, ROMs und Hardware werden automatisch gefunden und später in den Einstellungen verfeinert |
| **Alles selbst einstellen (12 Schritte)** | 12 | zusätzlich Emulatoren, ROMs, echte Hardware, Startziel F5 und Editor |
| **Nur einen Teil** | 3 bis 4 | die Teil-Assistenten **ROMs …**, **Emulatoren …**, **Hardware …** und **Darstellung und Editor …** richten nur diesen Teil ein |

![Erster Schritt „1. Willkommen“ des Einrichtungsassistenten: links die Schrittleiste, rechts die Sprachwahl „Automatisch“, „Deutsch“, „English“ und die Wahl zwischen „Schnelleinrichtung (7 Schritte)“ (gewählt) und „Alles selbst einstellen (12 Schritte)“.](bild:assistent-weg-waehlen "Der Willkommen-Schritt: Sprache wählen und zwischen Schnelleinrichtung und „Alles selbst einstellen“ entscheiden.")

Mit **Weiter** gehst du einen Schritt vor, mit **Zurück** einen zurück. Bereits erreichte Schritte kannst du in der Liste links direkt anklicken. Hast du den Assistenten schon einmal unvollständig verlassen, steht auf dieser Seite „Die Einrichtung ist noch nicht abgeschlossen.“

### Schnelleinrichtung, Schritt für Schritt

**Schritt 2: Design.** Wähle **System** (folgt dem Betriebssystem), **Dunkel** oder **Hell**. Zwei Knopfpaare **−** und **+** stellen den **Zoom** der Oberfläche (in Prozent) und die **Editor-Schrift** (in Punkt) ein. Alles wirkt sofort.

**Schritt 3: Zielsysteme.** Hake die Systeme an, mit denen du arbeiten willst: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE, MEGA65. Nur gewählte Systeme erscheinen später in der Oberfläche; abgewählte werden nur ausgeblendet, nichts geht verloren.

> **Info:** Der Umfang dieser Version sind C64, C128, C16/Plus/4, VC-20, Atari 8 Bit und MEGA65. Der Commodore PET/CBM gehört nicht dazu.

**Schritt 4: Toolchain.** Die Seite nennt den eingebauten **Assembler** („Eingebaut (vc-asm). Kein weiteres Programm nötig.“; kein Java, keine Zusatzinstallation) und zeigt darunter dieselbe Seite wie **Einstellungen ▸ Ziele und Hardware**: für jeden Emulator, ob er gefunden wurde („Gefunden: …“) oder fehlt („Nicht gefunden. Pfad von Hand eintragen.“). Gesucht wird im `PATH` und in den üblichen Ordnern. Fehlende Emulatoren trägst du hier, im Schritt **Emulatoren** (langer Weg) oder später in den Einstellungen ein. Ohne Emulator lassen sich die betroffenen Systeme bauen, aber nicht starten.

> **Info:** Auf dem Mac sucht die Erkennung im `PATH` und in den üblichen Ordnern (Homebrew, `/Applications`). Unter Windows und Linux sucht sie vor allem im `PATH`. Liegt dein Emulator woanders, trägst du den Pfad in den Einstellungen ein (**Ziele und Hardware**, Feld **Programm**). Unter macOS blockiert Gatekeeper nicht beglaubigte Programme beim ersten Start; dann hilft Rechtsklick ▸ **Öffnen**.

**Schritt 5: Projekte.** Der **Projektordner** ist der Ordner, in dem deine Projekte liegen; **Neues Projekt** schlägt ihn als Speicherort vor. Leer heißt: `~/VintageCoder`. Das Ändern verschiebt nichts, es ändert nur die Voreinstellung für neue Projekte. Mit dem Schalter **Beispielprojekte der gewählten Systeme kopieren** legt der Assistent die Beispielprojekte für deine Systeme in den Projektordner (Unterordner „Beispiele“); du kannst einzelne Beispiele abwählen. Kopiert wird erst bei **Übernehmen**, vorhandene Dateien bleiben unberührt. Liegen in deiner Installation keine Beispiele, steht dort „In dieser Installation liegen keine Beispiele.“

**Schritt 6: Zusammenfassung.** Die Liste zeigt jede Änderung mit Haken, dem Bereich und der Art (+ neu, ~ geändert, − entfernt) sowie dem alten und dem neuen Wert. Jeden Eintrag kannst du abwählen; was nicht angehakt ist, wird nicht geschrieben. Vor dem Schreiben entsteht eine Sicherung der Einstellungen. Gibt es nichts zu ändern, steht dort „Es gibt nichts zu ändern. Alles steht schon so.“ **Übernehmen** schreibt.

![Einrichtungsassistent, Schritt „6. Zusammenfassung“. Eine Karte nennt, was geändert wird: unter „SYSTEM“ das Zielsystem Commodore 64 mit VICE x64sc und unter „PROJEKTE“ das Kopieren von 15 Beispielen, jeweils mit gesetztem Haken. Unten rechts stehen „Zurück“ und „Übernehmen“.](bild:assistent-zusammenfassung "Die Zusammenfassung listet jede Änderung mit Haken, geschrieben wird erst nach „Übernehmen“.")

**Schritt 7: Fertig.** Der Assistent meldet „Übernommen.“ und zeigt den Ort der Sicherung, die Zahl der kopierten Beispiele und Hinweise. Mit **Rückgängig** stellst du Einstellungen und kopierte Beispiele wieder her. Unter **Erstes Projekt** legst du mit **Neues Projekt anlegen …** gleich ein Projekt an (der Dialog **Neues Projekt erstellen** öffnet sich) oder öffnest mit **Beispiel öffnen: …** ein kopiertes Beispiel. **Schließen** beendet den Assistenten.

> **Achtung:** Der Assistent baut und startet **nichts von selbst**. Ob die Kette aus Bauen und Emulator wirklich läuft, prüfst du im Abschnitt „Der Test: Hello World mit F5“.

### „Alles selbst einstellen“: die zusätzlichen Schritte

Der lange Weg hat zwölf Schritte: Willkommen, Design, Zielsysteme, Toolchain, **Emulatoren**, **ROMs**, **Echte Hardware**, **Startziel F5**, **Editor**, Projekte, Zusammenfassung, Fertig. Die Schritte dazwischen sind:

| Schritt | Was du dort einstellst |
|---|---|
| **Emulatoren** | je System der Standard-Emulator, Pfad und Startparameter (dieselbe Seite wie **Ziele und Hardware** in den Einstellungen) |
| **ROMs** | dieselbe Seite wie **Einstellungen ▸ ROMs**: ROM-Dateien oder ZIP-Archive wählen oder auf das Fenster ziehen; erkannt wird am Inhalt, ein Download findet nicht statt. Dateien kommen erst nach deiner Bestätigung in den ROM-Ordner. Freie Ersatz-ROMs genügen oft |
| **Echte Hardware** | Maschinen (zum Beispiel Ultimate 64 / II+) eintragen; Verbindungen werden nur auf Klick getestet |
| **Startziel F5** | je System genau ein Ziel: der Standard-Emulator oder eine eingetragene Maschine; F5 und F6 starten dorthin |
| **Editor** | Editor-Schrift, **Zeilenumbruch** und **Zeile beim Verlassen formatieren** (die Seite **Editor** der Einstellungen). Die **Tastenkürzel sind fest**; ein Anpassen ist nicht möglich |

> **Achtung:** Das Netzwerkpasswort einer Ultimate liegt in `settings.json` (nur für dich lesbar). Die Ablage im Schlüsselbund des Betriebssystems gibt es noch nicht.

### Den Assistenten später wieder aufrufen

Du kannst den Assistenten jederzeit erneut öffnen:

- Menü **Hilfe ▸ Konfigurationsassistent …**,
- in den **Einstellungen** auf der Seite **Allgemein** in der Karte **Einrichtung** (**Schnelleinrichtung (7 Schritte)**, **Alles selbst einstellen (12 Schritte)** oder ein Teil),
- auf der Startseite mit **Einrichtung starten**, solange die Karte **Willkommen bei VintageCoder** zu sehen ist,
- in der Befehlspalette (Cmd/Strg+K) mit dem Wort „Konfigurationsassistent“.

Die **Teil-Assistenten** richten nur einen Teil ein: **Konfigurationsassistent: nur ROMs …**, **nur Emulatoren …**, **nur echte Hardware …** und **nur Design und Editor …**. **Konfigurationsassistent: Nach Neuem suchen …** öffnet den Teil für die ROMs. Du erreichst sie im Menü **Hilfe**, in der Befehlspalette und in der Karte **Einrichtung** der Einstellungen. Hat VintageCoder seit der letzten Einrichtung einen neuen Emulator gefunden, zeigt die Startseite die Karte **Neues erkannt** mit **Emulatoren einrichten**.

**Konfiguration exportieren, importieren und zurücksetzen** gehören nicht zum Assistenten. Du findest sie unter **Einstellungen ▸ Sicherung**: **Konfiguration exportieren …** schreibt die Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen, Hilfe- und Update-Wahl) in eine JSON-Datei, ohne Passwörter. **Konfiguration importieren …** zeigt vorher, was drinsteht, und ersetzt dann die jetzigen Einstellungen. **Auf Vorgabe zurücksetzen …** stellt die Vorgabe her. Farbschema und Zoom gehören nicht dazu.

### Was der Assistent nicht kann

- Er **installiert keine Programme** und lädt nichts herunter (auch keine ROMs).
- Er stellt **keine Tastenbelegung** ein; die Kürzel sind fest.
- Die englische Oberfläche ist maschinell übersetzt und noch nicht geprüft.
- Die Passwortablage im **Schlüsselbund** des Betriebssystems fehlt noch.
- Die automatische Emulator-Suche ist auf dem Mac am gründlichsten; unter Windows und Linux hilft das Feld **Programm** in den Einstellungen.

> **Tipp:** Für Skripte und Testläufe unterdrückt die Umgebungsvariable `VC_NO_WIZARD=1` den automatischen Start des Assistenten.

## Von Hand einstellen: Emulator, ROMs und Hardware

Alles, was der Assistent einstellt, findest du auch in den **Einstellungen**. Die folgenden Abschnitte zeigen den manuellen Weg, zum Beispiel für Feinheiten oder wenn du die Einstellungen später ändern willst.

### Schritt für Schritt: Einen Emulator finden und einstellen

VintageCoder startet dein Programm in einem **Emulator**, einem eigenen Programm, das du selbst installierst. VintageCoder bringt keinen Emulator mit und lädt nie ungefragt etwas: Wo es für dein System ein festes Paket gibt, bietet die Karte des Emulators in den Einstellungen den Knopf **Portabel installieren (Version) …** an; geladen wird erst nach deiner Bestätigung mit **Jetzt laden** (mit Prüfung der SHA-256-Summe). Unter Linux siehe auch [Emulatoren unter Linux](kapitel:systeme).

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
2. Öffne die **Einstellungen**: mit dem Zahnrad ⚙ in der Kopfzeile, mit **Cmd/Strg+Komma** oder im Menü **Ansicht ▸ Einstellungen**.
3. Wechsle zur Seite **Ziele und Hardware** und wähle links dein System unter **Zielsysteme**. Fehlt es in der Liste, nimmst du es mit **System hinzufügen …** und **Hinzufügen** auf.
4. Rechts steht in der Karte des Emulators bei **Programm (leer = automatisch suchen)** der Pfad des Programms. **Leer** heißt: VintageCoder sucht selbst im `PATH` und in den üblichen Ordnern. Der Knopf **Suchen** trägt den gefundenen Pfad ein. Findet er nichts, trägst du den Pfad von Hand ein. Darunter zeigt eine Zeile mit ✓ oder ✗, ob der Emulator gefunden wurde („Gefunden: …“, bei der automatischen Suche mit dem Pfad, oder „Nicht gefunden. Pfad von Hand eintragen.“).
5. Mit **Testen** startest du den Emulator kurz zur Probe. Die Meldung zeigt, ob es geklappt hat.
6. Klicke unten auf **Speichern**. **Verwerfen** nimmt deine Änderungen zurück. Daneben steht „Ungespeicherte Änderungen“, solange etwas offen ist.

![Einstellungen, Seite „Ziele und Hardware“ mit gewähltem System Commodore 64. Die Karte „Emulatoren“ zeigt VICE x64sc (taktgenau) als Standard, das Programmfeld mit „automatisch“, die Knöpfe „Suchen“ und „Testen“ und die grüne Meldung „Gefunden“ mit dem Pfad. Darunter stehen „Startparameter“ (eingeklappt), „Entfernen“, „Emulator hinzufügen ...“ und unter „ECHTE HARDWARE“ der Knopf „Ultimate 64 / II+ (Netzwerk) anbinden“.](bild:einstellungen-emulator "Die Emulator-Karte: Programm suchen und mit „Testen“ prüfen.")

**Wo VintageCoder automatisch sucht:** im `PATH` und zusätzlich zum Beispiel unter macOS in `/Applications/vice-arm64-gtk3-3.10/bin`, `/Applications/vice-arm64-gtk3-3.9/bin`, `/opt/homebrew/bin` und `/usr/local/bin` (Xemu auch unter `/Applications/xmega65.app/Contents/MacOS`). Liegt dein Emulator woanders, trage den Pfad ein. Für Xemu geht auch die Umgebungsvariable `VC_XEMU`.

Einige Besonderheiten:

- Ein System kann **mehrere Emulatoren** haben; einer ist der **Standard** (Auswahlknopf **Standard**). Mit **Emulator hinzufügen …** und **Hinzufügen** kommt ein weiterer der gewählten Art dazu.
- **Startparameter** (Karte **Argumente**) hängt VintageCoder an die Kommandozeile an (siehe [Bauen und Starten](kapitel:bauen-starten)).
- **atari800** kann ein eigenes **OS-ROM** bekommen, **Xemu** den **Ordner mit `MEGA65.ROM`** (Feld **ROM**). Im Normalfall legst du die ROMs aber im ROM-Ordner ab (nächster Abschnitt).
- Die Karte **Startziel** bestimmt, wohin F5 und F6 starten: **Emulator (Standard des Systems)** oder eine eingetragene Maschine.

#### Ultimate 64 (echte Hardware)

Unter **Echte Hardware** trägst du mit dem Knopf **Ultimate 64 / II+ anbinden** eine Maschine ein: **Adresse** (wie `192.168.1.50`) und, falls in der Firmware eingestellt, das **Netzwerkpasswort**. **Verbindung testen** prüft, ob sie antwortet; das geschieht nur auf Klick. Die Anbindung an einen echten MEGA65 (USB oder Ethernet) ist in den Paketen dieser Version ausgeblendet. Einzelheiten und der Ablauf stehen im Kapitel [Bauen und Starten](kapitel:bauen-starten).

### ROM-Dateien: brauchst du sie?

Ein **ROM** ist der fest eingebaute Speicher eines alten Computers: Betriebssystem, BASIC, Zeichensatz. VintageCoder liefert **keine Original-ROMs** mit und lädt nichts aus dem Netz. Wer sie braucht, legt seine **eigenen Abbilder** (Dateien) in den ROM-Ordner. VintageCoder erkennt sie am **Inhalt** (Größe und Prüfsumme), der Dateiname ist egal. Ob du ein Abbild benutzen darfst, klärst du selbst.

| Wofür | Ohne ROM | Mit ROM |
|---|---|---|
| VICE, atari800 | laufen mit eigenen Ersatz-ROMs | nutzen deine Abbilder |
| Xemu (MEGA65) | startet **nicht**: braucht eine `MEGA65.ROM` von genau 131072 Byte | läuft |
| Zeichensatz-Editor, Zeichenwähler | der eigene Standardsatz von VintageCoder (VC-Standard), mit Hinweis | Original-Zeichensatz deines Systems |

Für das Hello-World-Programm mit VICE brauchst du **keine** ROMs.

#### Schritt für Schritt: ROMs einrichten

1. Starte den Teil-Assistenten über **Hilfe ▸ Konfigurationsassistent: nur ROMs …**, über die Befehlspalette (Cmd/Strg+K, Wort „ROMs“) oder über **Einstellungen ▸ Allgemein ▸ Einrichtung ▸ ROMs …**.
2. Der Teil-Assistent hat drei Schritte: **Zielsysteme**, **ROMs** und **Zusammenfassung**. Im Schritt **ROMs** wählst du deine ROM-Dateien oder ZIP-Archive mit **ROM-Datei oder Zip wählen …** oder ziehst sie auf das Fenster; eine Liste zeigt, was erkannt wurde, und erst **n übernehmen** kopiert die angehakten Dateien in den ROM-Ordner. **ROM-Ordner öffnen** zeigt den Ordner.
3. Dieselbe Seite findest du unter **Einstellungen ▸ ROMs**. Sie zeigt je System (Reiter mit Ampel) und Rolle (zum Beispiel KERNAL, BASIC, Zeichensatz beim C64), ob eine passende Datei gefunden wurde, und bietet **ROM-Ordner öffnen**, **Neu prüfen**, **ROM-Datei oder Zip wählen …** und **ROMs anderer Programme suchen**. Der Schalter **ROMs anderer Programme mitbenutzen (VICE, Xemu, atari800)** (Vorgabe aus) lässt VintageCoder auch die ROM-Ordner dieser Programme lesen. Unten stehen die **Durchsuchten Orte**.

![Einstellungen, Seite ROMs. Links die Liste der Seiten, rechts oben die Knöpfe ROM-Ordner öffnen und Neu prüfen, darunter ROM-Datei oder Zip wählen, ROMs anderer Programme suchen und Fundliste kopieren. Reiter je System (C64 gewählt) und je Rolle der Status Fehlt mit erwarteter Größe, unten die durchsuchten Orte.](bild:einstellungen-roms "Die Seite ROMs zeigt je System und Rolle, ob eine Datei gefunden wurde; hier fehlt noch alles.")

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

## Neue Versionen finden

VintageCoder kann nachsehen, ob es eine neue Version gibt. **Ohne deine Zustimmung geschieht das nie:** Solange du unter **Einstellungen ▸ Allgemein ▸ Updates** nichts gewählt hast, steht dort *Noch nicht gefragt* und nichts wird gesendet. Du kannst *Nur auf Wunsch*, *Täglich* oder *Wöchentlich* wählen (täglich und wöchentlich prüft VintageCoder beim Start) und mit **Speichern** übernehmen.

Gibt es eine neuere Version, steht in der Statusleiste **Update X verfügbar**. Ein Klick öffnet **Einstellungen ▸ Info und Update** mit den **Änderungen** seit deiner Version und diesen Knöpfen: **Änderungen ansehen** (öffnet die Seite der neuen Version in deinem Browser, von dort holst du sie wie beim ersten Mal), **Später erinnern** und **Diese Version überspringen**. **VintageCoder lädt und installiert nichts selbst.** Bringt die neue Version ein neueres Projektformat mit, warnt die Seite davor: ältere Versionen können deine Projekte dann nicht mehr lesen.

Übertragen wird nur eine Anfrage nach der Versionsliste an `github.com`; GitHub sieht dabei deine IP-Adresse und den Programmnamen „VintageCoder-update“, sonst nichts (keine Kennung, keine Angaben zu Rechner oder Projekten). Dieselbe Seite **Info und Update** hat den Knopf **Jetzt nach Updates suchen**; du erreichst sie auch über das Menü **Hilfe ▸ Über VintageCoder** oder **Hilfe ▸ Nach Updates suchen …**. Auf der Kommandozeile zeigt `vcc update` dasselbe. Einzelheiten: Hilfe „Nach Updates suchen“. In Paketen ohne Update-Prüfung ist der Knopf grau.

## Beispielprojekte

VintageCoder bringt fertige Beispiele mit. Auf der Startseite öffnet **Beispiel öffnen** einen Dialog mit den Beispielen aus dem Ordner `examples/` neben der Hardware-Datenbank (zum Beispiel „c64 / border“); **Öffnen** öffnet das Projekt, **Kopieren …** legt eine Kopie an einem Ort deiner Wahl an. Dieselben Beispiele stehen auch auf der Startseite unter **Beispielprojekte**. Unter Windows kopiert das Setup die Beispiele nach „Dokumente\VintageCoder\Beispiele“, auf dem Mac liegt der Ordner **Beispiele** im .dmg. Es gibt Beispiele für alle Systeme, zum Beispiel `c64/border`, `atari/colorbars` und `mega65/hello`.

## Der Test: Hello World mit F5

Zum Schluss prüfst du, dass die ganze Kette läuft.

1. Klicke auf der Startseite auf **Neues Projekt** (oder drücke Cmd/Strg+Alt+N). Hast du im Konfigurationsassistenten unter „Erstes Projekt“ **Neues Projekt anlegen …** gewählt, öffnet sich der Dialog nach dem Schritt **Fertig** schon.
2. Wähle das Zielsystem **Commodore 64** und die Vorlage **Assembler (vcasm)** ▸ **Assembler-Programm**. Gib dem Projekt einen Namen wie `hallo` und wähle einen Speicherort. Klicke auf **Projekt erstellen und öffnen**.

![Der Dialog „Neues Projekt erstellen“ über der Startseite. Links die Liste „ZIELSYSTEM“ mit Commodore 64 (gewählt) bis MEGA65 und dem Hardware-Profil „gültig“, in der Mitte die „VORLAGE“ mit „Assembler-Programm“ (gewählt), „Steckmodul“ und „Leeres Projekt“, rechts die „PROJEKT-DETAILS“ mit Projektname, Speicherort, Videonorm, Standard-Emulator, Startziel, Diskette und Zeichensatz. Unten die Knöpfe „Abbrechen“ und „Projekt erstellen und öffnen“.](bild:neues-projekt-dialog "Der Dialog „Neues Projekt erstellen“ mit Zielsystem, Vorlage und Projekt-Details.")

3. Drücke **F5**. VintageCoder baut das Programm und startet VICE. Im Emulator läuft der Rahmen in bunten Streifen durch die Farben.
4. Schließe den Emulator.

Klappt das, ist die Einrichtung fertig. Hast du im Assistenten die Beispiele kopieren lassen, geht es auch mit einem Beispiel: **Beispiel öffnen**, ein Beispiel im Dialog mit **Öffnen** öffnen, **F5**. Eine ausführliche Anleitung mit Bildern steht in [Dein erstes Projekt](kapitel:erstes-projekt).

## Häufige Stolpersteine

- **Der Assistent ist verschwunden oder hat sich nicht geöffnet.** Er öffnet sich nur beim allerersten Start von selbst. Ruf ihn über **Hilfe ▸ Konfigurationsassistent …**, die Befehlspalette oder **Einstellungen ▸ Allgemein ▸ Einrichtung** auf.
- **Das Startbild stört.** Schalte es unter **Einstellungen ▸ Allgemein ▸ Startbild** aus.
- **Emulator „Nicht gefunden“ in den Einstellungen.** Er ist nicht installiert oder liegt an einem ungewöhnlichen Ort. Installiere ihn und klicke **Suchen**, oder trage den Pfad ein.
- **macOS verweigert das Öffnen.** Rechtsklick auf die App und **Öffnen** (Gatekeeper-Hinweis oben).
- **`vcc` wird im Terminal nicht gefunden.** Windows: Haken „zum PATH hinzufügen“ im Setup nachholen (Setup erneut ausführen) und eine neue Eingabeaufforderung öffnen. macOS: **Kommandozeile installieren.command** ausführen.
- **Der MEGA65-Emulator startet nicht.** Xemu braucht eine `MEGA65.ROM` von genau 131072 Byte.
- **Der Zeichensatz-Editor zeigt den eigenen Satz von VintageCoder.** Es fehlt das Zeichensatz-ROM deines Systems; richte es mit dem Teil-Assistenten „nur ROMs“ oder auf der Seite **Einstellungen ▸ ROMs** ein.
- **Windows-Debugger stürzt ab.** VICE 3.9 stürzt beim Einzelschritt ab; nimm 3.10 oder neuer.

## Wie geht es weiter?

Weiter geht es mit [Willkommen bei VintageCoder](kapitel:willkommen), falls du einen Überblick möchtest, oder direkt mit [Dein erstes Projekt](kapitel:erstes-projekt). Die Oberfläche erklärt [Die Oberfläche im Überblick](kapitel:oberflaeche).
