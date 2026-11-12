# MEGA65 (45GS02)

Target `mega65`: PRG ab `$2001` mit `.basicstub`, Chips `vic4` (VIC-III/IV), `dmagic`, `mega65`.

## Befehlssatz (65CE02 + MEGA65)
- Neue Register **Z** und **B**: `ldz`, `cpz`, `inz`, `dez`, `phz`, `plz`, `taz`, `tza`, `tab`, `tba`; Stack: `tsy`, `tys`.
- Zeiger: `lda ($nn),z`, `lda ($nn,sp),y` (Stack-relativ).
- 16 Bit: `phw #$1234`, `phw $1234`, `inw`, `dew`, `asw`, `row`.
- Sprünge: `bsr`, `jsr ($nnnn,x)`, `jmp ($nnnn,x)`, `rts #n`, **lange Sprünge** `lbne`, `lbeq`, `lbra` … (16 Bit); kurze `bne` bleiben 8 Bit.
- Bits: `bbr0..7`, `bbs0..7` (`bbr3 $10,ziel`), `rmb`, `smb`, `trb`, `tsb`, `stz`.
- `neg`, `asr`, `map`, `eom` (`nop` ist ein Alias), `cle`, `see`.

## 32-Bit
- **Zeiger:** `lda [$nn],z` – vier Zeropage-Bytes ab `$nn` sind eine 32-Bit-Adresse.
- **Q-Register** (A, X, Y, Z zusammen): `ldq`, `stq`, `adcq`, `sbcq`, `andq`, `orq`, `eorq`, `cpq`, `bitq`, `aslq`, `lsrq`, `rolq`, `rorq`, `asrq`, `inq`, `deq`, `negq` (`aslq q`).

## VIC-IV freischalten
Der **VIC-IV** ist der erweiterte Grafikchip des MEGA65. Er enthält den VIC-II des C64 und bringt zusätzliche Register mit (bis zu 640 Pixel breiter Text, Farbzeichen mit 256 Farben, 32-Bit-Zeiger auf den Bildschirmspeicher, breite Sprites). Damit sich der MEGA65 nach dem Einschalten wie ein C64 verhält, sind diese **erweiterten Register zunächst gesperrt**. Reine VIC-II-Register wie `vic2.border` funktionieren immer; ein Schreibzugriff auf ein gesperrtes Register (alle mit dem Namen `VIC4_…` oder in der Chip-Gruppe `vic4`) bleibt wirkungslos.

Das **Freischalten** geschieht mit dem Schlüsselregister `vic4.key` (Adresse `$D02F`): Schreibe zuerst `$47`, dann `$53` hinein, in dieser Reihenfolge. Man braucht es einmal am Programmanfang, bevor ein erweitertes Register benutzt wird. Weil der Assembler nur einen Befehl je Zeile kennt (kein `:` als Trenner), sind es vier Zeilen:

```asm target=mega65
        lda #$47
        sta vic4.key
        lda #$53
        sta vic4.key
;= a9 47 8d 2f d0 a9 53 8d 2f d0
```

Kürzer geht es mit dem Makro `mega65_enable_vic4` der Systembibliothek ([Systembibliothek](help:asm-library)). Es erzeugt dieselben vier Befehle und verändert nur den Akkumulator:

```asm target=mega65
#include <mega65/gs4502.asm>
        mega65_enable_vic4
;= a9 47 8d 2f d0 a9 53 8d 2f d0
```

Welche Schreibweise? Die Makro-Form ist kürzer und sagt, was gemeint ist; die vier Zeilen brauchen keine Bibliothek und zeigen, was passiert. Beide ergeben genau dieselben Bytes. Die Projektvorlage und die Beispielprojekte (`examples/mega65`) benutzen das Makro; die erzeugten Bytes sind dieselben wie bei den vier Zeilen.

