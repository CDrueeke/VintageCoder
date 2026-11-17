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

1. Starte VintageCoder. Als Erstes erscheint kurz das **Startbild**: eine kleine Karte mit dem Logo, dem Namen, der Version und dem Hinweis **Alpha-Version**. Ein Klick oder eine beliebige Taste schließt sie; sonst verschwindet sie nach wenigen Sekunden von selbst.
2. **Alpha-Version** heißt: VintageCoder ist noch im Alpha-Stadium und enthält wahrscheinlich noch viele Fehler. Es gibt keine Gewährleistung und keine Haftung für Datenverlust (siehe Lizenz). Den Hinweis liest du jederzeit im Dialog **Über VintageCoder**. Sichere deine Projekte regelmäßig, zum Beispiel mit Git oder einer Datensicherung, besonders bevor du etwas auf echte Geräte überträgst.
3. Das Startbild lässt sich abschalten: **Einstellungen ▸ Editor ▸ Startbild** (An oder Aus). Die Wahl gilt ab dem nächsten Start.
4. Beim **allerersten Start** (es gibt weder `settings.json` noch `wizard.json` im Konfigurationsordner) öffnet sich der **Konfigurationsassistent** über der Oberfläche. Im nächsten Abschnitt gehst du ihn durch. Danach siehst du die **Startseite**, den Startpunkt, solange keine Datei offen ist.

![Die Startseite mit den Knöpfen „Neues Projekt“, „Projekt öffnen“, „Beispiel öffnen“ und „Einrichten“, der Liste der zuletzt geöffneten Projekte, den Studios und rechts dem Kasten „Toolchain und Hardware“ mit der Schnellreferenz.](bild:willkommen-seite "Die Startseite")

5. Oben stehen **Neues Projekt**, **Projekt öffnen** (Cmd/Strg+Umschalt+O), **Beispiel öffnen** (klappt auf der Seite eine Liste der mitgelieferten Beispiele auf) und **Einrichten** (öffnet die Einstellungen). Darunter folgen die Liste der **zuletzt geöffneten Projekte** (beim ersten Start leer) und die **Studios** für Grafik und Klang (gesperrt, solange kein Projekt offen ist).
6. Rechts zeigt der Kasten **Toolchain und Hardware** je Emulator einen grünen Punkt mit „bereit“ (gefunden) oder einen grauen mit „nicht gefunden“, darunter die eingetragenen Maschinen. Der Knopf **Einstellungen öffnen** springt zu den Einstellungen, der Knopf **Konfigurationsassistent** startet den Assistenten erneut, **Feedback senden …** bereitet eine Mail vor. Beim Start findet kein Netzwerkzugriff statt; Hardware steht deshalb auf „ungeprüft“, bis du **Testen** klickst.

> **Info:** Cmd bedeutet auf dem Mac, was unter Windows Strg ist. Im Handbuch steht deshalb **Cmd/Strg**. Die Startseite erreichst du später mit dem Befehl **Startseite** (Menü **Datei**, auch in der Befehlspalette).

> **Info:** Die Menüleiste mit **Datei**, **Bearbeiten**, **Ansicht**, **Projekt** und **Hilfe** gibt es auf allen Systemen: auf dem Mac oben am Bildschirmrand, unter Windows und Linux als schmale Zeile über der Kopfzeile (Einstellungen ▸ Editor ▸ „Menüleiste anzeigen“, im Zen-Modus ausgeblendet). Unter Windows und Linux öffnest du ein Menü mit **Alt+D**, **Alt+B**, **Alt+A**, **Alt+P** oder **Alt+H**, wählst mit den Pfeiltasten und löst mit Enter aus; auf dem Mac bedienst du die Menüleiste des Systems wie gewohnt (Option+Buchstabe bleibt dort immer ein Textzeichen). Alle Befehle erreichst du außerdem über die Befehlspalette (**Cmd/Strg+K**), die Kürzel, die Kopfzeile und die Kontextmenüs.

