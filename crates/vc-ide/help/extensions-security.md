# Erweiterungen: Sicherheit und Rechte

*Hinweis: Die Verwaltung von Erweiterungen kommt in einer späteren Fassung. Diese Seite beschreibt die vorgesehenen Regeln.*

Erweiterungen bringen Daten (zum Beispiel Vorlagen) oder Werkzeuge (eigene Programme) in VintageCoder. Diese Seite sagt offen, was VintageCoder dabei sichert und was nicht.

## Werkzeuge laufen mit deinen Rechten

Dieses Werkzeug läuft als eigenes Programm mit deinen Rechten. Die angegebenen Rechte werden angezeigt, aber nicht erzwungen. Erlaube es nur aus einer Quelle, der du vertraust.

VintageCoder kann fremde Programme nicht einsperren.

Der Zustimmungsdialog zeigt Name, Herausgeber, Version, Lizenz, Programme, Befehle im Menü und die deklarierten Rechte (`run-programs`, `read-project`, `write-project`, `network-intended`). Ohne deine Zustimmung läuft **nichts**.

## Erweiterungen im Projekt

Beim Öffnen eines Projekts mit `.vintagecoder/extensions/` erscheint ein Banner. Das Projekt öffnet sofort; es werden **nur Datenerweiterungen** geladen, **keine Werkzeuge**, bis du „Vertrauen“ wählst. Das Banner sagt:

Dieses Projekt bringt Erweiterungen mit. Werkzeuge sind abgeschaltet, bis du dem Projekt vertraust. [Vertrauen] [Nicht vertrauen] [Was bedeutet das?]

Das Vertrauen wird **je Projektordner lokal** gespeichert (nicht im Projekt). Der Standard ist „Nicht vertrauen“.

Siehe auch: [ROM-Dateien](hilfe:roms), [Lizenz](hilfe:license).
