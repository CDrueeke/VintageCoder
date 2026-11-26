# PLAY-Tracker (MEGA65)

Raster aus **Schritten × Stimmen** für den BASIC-65-Befehl `PLAY`. Ein Schritt ist ein Sechzehntel; eine Note klingt, bis eine neue Note oder `===` (Pause) folgt. Der Editor ist der [Tracker](help:tracker) im Bereich **Musik**; für `.vcplay`-Dateien hat er sechs Stimmen, keine Reihenfolge der Muster (ein Muster wird einmal gespielt) und Hüllkurven 0 bis 9 statt Instrumenten.

- Neue PLAY-Songs legt der Dialog **Neue Datei** in dieser Version noch nicht an. Öffne ein vorhandenes Stück oder das Beispiel (siehe unten).
- Eingabe wie im Tracker: Tastaturreihen Z S X D C V G B H N J M, Q 2 W 3 E R 5 T 6 Y 7 U I (nach Tastenposition; auf QWERTZ sind Y und Z vertauscht); Entf oder Rücktaste leeren die Zelle, `1` setzt eine Pause, `[` und `]` wählen die Hüllkurve, `,` und `.` ändern die Oktave. **Tempo**, **Zeilen** (Schritte) und **Oktave** stehen in der Werkzeugleiste, die **Lautstärke** rechts unter der Klaviatur.

*[Bild folgt in E13: play-tracker]*

- Hüllkurven 0–9 entsprechen `ENVELOPE n,a,d,s,r,wellenform,pw`.
- **Exportieren …** (Werkzeugleiste) öffnet den Export-Dialog; **In Projekt speichern** schreibt `build/export/NAME.bas`: `VOL`, `TEMPO`, `ENVELOPE` und je Stimme ein oder mehrere `PLAY "V1T0O4QC…"`-Strings (T Hüllkurve, O Oktave, W/H/Q/I/S Dauer, `.` punktiert, R Pause).
- Längen, die sich nicht als eine Note schreiben lassen (z. B. 5 Schritte), werden als wiederholter Anschlag ausgegeben.

Hinweis: Das Vorhören in der App ist eine Näherung; der erzeugte BASIC-Text wurde nicht auf echter Hardware geprüft.

## Beispiel
`examples/mega65/play_demo/music/tune_play.vcplay` ist dasselbe Musikstück wie `tune4` (Tracker mit vier SIDs), umgesetzt auf sechs PLAY-Stimmen: Melodie, Bass, drei Arpeggio-Stimmen und Snare, 64 Schritte, dieselben vier Hüllkurven. Im Projekt öffnen (Gruppe „PLAY-Tracker“), mit der Leertaste anhören und mit `vcc media export` als BASIC-Programm ausgeben.