> **Tipp:** Hattest du VintageCoder schon vor dem Assistenten benutzt (es gibt eine `settings.json`, aber noch keine `wizard.json`), öffnet er sich nicht von selbst. Dann bietet die Startseite einmal den Hinweis „Neu: Konfigurationsassistent“ an (Knopf **Assistent öffnen**), den du annehmen oder mit ✕ wegklicken kannst.

## Der Konfigurationsassistent

Der Assistent bringt dich in wenigen Minuten von „frisch installiert“ bis „F5 startet mein Programm“. Er arbeitet nach drei Regeln:

- **Nichts wird gesendet.** Kein Netzwerkzugriff, keine Telemetrie. Er **installiert nichts** und lädt **nichts** herunter; er zeigt nur, was auf deinem Rechner gefunden wurde. Eine Ausnahme gibt es: Im Schritt **Emulatoren** (langer Weg) zeigt der Assistent dieselbe Seite wie die Einstellungen. Fehlt dort ein Emulator, kann sie eine portable Installation anbieten; geladen wird erst nach deinem Klick auf **Jetzt laden**.
- **Er arbeitet auf einem Entwurf.** Alle Änderungen stehen zuerst im Speicher. Erst in der **Zusammenfassung** siehst du die Liste und bestätigst mit **Übernehmen**. Nur das **Design** (Farbschema, Größe, Schrift) wirkt sofort, damit du siehst, was du wählst; **Abbrechen** stellt das Alte wieder her.
- **Du kannst jederzeit aufhören.** **Abbrechen** beendet den Assistenten, ohne Einstellungen zu ändern. **Später fortsetzen** tut dasselbe und merkt sich, bei welchem Schritt du warst, nicht deine Eingaben.

Die Oberfläche des Assistenten ist **Deutsch**; eine Sprachwahl gibt es erst, wenn VintageCoder eine zweite Sprache hat.

### Den Weg wählen

Der erste Schritt, **Willkommen**, bietet drei Wege:

| Weg | Schritte | Wofür |
|---|---|---|
| **Schnelleinrichtung** (Knopf **Schnelleinrichtung starten**) | 7 | Design, Zielsysteme, Toolchain und Projekte; Emulatoren, ROMs und Hardware werden automatisch gefunden und später in den Einstellungen verfeinert |
| **Alles selbst einstellen** | 12 | zusätzlich Emulatoren, ROMs, echte Hardware, Startziel F5 und Editor |
| **Nur einen Teil** | 3 bis 4 | die Knöpfe **ROMs**, **Emulatoren**, **Hardware** und **Design und Editor** richten nur diesen Teil ein |

Links siehst du die **Schrittleiste** mit allen Schritten des gewählten Weges. Mit **Weiter** gehst du einen Schritt vor, mit **Zurück** einen zurück. Bereits erreichte Schritte kannst du in der Leiste direkt anklicken; ein ✓ steht vor den Schritten, die du hinter dir hast. Hast du den Assistenten schon einmal mit **Später fortsetzen** verlassen, steht auf dieser Seite „Offen von der letzten Einrichtung“ mit den Schritten, die noch fehlen.

### Schnelleinrichtung, Schritt für Schritt

**Schritt 2: Design.** Wähle **Dunkel**, **Hell** oder **System** (folgt dem Betriebssystem). Zwei Knopfpaare **−** und **+** stellen die Größe der **Oberfläche** (in Prozent) und der **Editor-Schrift** (in Punkt) ein. Alles wirkt sofort.

**Schritt 3: Zielsysteme.** Hake die Systeme an, mit denen du arbeiten willst: C64, C128, C16/Plus/4, VC-20, Atari 800 XL/XE, MEGA65. Nur gewählte Systeme erscheinen später in der Oberfläche; abgewählte werden nur ausgeblendet, nichts geht verloren.

> **Info:** Der Umfang dieser Version sind C64, C128, C16/Plus/4, VC-20, Atari 8 Bit und MEGA65. Der Commodore PET/CBM und der CBM-II gehören nicht dazu.

