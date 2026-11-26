# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche zum Umwandeln von BASIC im Editor (In Labels umwandeln, In Nummern umwandeln, Neu nummerieren).
# Fachbegriffe: Modus = Zeilennummern oder Labels; Label = Sprungmarke @name. Meldungen B160 ff. kommen fertig aus dem Kern.

basicconv-nur-basic = Das geht nur in einer BASIC-Datei des Projekts.
basicconv-titel-labels = In Labels umwandeln
basicconv-titel-numbers = In Nummern umwandeln
basicconv-titel-renumber = Neu nummerieren
basicconv-erklaerung-labels = Ersetzt Zeilennummern und die Sprungziele dahinter durch Labels (@name). Namen, die du selbst vergeben hast, bleiben erhalten; neue Namen werden aus Kommentaren abgeleitet und mit einer Markerzeile gekennzeichnet.
basicconv-erklaerung-numbers = Vergibt Zeilennummern nach Start und Schrittweite und ersetzt die Labels in den Sprungzielen durch Nummern. Die Namen bleiben in Markerzeilen (;@name) erhalten, damit du später wieder in Labels umwandeln kannst.
basicconv-erklaerung-renumber = Vergibt die Zeilennummern neu nach Start und Schrittweite und passt alle Sprungziele an. Mit einem Bereich werden nur die Zeilen von Nummer bis Nummer neu nummeriert.
basicconv-modus = Modus: { $modus }
basicconv-modus-numbers = Zeilennummern
basicconv-modus-labels = Labels
basicconv-modus-mixed = gemischt
basicconv-modus-undetermined = noch unbestimmt
basicconv-nummerierung = Start { $start }, Schrittweite { $step }
basicconv-includes = Eingebundene Dateien mitumwandeln ({ $n })
basicconv-bereich = Bereich (Zeilennummern, optional)
basicconv-bereich-von = Von
basicconv-bereich-bis = Bis
basicconv-bereich-hinweis = Leer lassen, um das ganze Programm neu zu nummerieren.
basicconv-bereich-beide = Bitte „Von“ und „Bis“ angeben oder beide leer lassen.
basicconv-meldungen = Meldungen
basicconv-vorschau = Vorschau
basicconv-rechnet = Vorschau wird berechnet …
basicconv-nichts = Nichts zu ändern.
basicconv-summe =
    { $zeilen ->
        [one] 1 Zeile ändert sich
       *[other] { $zeilen } Zeilen ändern sich
    }, { $labels ->
        [0] keine neuen Labels
        [one] 1 neues Label
       *[other] { $labels } neue Labels
    }.
basicconv-mehr = … und { $n } weitere Zeilen.
basicconv-anwenden = Anwenden
basicconv-fertig = Umgewandelt: { $zeilen } Zeilen in { $dateien } Datei(en). Rückgängig macht die Änderung in dieser Datei wieder rückgängig.
basicconv-include-ungespeichert = Eingebundene Dateien mit ungespeicherten Änderungen: { $dateien }. Erst speichern oder die Option „Eingebundene Dateien mitumwandeln“ ausschalten.
basicconv-reiter-gewechselt = Der Reiter hat gewechselt. Bitte den Dialog schließen und neu öffnen.
basicconv-text-geaendert = Der Text hat sich geändert; die Vorschau wurde neu berechnet.
basicconv-medien = Eingebundene Medien: { $n } (werden nicht verändert)
