# SPDX-License-Identifier: GPL-3.0-or-later
# ROM-Bibliothek (vc-roms): Statusnamen, Orte, Rollen, Meldungen

# Status einer ROM-Rolle
roms-status-recognized = Erkannt
roms-status-unknown-revision = Unbekannte Revision
roms-status-replacement = Ersatz (frei)
roms-status-missing = Fehlt
roms-status-unusable = Nicht benutzbar

# Art eines Suchorts
roms-stage-explicit = Einstellung
roms-stage-project = Projekt
roms-stage-portable = Programmordner (portabel)
roms-stage-user = Benutzerordner
roms-stage-legacy = Alte Ablage
roms-stage-foreign = Anderes Programm
roms-stage-bundled = Mitgeliefert (Ersatz)

# $system: Systemkennung | Beispiel: c64
# $role: Rolle | Beispiel: kernal
roms-place-setting = Einstellung { $system }/{ $role }
roms-place-legacy-mega65 = Alte Ablage (mega65)
roms-place-xemu-appdata = Xemu (Programmdaten)
roms-place-vice-configured = VICE (eingestellt)

# Kurzzeile einer aufgelösten Rolle
# $system: Systemkennung | Beispiel: c64
# $role: Rolle | Beispiel: kernal
# $status: Status | Beispiel: Erkannt
roms-describe = { $system }/{ $role }: { $status }
# $label: Bezeichnung der Revision | Beispiel: C64 KERNAL 901227-03
roms-describe-label = – { $label }
# $stage: Art des Ortes | Beispiel: Benutzerordner
# $path: Pfad | Beispiel: /pfad
roms-describe-place = ({ $stage }: { $path })
# $sizes: Größen, getrennt durch „oder“ | Beispiel: 8192 oder 16384
roms-describe-expected = – erwartet { $sizes } Byte
roms-or = oder

# Gründe, warum eine Datei nicht erkannt wurde
roms-ident-no-rom-size = keine ROM-Größe
roms-ident-too-large = zu groß für ein ROM-Abbild
roms-ident-unreadable = nicht lesbar
# $h: System | Beispiel: c64
# $size: Größe in Byte | Beispiel: 123
roms-ident-no-role = passt zu keiner Rolle von { $h } (Größe { $size } Byte)

# Portabler Modus
roms-portable-translocation = App-Translocation: die App läuft aus einem schreibgeschützten Zufallspfad; zum Aktivieren in den Ordner „Programme“ verschieben und neu starten
roms-portable-bundle-no-parent = Bundle ohne übergeordneten Ordner
roms-portable-no-program-dir = kein Programmordner

# Tabelle bekannter ROMs
roms-known-format = known.json: Format { $format } wird nicht unterstützt
# $id: Kennung des Eintrags | Beispiel: c64-kernal
roms-known-no-uses = known.json: { $id } ohne "uses"
roms-known-bad-parts = known.json: { $id } hat ungültige Teile (Offset oder Größe)

# Rollen
roms-role-kernal = KERNAL
roms-role-basic = BASIC
roms-role-chargen = Zeichensatz
roms-role-dos1541 = Laufwerk 1541
roms-role-dos1571 = Laufwerk 1571
roms-role-dos1581 = Laufwerk 1581
roms-role-basiclo = BASIC (niedrig)
roms-role-basichi = BASIC (hoch)
roms-role-kernal64 = KERNAL (C64-Betrieb)
roms-role-basic64 = BASIC (C64-Betrieb)
roms-role-functionlo = Funktions-ROM (niedrig)
roms-role-functionhi = Funktions-ROM (hoch)
roms-role-os-xl = Betriebssystem XL/XE
roms-role-os-b = Betriebssystem 400/800 (B)
roms-role-os-a = Betriebssystem 400/800 (A)
roms-role-rom = System-ROM
roms-role-basic-b000 = BASIC ($B000)
roms-role-basic-c000 = BASIC ($C000)
roms-role-basic-d000 = BASIC ($D000)
roms-role-editor = Bildschirm-Editor
roms-role-chip-c000 = Chip $C000 (2 KB)
roms-role-chip-c800 = Chip $C800 (2 KB)
roms-role-chip-d000 = Chip $D000 (2 KB)
roms-role-chip-d800 = Chip $D800 (2 KB)
roms-role-chip-f000 = Chip $F000 (2 KB)
roms-role-chip-f800 = Chip $F800 (2 KB)

# Systeme
roms-system-c64 = C64
roms-system-c128 = C128
roms-system-vic20 = VC-20
roms-system-plus4 = Plus/4, C16
roms-system-atari8 = Atari 8 Bit
roms-system-mega65 = MEGA65
roms-system-pet = PET/CBM

# Zeichensätze im ROM-Abbild
roms-set-upper = Großbuchstaben und Grafik
roms-set-lower = Groß-/Kleinschreibung
roms-set-vdc = 80 Spalten (VDC, 8×16)
roms-set-standard = Standard
roms-set-international = International
