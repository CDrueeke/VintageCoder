# BASIC in Assembler umwandeln

Der Befehl **Datei ▸ BASIC in Assembler umwandeln** (auch in der Befehlspalette, ohne Kürzel) übersetzt das offene BASIC-Programm (`.bas`) in Assembler-Quelltext. Er braucht ein geöffnetes Projekt und fragt nichts ab: Es gibt keinen Dialog.

Das Ergebnis steht in einer **neuen Datei** neben dem Original: `<name>_asm.asm`. Ist der Name belegt, heißt sie `<name>_asm-2.asm` und so weiter; nichts wird überschrieben. Dazu entsteht eine Abbildungsdatei `<name>_asm.b2a.json`. Die neue Datei öffnet im Editor. Die Umwandlung richtet sich nach dem ersten Ziel des Projekts. Eine Meldung im **Protokoll** der Schublade nennt Stufe, Zeilenzahl und Module; Hinweise und Fehler stehen im Reiter **Probleme**.

Die Umwandlung ist mechanisch: Sie ersetzt BASIC-Anweisungen durch Aufrufe von Unterprogrammen und braucht dieselben Dialekte wie der BASIC-Assembler. Prüfe das Ergebnis von Hand; nicht jedes BASIC-Programm lässt sich vollständig übertragen. Siehe auch [BASIC](help:basic).