**Schritt 4: Toolchain.** Die Seite nennt den eingebauten **Assembler** (kein Java, keine Zusatzinstallation) und zeigt für jeden Emulator, ob er gefunden wurde („bereit“) oder fehlt („nicht gefunden“). Gesucht wird im `PATH` und in den üblichen Ordnern. Fehlende Emulatoren trägst du im Schritt **Emulatoren** (langer Weg) oder später in den Einstellungen ein. Ohne Emulator lassen sich die betroffenen Systeme bauen, aber nicht starten.

> **Info:** Auf dem Mac sucht die Erkennung im `PATH` und in den üblichen Ordnern (Homebrew, `/Applications`). Unter Windows und Linux sucht sie vor allem im `PATH`. Liegt dein Emulator woanders, trägst du den Pfad in den Einstellungen ein (**Zielsysteme und Hardware**, Feld **Pfad**). Unter macOS blockiert Gatekeeper nicht beglaubigte Programme beim ersten Start; dann hilft Rechtsklick ▸ **Öffnen**.

**Schritt 5: Projekte.** Das **Basisverzeichnis** ist der Ordner, in dem deine Projekte liegen; der Projekt-Assistent schlägt ihn als Speicherort vor. Leer heißt: `~/VintageCoder`. Das Ändern verschiebt nichts, es ändert nur die Voreinstellung für neue Projekte. Mit dem Haken **Beispielprojekte in meinen Projektordner kopieren (Ordner „Beispiele“)** legt der Assistent die Beispielprojekte für deine Systeme dorthin; du kannst einzelne Beispiele abwählen. Kopiert wird erst bei **Übernehmen**, vorhandene Dateien bleiben unberührt. Unter **Erstes Projekt** wählst du, womit es nach **Fertig** weitergeht: **Später entscheiden**, **Neues Projekt anlegen** oder ein Beispiel öffnen.

**Schritt 6: Zusammenfassung.** Die Liste zeigt jede Änderung mit Haken, dem Bereich und der Art (**neu**, **geändert**, **entfernt**) sowie dem alten und dem neuen Wert. Jeden Eintrag kannst du abwählen; was nicht angehakt ist, wird nicht geschrieben. Darüber steht, wie viele Änderungen übernommen werden, und dass vorher die Sicherung `settings.json.bak` entsteht. Gibt es nichts zu ändern, steht dort „Es gibt nichts zu ändern“. **Übernehmen** schreibt.

**Schritt 7: Fertig.** Der Assistent meldet, dass die Einrichtung übernommen ist, und zeigt den Ort der Sicherung, Hinweise und die kopierten Beispiele. Mit **Rückgängig** stellst du Einstellungen und kopierte Beispiele wieder her. **Fertig** schließt den Assistenten und führt aus, was du unter **Erstes Projekt** gewählt hast (Projekt-Assistent oder Beispiel öffnen).

> **Achtung:** Der Assistent baut und startet **nichts von selbst**. Ob die Kette aus Bauen und Emulator wirklich läuft, prüfst du im Abschnitt „Der Test: Hello World mit F5“.

### „Alles selbst einstellen“: die zusätzlichen Schritte

Der lange Weg hat zwölf Schritte: Willkommen, Design, Zielsysteme, Toolchain, **Emulatoren**, **ROMs**, **Echte Hardware**, **Startziel F5**, **Editor**, Projekte, Zusammenfassung, Fertig. Die Schritte dazwischen sind:

| Schritt | Was du dort einstellst |
|---|---|
| **Emulatoren** | je System der Standard-Emulator, Pfad und Startparameter (dieselbe Seite wie **Zielsysteme und Hardware** in den Einstellungen) |
| **ROMs** | ROM-Dateien aus dem ROM-Ordner oder per Ziehen auf das Fenster; erkannt wird am Inhalt, ein Download findet nicht statt. Dateien kommen erst nach deiner Bestätigung in den ROM-Ordner. Freie Ersatz-ROMs genügen oft |
| **Echte Hardware** | Maschinen (zum Beispiel Ultimate 64 / II+) eintragen; Verbindungen werden nur auf Klick getestet |
| **Startziel F5** | je System genau ein Ziel: der Standard-Emulator oder eine eingetragene Maschine; F5 und F6 starten dorthin |
| **Editor** | dieselbe Seite wie **Editor** in den Einstellungen (Schrift, Zeilenumbruch, Startbild, Takt und mehr). Die **Tastenkürzel sind fest**; ein Anpassen ist noch nicht möglich |

