# Handprüfung E9 Bildschirm-Editor

Start: `start-bildschirm.command` im Paket (öffnet `arbeitskopie/gfx_demo`, Datei `graphics/title.vcscreen`, C64).

| # | Schritte | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Datei öffnen (Startskript) | Grafik ▸ **Bildschirm**; links die Zeichentafel (256 Zeichen) und die Zeichenfarbe, in der Mitte der Bildschirm mit Rand und Hintergrund, rechts Größe 40 × 25, Farben des Systems, Zeichensatz, Ausgabe | |
| 2 | Zeichen in der Tafel anklicken, dann Zellen anklicken | Das Zeichen steht in der Zelle; rechte Maustaste löscht | |
| 3 | Mit gedrückter Maustaste über Zellen ziehen (auch schnell) | Keine Lücken; ein Rückgängig nimmt den ganzen Zug zurück | |
| 4 | Zeichenfarbe wählen, Zellen setzen | Das Zeichen erscheint in der gewählten Farbe | |
| 5 | Werkzeuge: Rahmen (aufziehen), Füllen, Pipette, Löschen; Tasten D, R, F, T, I, E | Wirkt wie beschriftet; die Pipette nimmt Zeichen und Farbe auf und wechselt zum Zeichenwerkzeug | |
| 6 | Text tippen: Werkzeug Text, Zelle anklicken, schreiben | Buchstaben erscheinen als Bildschirmzeichen, der Cursor rückt vor; Pfeile bewegen ihn; Esc beendet; das Ganze ist ein Rückgängig-Schritt | |
| 7 | Tastatur: Pfeiltasten in den Zellen, Enter | Der Cursor wandert, Enter wendet das Werkzeug an (Rahmen: zweimal Enter); Fußzeile „Spalte x Zeile y · Zeichen n“ | |
| 8 | Rahmen-, Hintergrund- und Zeichenfarbe des Systems ändern | Das Bild folgt sofort | |
| 9 | Zeichensatz wechseln (Auswahlfeld) | Der Bildschirm wird mit dem anderen Satz gezeichnet; „Ohne Zeichensatz“ zeigt keine Zeichen; Rückgängig stellt zurück | |
| 10 | **In Kacheln zerlegen …**: Größe wählen, Zerlegen | Vorschau „n Kacheln, m Patches, x → y Byte“; Zerlegen legt Kachelsatz und Bank neben dem Bildschirm an und öffnet den Kachelsatz im Kachel-Editor | |
| 11 | Eine Bildschirmbank (z. B. `level.vcscreen`) öffnen | Öffnet als Text im Code-Editor (der Karten-Editor kommt später) | |
| 12 | Export …, Speichern (Cmd/Strg+S), Rückfrage bei ungespeichert | Export mit Vorschau; Rückfrage „Ungespeicherter Bildschirm“ | |
| 13 | **Tastatur und VoiceOver:** Zeichentafel und Zellen | Ansage „Zeichen n, $hh, n von 256“ und „Spalte x, Zeile y: Zeichen n, $hh“ | |

Nicht in E9: Bildschirmbanken und Karten (E10, nach 0.3.1), Zoom (der Bildschirm passt sich der Breite an).
