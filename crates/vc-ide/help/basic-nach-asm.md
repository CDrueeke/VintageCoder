# BASIC in Assembler umwandeln

Der Befehl **BASIC in Assembler umwandeln** (Menü **Datei**, ohne Kürzel) soll das offene BASIC-Programm (`.bas`) in Assembler-Quelltext übersetzen. In dieser Version steht er in der Befehlsliste, ist aber **noch nicht umgesetzt**: Er meldet, dass er in einer späteren Etappe folgt.

Die Umwandlung gibt es schon auf der Kommandozeile: `vcc basic2asm programm.bas --target c64 -o programm.asm`. Die Schalter, die Stufen und die Grenzen stehen auf der Seite [BASIC: Werkzeuge](help:basic-tools).

Die Umwandlung ist mechanisch: Sie ersetzt BASIC-Anweisungen durch Aufrufe von Unterprogrammen. Prüfe das Ergebnis von Hand; nicht jedes BASIC-Programm lässt sich vollständig übertragen. Siehe auch [BASIC](help:basic).