> **Achtung:** Das Netzwerkpasswort einer Ultimate liegt in `settings.json` (nur für dich lesbar). Die Ablage im Schlüsselbund des Betriebssystems gibt es noch nicht.

### Den Assistenten später wieder aufrufen

Du kannst den Assistenten jederzeit erneut öffnen:

- Menü **Hilfe ▸ Konfigurationsassistent …** (nur Mac, sonst die Befehlspalette),
- auf der Startseite der Knopf **Konfigurationsassistent**,
- in den **Einstellungen** auf der Seite **Allgemein** im Abschnitt „Einrichtung“ der Knopf **Assistent starten**,
- in der Befehlspalette (Cmd/Strg+K) mit dem Wort „Konfigurationsassistent“.

Die **Teil-Assistenten** richten nur einen Teil ein: **Konfigurationsassistent: nur ROMs …**, **nur Emulatoren …**, **nur echte Hardware …** und **nur Design und Editor …**. **Konfigurationsassistent: Nach Neuem suchen …** liest den ROM-Ordner neu ein und öffnet den Teil für die Emulatoren. Du erreichst sie im Menü **Hilfe**, in der Befehlspalette und auf der ersten Seite des Assistenten.

Zwei Hinweise auf der Startseite erinnern dich an offene Dinge: „Einrichtung unvollständig: ein paar Schritte sind noch offen“ (Knopf **Fertig einrichten**), wenn du den Assistenten mit **Später fortsetzen** verlassen hast, und der einmalige Hinweis „Neu: Konfigurationsassistent“ (Knopf **Assistent öffnen**). Mit ✕ blendest du sie aus.

**Konfiguration exportieren, importieren und zurücksetzen** gehören nicht zum Assistenten. Du findest sie unter **Einstellungen ▸ Allgemein ▸ Konfiguration sichern**: **Exportieren …** schreibt die Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen, Parameter) in eine JSON-Datei, ohne Passwörter. **Importieren …** zeigt vorher, was drinsteht, und ersetzt dann die jetzigen Einstellungen. **Zurücksetzen …** stellt die Vorgabe her. Farbschema und Größe gehören nicht dazu.

### Was der Assistent nicht kann

- Er **installiert keine Programme** und lädt nichts herunter (auch keine ROMs).
- Er stellt **keine Tastenbelegung** ein; die Kürzel sind fest.
- Die Oberfläche ist **nur auf Deutsch**.
- Die Passwortablage im **Schlüsselbund** des Betriebssystems fehlt noch.
- Die automatische Emulator-Suche ist auf dem Mac am gründlichsten; unter Windows und Linux hilft das Feld **Pfad** in den Einstellungen.

> **Tipp:** Für Skripte und Testläufe unterdrückt die Umgebungsvariable `VC_NO_WIZARD=1` den automatischen Start des Assistenten.

## Von Hand einstellen: Emulator, ROMs und Hardware

Alles, was der Assistent einstellt, findest du auch in den **Einstellungen**. Die folgenden Abschnitte zeigen den manuellen Weg, zum Beispiel für Feinheiten oder wenn du die Einstellungen später ändern willst.

### Schritt für Schritt: Einen Emulator finden und einstellen