## Ausführen und Debuggen
„Läuft auf: MEGA65 (Xemu/Hardware)“ lädt das Programm über den UART-Monitor in den Speicher und setzt den PC auf den Einstieg. Verbindung: `xemu` (startet den Emulator Xemu, Umgebungsvariable `VC_XEMU_SD` = Ordner mit `MEGA65.ROM`; sonst das ROM-Verzeichnis, siehe „ROM-Dateien“), `unix:/pfad/socket` (laufender Xemu mit `-uartmon`) oder `/dev/cu.usbserial-…` (echte Hardware). Möglich: Register, Speicher, Pause/Weiter, Einzelschritt, ein Haltepunkt.

## BASIC 65
Dialekt `basic65` (Ladeadresse `$2001`). BASIC-Quelltext (`.bas`) wird zu BASIC-65-Tokens übersetzt; das Ergebnis ist byte-identisch zu `petcat -w65`. Befehle wie `border`, `background`, `foreground`, `vsync`, `play`, `tempo`, `envelope` kennt die Schlüsselwortliste (F1 auf dem Wort).

## Vier SIDs, VIC-IV
- **Musik:** das Profil `mega65.sid4.song` hat 12 Stimmen (4 × SID bei `$D400/$D420/$D440/$D460`). Der Tracker exportiert wie beim C64; die Spielroutine liegt in `examples/mega65/music_demo/lib/music.inc`.
- **PLAY-Tracker:** Profil `mega65.play` – 6 Stimmen, Export als BASIC-Programm mit `VOL`, `TEMPO`, `ENVELOPE` und `PLAY`-Strings (siehe Thema „PLAY-Tracker“).
- **Palette:** Profil `mega65.palette` (256 Farben, 8 Bit je Kanal); Export in die Registerform von `$D100/$D200/$D300` (Halbbytes vertauscht). Beispiel: `examples/mega65/pal_demo/pal_demo.asm`.
- **Vollfarb-Zeichen (FCM):** Profil `mega65.char.fcm`, 8×8 Pixel mit 2 Hex-Ziffern (Palettenindex 0–255) je Pixel, 64 Bytes je Zeichen.

## F5 und Disk-Image
F5 startet `xmega65` und lädt das Programm über den Monitor. Format wählen: `vcc asm … --format d81`.

Hinweis: Die Hardware-Anbindung ist nicht an einem echten Gerät getestet; BASIC-65-Programme und Autoload wurden nicht in Xemu ausgeführt (Xemu bleibt ohne Konfiguration im Einrichtungsbildschirm).

## Xemu einrichten
- **ROM:** Eine MEGA65-ROM-Datei (128 KB, 131072 Byte; Original oder die quelloffenen Open-ROMs) in den ROM-Ordner legen: Einstellungen ▸ **ROMs** (Ablagezone), der Assistent „ROMs einrichten …“ oder `vcc roms import <Datei>`; der Name ist egal, erkannt wird am Inhalt. Die alte Ablage `~/.config/vintagecoder/mega65/MEGA65.ROM`, der Ordner unter Einstellungen ▸ Zielsysteme und `VC_XEMU_SD` gelten weiter. Xemu wird mit `-rom` auf die gefundene Datei gestartet; fehlt sie, nennt die Meldung die gesuchten Orte. Siehe Hilfe „ROM-Dateien“.
- **Programm:** `xmega65` wird in `/opt/homebrew/bin`, `/usr/local/bin` und `/Applications/xmega65.app/Contents/MacOS` gesucht (sonst `VC_XEMU` setzen).
- **Start:** F5 und F6 starten Xemu, laden das Programm über den Monitor in den Speicher und beginnen beim `SYS`-Ziel des BASIC-Starters (F6 hält danach am Haltepunkt an). Der Start dauert etwa 15 Sekunden. Der Autoload von Diskette (`RUN"*"`) scheitert mit den Open-ROMs, deshalb wird er nicht benutzt.
- **Haltepunkte:** Xemu hält erst *nach* dem Befehl am Haltepunkt an; der PC steht dann auf dem folgenden Befehl. Pro Sitzung gibt es einen Haltepunkt.
- **Selbsttests ohne Fenster:** `VC_XEMU_ARGS="-headless"`.

