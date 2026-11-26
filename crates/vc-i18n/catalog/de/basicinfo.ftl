# SPDX-License-Identifier: GPL-3.0-or-later
# Texte zu den Sprachdiensten für natives BASIC: Karten (Hover) für Direktiven und `#define`-Namen, Anzeige des Dialekts, Gliederung.
# Die Karten sind Markdown. `{ $name }`, `{ $value }`, `{ $line }`: Name und Wert der Definition und ihre Zeile (ab 1).

basicinfo-define = `{ $name }` ist per `#define` definiert (Zeile { $line }): `{ $value }`
basicinfo-define-text = Vor dem Nummerieren wird der Name durch den Wert ersetzt (nicht in Zeichenketten und nach `REM`).
basicinfo-dir-define = `#define NAME wert`
basicinfo-dir-define-text = Definiert einen Namen. Vor dem Nummerieren wird er im Programmtext durch den Wert ersetzt (nicht in Zeichenketten und nach `REM`).
basicinfo-dir-if = `#if BEDINGUNG`
basicinfo-dir-if-text = Nimmt die folgenden Zeilen nur auf, wenn die Bedingung gilt: `SYSTEM_C64` (Zielsystem), ein definierter Name (wahr, wenn sein Wert nicht 0 ist). Ausgeschlossene Zeilen erscheinen grau.
basicinfo-dir-else = `#else`
basicinfo-dir-else-text = Die folgenden Zeilen gelten, wenn die Bedingung des zugehörigen `#if` nicht gilt.
basicinfo-dir-endif = `#endif`
basicinfo-dir-endif-text = Beendet den Block von `#if`.
basicinfo-dir-asm = `#asm "datei.asm" [with "…"]`
basicinfo-dir-asm-text = Bindet Maschinencode ein: Der Code wird mit dem Assembler übersetzt und hinter den BASIC-Text (MEGA65 und Atari: als DATA-Zeilen) gelegt. Im Programm ergibt `asm.name` die Adresse der Marke `name`.
basicinfo-dir-blob = `#blob` gibt es nicht mehr: Die Direktive heißt jetzt `#asm`.
basicinfo-dir-include = `#include "datei"`
basicinfo-dir-include-text = Bindet eine Datei des Projekts ein; bei einem Medium (Sprite, Zeichensatz, Song …) werden DATA-Zeilen daraus erzeugt.
basicinfo-dir-numbering = `#numbering start=N step=N`
basicinfo-dir-numbering-text = Start und Schrittweite der Zeilennummern für diese Datei; schlägt die Einstellung des Projekts.
basicinfo-dir-dialect = `#dialect <id>`
basicinfo-dir-dialect-text = BASIC-Dialekt dieser Datei; schlägt die Einstellung des Projekts und des Zielsystems.
basicinfo-dir-longnames = `#longnames`
basicinfo-dir-longnames-text = Schaltet lange Variablennamen ein; sie werden beim Bauen auf zwei Zeichen gekürzt (Zuordnung in der Datei `.vars.txt`).