VintageCoder startet dein Programm in einem **Emulator**, einem eigenen Programm, das du selbst installierst. VintageCoder bringt keinen Emulator mit und lädt nie ungefragt etwas: Wo es für dein System ein festes Paket gibt, bietet die Emulator-Karte in den Einstellungen den Knopf **Portabel installieren …** an; geladen wird erst nach deiner Bestätigung (mit Prüfung der SHA-256-Summe). Unter Linux siehe auch [Emulatoren unter Linux](kapitel:systeme).

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
2. Öffne die **Einstellungen**: mit dem Zahnrad ⚙ rechts in der Kopfzeile, mit **Cmd/Strg+Komma** oder (Mac) im Menü **Ansicht ▸ Einstellungen**.
3. Wechsle zur Seite **Zielsysteme und Hardware** und wähle links dein System. Der Punkt vor dem System ist grün, wenn der Standard-Emulator gefunden wurde, und rot, wenn er fehlt. Fehlt dein System in der Liste, nimmst du es links unten mit **Zielsystem hinzufügen** auf.
4. Rechts steht in der Karte des Emulators bei **Pfad** der Pfad des Programms. **Leer** heißt: VintageCoder sucht selbst im `PATH` und in den üblichen Ordnern. Der Knopf **Automatisch suchen** trägt den gefundenen Pfad ein. Findet er nichts, trägst du den Pfad von Hand ein. Ein Chip zeigt **Automatisch erkannt**, **Gefunden** oder **Fehlt**.
5. Mit **▶ Test-Start** startest du den Emulator kurz zur Probe. Die Meldung zeigt, ob es geklappt hat.
6. Klicke unten auf **Speichern**. **Verwerfen** nimmt deine Änderungen zurück. Unten steht „● Ungespeicherte Änderungen“, solange etwas offen ist.

![Einstellungen, Seite „Zielsysteme und Hardware“: links die Systemliste mit grünen Punkten, rechts das gewählte System mit dem Standard-Emulator VICE x64sc, dem Pfadfeld, den Startparametern und dem Knopf „Test-Start“.](bild:einstellungen-emulator "Einstellungen: Emulator-Pfad prüfen und mit „Test-Start“ ausprobieren")

**Wo VintageCoder automatisch sucht:** im `PATH` und zusätzlich zum Beispiel unter macOS in `/Applications/vice-arm64-gtk3-3.10/bin`, `/Applications/vice-arm64-gtk3-3.9/bin`, `/opt/homebrew/bin` und `/usr/local/bin` (Xemu auch unter `/Applications/xmega65.app/Contents/MacOS`). Liegt dein Emulator woanders, trage den Pfad ein. Für Xemu geht auch die Umgebungsvariable `VC_XEMU`.

Einige Besonderheiten:

- Ein System kann **mehrere Emulatoren** haben; einer ist der **Standard** („Als Standard setzen“). Die Knöpfe **+ VICE x64sc** und ähnliche über den Karten fügen einen weiteren Emulator der jeweiligen Sorte hinzu.
- **Startparameter** hängt VintageCoder an die Kommandozeile an (siehe [Bauen und Starten](kapitel:bauen-starten)).
- **atari800** kann ein eigenes **OS-ROM** bekommen, **Xemu** den **Ordner mit `MEGA65.ROM`**. Im Normalfall legst du die ROMs aber im ROM-Ordner ab (nächster Abschnitt).
- **Startziel (F5)** bestimmt, wohin F5 und F6 starten: der Standard-Emulator oder eine eingetragene Maschine.

#### Ultimate 64 (echte Hardware)

Unter **Physische Zielhardware & Netzwerkgeräte** trägst du mit dem Knopf **+ Ultimate 64 / II+** (Adresse wie `192.168.1.50`, optional Netzwerkpasswort) eine Maschine ein. **Verbindung testen** prüft, ob sie antwortet. Die Anbindung an einen echten MEGA65 (USB oder Ethernet) ist in den Paketen dieser Version ausgeblendet. Einzelheiten und der Ablauf stehen im Kapitel [Bauen und Starten](kapitel:bauen-starten).

### ROM-Dateien: brauchst du sie?

Ein **ROM** ist der fest eingebaute Speicher eines alten Computers: Betriebssystem, BASIC, Zeichensatz. VintageCoder liefert **keine Original-ROMs** mit und lädt nichts aus dem Netz. Wer sie braucht, legt seine **eigenen Abbilder** (Dateien) in den ROM-Ordner. VintageCoder erkennt sie am **Inhalt** (Größe und Prüfsumme), der Dateiname ist egal. Ob du ein Abbild benutzen darfst, klärst du selbst.

