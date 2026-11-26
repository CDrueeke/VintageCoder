# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.
# Machine-translated, to be reviewed.

basicconv-nur-basic = This only works in a BASIC file of the project.
basicconv-titel-labels = Convert to labels
basicconv-titel-numbers = Convert to line numbers
basicconv-titel-renumber = Renumber
basicconv-erklaerung-labels = Replaces line numbers and the jump targets behind them with labels (@name). Names you assigned yourself are kept; new names are derived from comments and marked with a marker line.
basicconv-erklaerung-numbers = Assigns line numbers by start and step and replaces the labels in jump targets with numbers. The names stay in marker lines (;@name), so you can convert back to labels later.
basicconv-erklaerung-renumber = Assigns the line numbers anew by start and step and adjusts all jump targets. With a range, only the lines from one number to another are renumbered.
basicconv-modus = Mode: { $modus }
basicconv-modus-numbers = line numbers
basicconv-modus-labels = labels
basicconv-modus-mixed = mixed
basicconv-modus-undetermined = not yet determined
basicconv-nummerierung = Start { $start }, step { $step }
basicconv-includes = Convert included files too ({ $n })
basicconv-bereich = Range (line numbers, optional)
basicconv-bereich-von = From
basicconv-bereich-bis = To
basicconv-bereich-hinweis = Leave empty to renumber the whole program.
basicconv-bereich-beide = Please enter both “From” and “To”, or leave both empty.
basicconv-meldungen = Messages
basicconv-vorschau = Preview
basicconv-rechnet = Calculating the preview …
basicconv-nichts = Nothing to change.
basicconv-summe =
    { $zeilen ->
        [one] 1 line changes
       *[other] { $zeilen } lines change
    }, { $labels ->
        [0] no new labels
        [one] 1 new label
       *[other] { $labels } new labels
    }.
basicconv-mehr = … and { $n } more lines.
basicconv-anwenden = Apply
basicconv-fertig = Converted: { $zeilen } lines in { $dateien } file(s). Undo reverts the change in this file.
basicconv-include-ungespeichert = Included files with unsaved changes: { $dateien }. Save first or switch off the option “Convert included files too”.
basicconv-reiter-gewechselt = The tab has changed. Please close the dialog and open it again.
basicconv-text-geaendert = The text has changed; the preview was recalculated.
basicconv-medien = Included media: { $n } (they are not changed)
