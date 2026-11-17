// SPDX-License-Identifier: GPL-3.0-or-later
//! Markdown-Schreibhilfen (Bearbeiten ▸ „Markdown: …“): Fett, Kursiv, Code, Durchgestrichen, Link, Bild, Überschriftsebene, Listen, Zitat, Aufgaben, Tabellen.
//! Die Textfunktionen stehen in `vc_md::edit` (reine Funktionen auf Zeichenindizes); hier werden sie auf den Editor des aktiven Reiters angewendet, jede als **ein** Schritt für Rückgängig.
use super::*;
use vc_md::edit as me;

/// Zeichenindex im Gesamttext (Zeilen durch `\n` verbunden) → Zeile und Spalte
fn pos_von(text: &str, idx: usize) -> Pos {
    let (mut line, mut col) = (0, 0);
    for (i, ch) in text.chars().enumerate() { if i == idx { break } if ch == '\n' { line += 1; col = 0 } else { col += 1 } }
    Pos { line, col }
}
fn idx_von(lines: &[String], p: Pos) -> usize { lines.iter().take(p.line).map(|l| l.chars().count() + 1).sum::<usize>() + p.col }

impl App {
    /// Ist der aktive Reiter eine Markdown-Datei (Endung `.md` oder von Hand als Markdown behandelt)?
    pub(crate) fn md_aktiv(&self) -> bool { self.name().is_some_and(|n| self.editoren.contains_key(&n) && (n.to_lowercase().ends_with(".md") || self.md_als.contains(&n))) }

    /// Eine Schreibhilfe auf den aktiven Editor anwenden. `f` bekommt Text, Auswahlanfang und -ende (Zeichenindizes) und liefert die Änderung (oder `None`).
    fn md_anwenden(&mut self, f: impl FnOnce(&str, usize, usize) -> Option<me::Edit>) {
        let Some(e) = self.ed() else { return };
        let text = e.text(); let (a, b) = match e.selection() { Some((x, y)) => (idx_von(&e.lines, x), idx_von(&e.lines, y)), None => { let i = idx_von(&e.lines, e.cursor); (i, i) } };
        let Some(ed) = f(&text, a, b) else { return };
        if ed.text == text { return }
        let ende = Pos { line: e.lines.len() - 1, col: e.line_len(e.lines.len() - 1) };
        e.replace(Pos { line: 0, col: 0 }, ende, &ed.text);
        let (pa, pb) = (pos_von(&ed.text, ed.a), pos_von(&ed.text, ed.b)); e.set_cursor(pa, false); if pb != pa { e.set_cursor(pb, true) }
    }

    pub(crate) fn md_befehl(&mut self, c: Cmd) -> Task<Msg> {
        if c == Cmd::MdAlsMarkdown {
            if let Some(n) = self.name().filter(|n| self.editoren.contains_key(n) && !n.to_lowercase().ends_with(".md")) { if !self.md_als.remove(&n) { self.md_als.insert(n); } }
            return Task::none();
        }
        if !self.md_aktiv() { return Task::none() }
        match c {
            Cmd::MdFett => self.md_anwenden(|t, a, b| Some(me::toggle_wrap(t, a, b, "**"))),
            Cmd::MdKursiv => self.md_anwenden(|t, a, b| Some(me::toggle_wrap(t, a, b, "*"))),
            Cmd::MdCode => self.md_anwenden(|t, a, b| Some(me::code_toggle(t, a, b))),
            Cmd::MdDurchgestrichen => self.md_anwenden(|t, a, b| Some(me::toggle_wrap(t, a, b, "~~"))),
            Cmd::MdLink => self.md_anwenden(|t, a, b| Some(me::link_insert(t, a, b))),
            Cmd::MdUeberschriftTiefer => self.md_anwenden(|t, a, b| Some(me::heading_shift(t, a, b, 1))),
            Cmd::MdUeberschriftHoeher => self.md_anwenden(|t, a, b| Some(me::heading_shift(t, a, b, -1))),
            Cmd::MdAufzaehlung => self.md_anwenden(|t, a, b| Some(me::list_toggle(t, a, b, me::ListKind::Bullet))),
            Cmd::MdNummerierung => self.md_anwenden(|t, a, b| Some(me::list_toggle(t, a, b, me::ListKind::Numbered))),
            Cmd::MdAufgaben => self.md_anwenden(|t, a, b| Some(me::list_toggle(t, a, b, me::ListKind::Task))),
            Cmd::MdZitat => self.md_anwenden(|t, a, b| Some(me::list_toggle(t, a, b, me::ListKind::Quote))),
            Cmd::MdAufgabeUmschalten => self.md_anwenden(|t, a, _| me::toggle_task(t, a)),
            Cmd::MdTabelleNeu => self.md_anwenden(|t, a, _| { let blk = me::table_new(3, 2, true, 3); Some(me::insert_block(t, a, &blk, 0)) }),
            Cmd::MdTabelleAusrichten => self.md_anwenden(|t, a, _| me::table_align(t, a)),
            Cmd::MdBild => {
                // PNG im Projekt wählen (Dialog des Systems); das Ergebnis kommt als Nachricht zurück
                let start = self.root.clone();
                return Task::perform(async move { rfd::AsyncFileDialog::new().set_title("Bild einfügen (PNG im Projekt)").set_directory(start).add_filter("PNG", &["png"]).pick_file().await.map(|f| f.path().to_path_buf()) }, Msg::MdBildGewaehlt);
            }
            _ => {}
        }
        Task::none()
    }
    pub(crate) fn md_bild_einfuegen(&mut self, p: Option<PathBuf>) {
        let Some(p) = p else { return };
        let rel = p.strip_prefix(&self.root).map(|r| vc_core::paths::slashes(&r.to_string_lossy())).unwrap_or_else(|_| p.to_string_lossy().to_string());
        self.md_anwenden(|t, a, b| Some(me::image_insert(t, a, b, &rel)));
    }
}