| Wofür | Ohne ROM | Mit ROM |
|---|---|---|
| VICE, atari800 | laufen mit eigenen Ersatz-ROMs | nutzen deine Abbilder |
| Xemu (MEGA65) | startet **nicht**: braucht eine `MEGA65.ROM` von genau 131072 Byte | läuft |
| Zeichensatz-Editor, Zeichenwähler | der eigene Standardsatz von VintageCoder (VC-Standard), mit Hinweis | Original-Zeichensatz deines Systems |

Für das Hello-World-Programm mit VICE brauchst du **keine** ROMs.

#### Schritt für Schritt: ROMs einrichten

1. Starte den Teil-Assistenten über **Hilfe ▸ Konfigurationsassistent: nur ROMs …** (Mac-Menü) oder über die Befehlspalette (Cmd/Strg+K, Wort „ROMs“).
2. Der Teil-Assistent hat drei Schritte: **Zielsysteme**, **ROMs** und **Zusammenfassung**. Im Schritt **ROMs** ziehst du deine ROM-Dateien oder ZIP-Archive auf das Fenster oder wählst sie mit **Auswählen …**; sie kommen erst nach deiner Bestätigung in den ROM-Ordner. **Ordner öffnen** zeigt den Ordner.
3. Dieselbe Seite findest du unter **Einstellungen ▸ ROMs**. Sie zeigt je System und Rolle (zum Beispiel `kernal`, `basic`, `chargen` beim C64), ob eine passende Datei gefunden wurde, und bietet **Ordner öffnen**, **Neu prüfen** und **Auswählen …**. Der Haken **ROMs anderer Programme mitbenutzen (VICE, Xemu, atari800 …)** (Vorgabe aus) lässt VintageCoder auch die ROM-Ordner dieser Programme lesen; **ROMs aus VICE importieren …** zeigt die gefundenen Dateien und kopiert sie nach Bestätigung.

![Einstellungen, Seite „ROMs“ (Ausschnitt): oben die Reiter je System, darunter je Rolle (KERNAL, BASIC, Zeichensatz, Laufwerk) der Status, die erwartete Größe und der Knopf „Auswählen …“.](bild:einstellungen-roms "Einstellungen ▸ ROMs: je Rolle ein Status; hier ist noch nichts gefunden")

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

VintageCoder kann nachsehen, ob es eine neue Version gibt. **Ohne deine Zustimmung geschieht das nie:** Solange du unter **Einstellungen ▸ Allgemein ▸ Updates** bei **Nach Updates suchen** nichts gewählt hast, steht dort „Noch nicht entschieden: es wird nichts gesendet“. Du kannst *Nur auf Wunsch*, *Täglich beim Start* oder *Wöchentlich beim Start* wählen. Den Abschnitt **Updates** gibt es nur in Paketen mit Update-Prüfung.

Gibt es eine neuere Version, steht in der Statusleiste **Update X verfügbar**. Ein Klick öffnet einen Dialog mit den **Änderungen** seit deiner Version und diesen Knöpfen: **Ansehen …** (öffnet die Seite der neuen Version in deinem Browser, von dort holst du sie wie beim ersten Mal), **Später erinnern**, **Diese Version überspringen** und **Schließen**. **VintageCoder lädt und installiert nichts selbst.** Bringt die neue Version ein neueres Projektformat mit, warnt der Dialog davor: ältere Versionen können deine Projekte dann nicht mehr lesen.

Übertragen wird nur eine Anfrage nach der Versionsliste an `github.com`; GitHub sieht dabei deine IP-Adresse und den Programmnamen „VintageCoder-update“, sonst nichts (keine Kennung, keine Angaben zu Rechner oder Projekten). Dieselbe Seite **Allgemein ▸ Updates** hat den Knopf **Jetzt prüfen**; ihn gibt es auch im Dialog **Über VintageCoder** (Menü **Hilfe**, nur Mac; sonst Befehlspalette). Auf der Kommandozeile zeigt `vcc update` dasselbe. Einzelheiten: Hilfe „Nach Updates suchen“.

## Beispielprojekte

VintageCoder bringt fertige Beispiele mit. Auf der Startseite klappt **Beispiel öffnen** eine Liste der Beispiele aus dem Ordner `examples/` neben der Hardware-Datenbank auf (zum Beispiel „c64 / border“); ein Klick öffnet das Projekt. Unter Windows kopiert das Setup die Beispiele nach „Dokumente\VintageCoder\Beispiele“, auf dem Mac liegt der Ordner **Beispiele** im .dmg. Es gibt Beispiele für alle Systeme, zum Beispiel `c64/border`, `atari/colorbars` und `mega65/hello`.

## Der Test: Hello World mit F5

Zum Schluss prüfst du, dass die ganze Kette läuft.

1. Klicke auf der Startseite auf **Neues Projekt** (oder drücke Cmd/Strg+Alt+N). Hast du im Konfigurationsassistenten unter „Erstes Projekt“ „Neues Projekt anlegen“ gewählt, öffnet sich der Projekt-Assistent nach **Fertig** schon.
2. Wähle das Zielsystem **Commodore 64** und die Vorlage **Assembler (vcasm)** ▸ **Assembler-Programm**. Gib dem Projekt einen Namen wie `hallo` und wähle einen Speicherort. Klicke auf **Projekt erstellen und öffnen**.

![Der Projekt-Assistent, rechte Spalte „Projekt-Details“: Projektname, Speicherort, Videonorm, Standard-Emulator, Startziel, Diskette, Zeichensatz, weitere Zielsysteme und die Vorschau der Dateistruktur.](bild:wizard-details "Spalte 3: Name, Speicherort und die Vorschau der Dateien")

3. Drücke **F5**. VintageCoder baut das Programm und startet VICE. Im Emulator läuft der Rahmen in bunten Streifen durch die Farben.
4. Schließe den Emulator.

Klappt das, ist die Einrichtung fertig. Hast du im Assistenten die Beispiele kopieren lassen, geht es auch mit einem Beispiel: **Beispiel öffnen**, ein Beispiel aus der Liste wählen, **F5**. Eine ausführliche Anleitung mit Bildern steht in [Dein erstes Projekt](kapitel:erstes-projekt).

## Häufige Stolpersteine

- **Der Assistent ist verschwunden oder hat sich nicht geöffnet.** Er öffnet sich nur beim allerersten Start von selbst. Ruf ihn über **Hilfe ▸ Konfigurationsassistent …** (Mac-Menü), die Befehlspalette oder den Knopf auf der Startseite auf.
- **Das Startbild stört.** Schalte es unter **Einstellungen ▸ Editor ▸ Startbild** aus.
- **Emulator rot in den Einstellungen.** Er ist nicht installiert oder liegt an einem ungewöhnlichen Ort. Installiere ihn und klicke **Automatisch suchen**, oder trage den Pfad ein. 
- **macOS verweigert das Öffnen.** Rechtsklick auf die App und **Öffnen** (Gatekeeper-Hinweis oben).
- **`vcc` wird im Terminal nicht gefunden.** Windows: Haken „zum PATH hinzufügen“ im Setup nachholen (Setup erneut ausführen) und eine neue Eingabeaufforderung öffnen. macOS: **Kommandozeile installieren.command** ausführen.
- **Der MEGA65-Emulator startet nicht.** Xemu braucht eine `MEGA65.ROM` von genau 131072 Byte.
- **Der Zeichensatz-Editor zeigt den eigenen Satz von VintageCoder.** Es fehlt das Zeichensatz-ROM deines Systems; richte es mit dem Teil-Assistenten „nur ROMs“ ein.
- **Windows-Debugger stürzt ab.** VICE 3.9 stürzt beim Einzelschritt ab; nimm 3.10 oder neuer.

## Wie geht es weiter?

Weiter geht es mit [Willkommen bei VintageCoder](kapitel:willkommen), falls du einen Überblick möchtest, oder direkt mit [Dein erstes Projekt](kapitel:erstes-projekt). Die Oberfläche erklärt [Die Oberfläche im Überblick](kapitel:oberflaeche).
