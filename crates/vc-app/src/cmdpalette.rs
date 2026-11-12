// SPDX-License-Identifier: GPL-3.0-or-later
//! Befehlspalette (Cmd/Strg+P): Overlay mit Suchfeld über der ganzen Oberfläche. Sie bietet dieselben Befehle wie die Menüleiste
//! (Tabelle `nativemenu::COMMANDS`, kein eigener Befehlssatz), dazu Dateien des Projekts und die Studios.
//! Bedienung nur mit der Tastatur möglich: Pfeile wählen, Enter führt aus, Esc schließt. Das Suchen und Ordnen (oberer Teil)
//! kennt keine Oberfläche und ist getestet; die Anwendung (`app.rs`) liefert die Einträge und führt die Auswahl aus.
use crate::nativemenu::Cmd;
use crate::ui::tokens::{t, tint};
use crate::ui::widgets::mono_chip;
use eframe::egui::{self, RichText};

/// Was ein Eintrag auslöst
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// Menübefehl
    Cmd(Cmd),
    /// Datei des Projekts öffnen (relativer Pfad)
    File(String),
    /// Diskettenabbild wählen und öffnen (Disk-Studio)
    PickImage,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub label: String,
    /// Menü bzw. Art (Datei, Projekt, Studio …)
    pub group: String,
    /// Tastenkürzel zur Anzeige (schon in der Schreibweise des Systems)
    pub keys: String,
    pub enabled: bool,
    /// Nur anzeigen, wenn gesucht wird (Dateien: sonst wäre die Liste ohne Eingabe endlos)
    pub only_when_searching: bool,
    pub item: Item,
}

/// Einträge für Menübefehle und Studios in dieser Auswahl: Befehle und Studios, die im Release fehlen, tauchen nicht auf
pub fn command_entries(f: &vc_ide::features::Features, ms: &crate::nativemenu::MenuState, mac: bool) -> Vec<Entry> {
    let mut v: Vec<Entry> = crate::nativemenu::COMMANDS.iter().filter(|i| crate::nativemenu::visible_with(i.cmd, f))
        .map(|i| Entry { label: i.label.into(), group: i.group.into(), keys: crate::nativemenu::keys_text(i.keys, mac), enabled: crate::nativemenu::enabled(i.cmd, ms), only_when_searching: false, item: Item::Cmd(i.cmd) }).collect();
    for (id, n) in [("sprite", "Sprites"), ("char", "Zeichensatz"), ("screen", "Bildschirm"), ("tiles", "Kacheln"), ("tracker", "Tracker"), ("palette", "Palette")] {
        if !f.studio(id) { continue; }
        v.push(Entry { label: format!("Studio: {n} – neue Datei …"), group: "Studio".into(), keys: String::new(), enabled: ms.project, only_when_searching: false, item: Item::Cmd(Cmd::NewMedia) });
    }
    if f.studio("disk") { v.push(Entry { label: "Studio: Disk-Image öffnen …".into(), group: "Studio".into(), keys: String::new(), enabled: ms.project, only_when_searching: false, item: Item::PickImage }); }
    v
}

// ---------------------------------------------------------------------------------------------
// Suchen und Ordnen (ohne Oberfläche)
// ---------------------------------------------------------------------------------------------

/// Trefferwert eines Eintrags für die Suche (kleiner = besser); `None` = kein Treffer. Jedes Wort der Eingabe muss vorkommen.
pub fn score(e: &Entry, query: &str) -> Option<u32> {
    let label = e.label.to_lowercase();
    let group = e.group.to_lowercase();
    let mut total = 0;
    for w in query.to_lowercase().split_whitespace() {
        let s = if label.starts_with(w) { 0 }
            else if label.split(|c: char| !c.is_alphanumeric()).any(|p| p.starts_with(w)) { 1 }
            else if label.contains(w) { 2 }
            else if group.contains(w) || e.keys.to_lowercase().contains(w) { 3 }
            else if is_subsequence(w, &label) { 4 }
            else { return None };
        total += s;
    }
    Some(total)
}

fn is_subsequence(needle: &str, hay: &str) -> bool {
    let mut it = hay.chars();
    needle.chars().all(|c| it.any(|h| h == c))
}

/// Indizes der anzuzeigenden Einträge in Reihenfolge der Anzeige: ohne Eingabe alle (außer reinen Suchtreffern) in der Reihenfolge der
/// Liste, mit Eingabe die Treffer nach Wert, wählbare vor gesperrten, bei Gleichstand in Reihenfolge der Liste.
pub fn rank(entries: &[Entry], query: &str, limit: usize) -> Vec<usize> {
    if query.trim().is_empty() { return entries.iter().enumerate().filter(|(_, e)| !e.only_when_searching).map(|(i, _)| i).take(limit).collect(); }
    let mut v: Vec<(u32, bool, usize)> = entries.iter().enumerate().filter_map(|(i, e)| score(e, query).map(|s| (s, !e.enabled, i))).collect();
    v.sort();
    v.into_iter().map(|x| x.2).take(limit).collect()
}

/// Nächster wählbarer Eintrag ab `from` in Richtung `dir` (+1/−1) innerhalb der Treffer, ohne Umbruch am Rand
pub fn step(entries: &[Entry], hits: &[usize], from: usize, dir: i32) -> usize {
    let mut i = from as i32;
    loop {
        let n = i + dir;
        if n < 0 || n as usize >= hits.len() { return from; }
        i = n;
        if entries[hits[i as usize]].enabled { return i as usize; }
    }
}

/// Was Enter auslöst: der markierte Eintrag, sofern er wählbar ist
pub fn activate(entries: &[Entry], hits: &[usize], sel: usize) -> Option<Item> {
    hits.get(sel).map(|&i| &entries[i]).filter(|e| e.enabled).map(|e| e.item.clone())
}

// ---------------------------------------------------------------------------------------------
// Oberfläche
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Palette { open: bool, query: String, sel: usize, scroll_to_sel: bool }

const LIMIT: usize = 60;

impl Palette {
    pub fn is_open(&self) -> bool { self.open }
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.query.clear();
        self.sel = 0;
        self.scroll_to_sel = true;
    }

    /// Zeichnet das Overlay; liefert den gewählten Eintrag (die Palette schließt sich dann)
    pub fn show(&mut self, ctx: &egui::Context, entries: &[Entry]) -> Option<Item> {
        if !self.open { return None; }
        let k = t();
        let hits = rank(entries, &self.query, LIMIT);
        // erster wählbarer Treffer, wenn die Auswahl ungültig ist
        if self.sel >= hits.len() || !entries[hits[self.sel]].enabled { self.sel = hits.iter().position(|&i| entries[i].enabled).unwrap_or(0); }
        let (down, up, enter, esc) = ctx.input_mut(|i| (
            i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown), i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter), i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)));
        if esc { self.open = false; return None; }
        if down { self.sel = step(entries, &hits, self.sel, 1); self.scroll_to_sel = true; }
        if up { self.sel = step(entries, &hits, self.sel, -1); self.scroll_to_sel = true; }
        let mut chosen: Option<Item> = None;
        if enter { chosen = activate(entries, &hits, self.sel); }
        let screen = ctx.content_rect();
        let width = 580.0_f32.min(screen.width() - 32.0).max(280.0);
        let area = egui::Area::new(egui::Id::new("cmd_palette")).order(egui::Order::Foreground).fixed_pos(egui::pos2(screen.center().x - width / 2.0, screen.top() + 70.0)).show(ctx, |ui| {
            egui::Frame::new().fill(k.bg3).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.45))).corner_radius(8.0).inner_margin(egui::Margin::same(10))
                .shadow(egui::Shadow { offset: [0, 6], blur: 24, spread: 0, color: egui::Color32::from_black_alpha(90) }).show(ui, |ui| {
                ui.set_width(width - 20.0);
                let r = ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Befehl, Datei oder Studio suchen …").desired_width(f32::INFINITY).font(egui::TextStyle::Body).margin(egui::Margin::symmetric(8, 6)));
                if r.changed() { self.sel = 0; self.scroll_to_sel = true; }
                r.request_focus();
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(360.0).auto_shrink([false, true]).show(ui, |ui| {
                    if hits.is_empty() { ui.label(RichText::new("Keine Treffer").color(k.mute)); }
                    for (pos, &i) in hits.iter().enumerate() {
                        let e = &entries[i];
                        let selected = pos == self.sel;
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), if e.enabled { egui::Sense::click() } else { egui::Sense::hover() });
                        if selected { ui.painter().rect_filled(rect, 4.0, tint(k.cy)); }
                        else if resp.hovered() && e.enabled { ui.painter().rect_filled(rect, 4.0, k.bg2); }
                        let col = if !e.enabled { k.dim } else if selected { k.cy } else { k.fg };
                        ui.painter().text(rect.left_center() + egui::vec2(8.0, 0.0), egui::Align2::LEFT_CENTER, &e.label, egui::FontId::proportional(13.0), col);
                        let mut x = rect.right() - 8.0;
                        if !e.keys.is_empty() {
                            let g = ui.painter().layout_no_wrap(e.keys.clone(), egui::FontId::monospace(11.0), k.mute);
                            let w = g.size().x + 14.0;
                            let br = egui::Rect::from_min_size(egui::pos2(x - w, rect.center().y - 9.0), egui::vec2(w, 18.0));
                            ui.painter().rect_filled(br, 9.0, k.bg2);
                            ui.painter().rect_stroke(br, 9.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
                            ui.painter().galley(br.center() - g.size() / 2.0, g, k.mute);
                            x -= w + 8.0;
                        }
                        ui.painter().text(egui::pos2(x, rect.center().y), egui::Align2::RIGHT_CENTER, &e.group, egui::FontId::proportional(11.0), k.dim);
                        if selected && self.scroll_to_sel { ui.scroll_to_rect(rect, None); }
                        if resp.clicked() { chosen = Some(e.item.clone()); }
                    }
                    self.scroll_to_sel = false;
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for (key, what) in [("↑↓", "wählen"), ("Enter", "ausführen"), ("Esc", "schließen")] { mono_chip(ui, key, k.mute); ui.label(RichText::new(what).small().color(k.dim)); }
                });
            });
        });
        // Klick außerhalb schließt
        if ctx.input(|i| i.pointer.any_pressed()) {
            if let Some(p) = ctx.input(|i| i.pointer.interact_pos()) { if !area.response.rect.contains(p) { self.open = false; } }
        }
        if chosen.is_some() { self.open = false; }
        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(label: &str, group: &str, keys: &str, enabled: bool) -> Entry {
        Entry { label: label.into(), group: group.into(), keys: keys.into(), enabled, only_when_searching: false, item: Item::File(label.into()) }
    }

    fn list() -> Vec<Entry> {
        vec![e("Projekt bauen", "Projekt", "F7", true), e("Startprogramm starten", "Projekt", "F5", true), e("Speichern", "Datei", "Strg+S", false),
             e("Aktive Datei bauen", "Projekt", "Umschalt+F7", true), e("Einstellungen …", "VintageCoder", "Strg+,", true)]
    }

    #[test]
    fn ohne_eingabe_alle_in_listenreihenfolge() {
        assert_eq!(rank(&list(), "  ", 10), vec![0, 1, 2, 3, 4]);
        assert_eq!(rank(&list(), "", 2), vec![0, 1]);
    }

    #[test]
    fn dateien_nur_bei_suche() {
        let mut v = list();
        v.push(Entry { only_when_searching: true, ..e("src/main.asm", "Datei im Projekt", "", true) });
        assert!(!rank(&v, "", 10).contains(&5));
        assert_eq!(rank(&v, "main", 10), vec![5]);
    }

    #[test]
    fn suche_ordnet_nach_trefferart() {
        // „bau“: Wortanfang in „Projekt bauen“ (1) und „Aktive Datei bauen“ (1), Reihenfolge der Liste bleibt
        assert_eq!(rank(&list(), "bau", 10), vec![0, 3]);
        // Anfang der Beschriftung schlägt Wortanfang
        assert_eq!(rank(&list(), "akt", 10), vec![3]);
        assert_eq!(rank(&list(), "start", 10), vec![1]);
    }

    #[test]
    fn mehrere_woerter_gruppe_und_kuerzel() {
        assert_eq!(rank(&list(), "datei bauen", 10), vec![3]);
        assert_eq!(rank(&list(), "projekt", 10), vec![0, 1, 3]);   // Gruppe
        assert_eq!(rank(&list(), "f7", 10), vec![0, 3]);           // Kürzel
        assert!(rank(&list(), "xyz", 10).is_empty());
    }

    #[test]
    fn unscharfe_treffer_und_gesperrte_zuletzt() {
        assert_eq!(rank(&list(), "pbn", 10), vec![0]);   // p…b…n in „projekt bauen“
        // gleicher Wert: gesperrte nach den wählbaren
        let v = vec![e("Speichern", "Datei", "", false), e("Speichern unter", "Datei", "", true)];
        assert_eq!(rank(&v, "speich", 10), vec![1, 0]);
    }

    #[test]
    fn pfeile_ueberspringen_gesperrte_und_bleiben_am_rand() {
        let v = list();
        let hits = rank(&v, "", 10);
        assert_eq!(step(&v, &hits, 1, 1), 3);    // Eintrag 2 (Speichern) ist gesperrt
        assert_eq!(step(&v, &hits, 3, -1), 1);
        assert_eq!(step(&v, &hits, 0, -1), 0);
        assert_eq!(step(&v, &hits, 4, 1), 4);
    }

    #[test]
    fn enter_fuehrt_den_markierten_befehl_aus() {
        let v = vec![e("Projekt bauen", "Projekt", "F7", true), e("Speichern", "Datei", "", false), e("Aktive Datei bauen", "Projekt", "", true)];
        // Suche „bau“, zweiter Treffer markiert
        let hits = rank(&v, "bau", 10);
        assert_eq!(hits, vec![0, 2]);
        assert_eq!(activate(&v, &hits, 1), Some(Item::File("Aktive Datei bauen".into())));
        // ohne Eingabe: der erste wählbare Eintrag, gesperrte lösen nichts aus
        let all = rank(&v, "", 10);
        assert_eq!(activate(&v, &all, 0), Some(Item::File("Projekt bauen".into())));
        assert_eq!(activate(&v, &all, 1), None);
        // keine Treffer oder Auswahl außerhalb der Liste
        assert_eq!(activate(&v, &rank(&v, "xyz", 10), 0), None);
        assert_eq!(activate(&v, &all, 9), None);
    }

    /// Kein Menü- und kein Palettenbefehl zeigt auf ein abgewähltes Studio (Konzept Release-Auswahl, Risiko 1)
    #[test]
    fn keine_befehle_fuer_abgewaehlte_studios() {
        use vc_ide::features::Features;
        use crate::nativemenu::{needs_studio, visible_menus, visible_with, MenuState, COMMANDS, M};
        let studio_words = ["sprite", "zeichensatz", "song", "studio", "kachel", "tracker", "grafik-editor", "disk", "bildschirm"];
        // Tabelle vollständig: jeder Befehl, dessen Beschriftung auf ein Studio deutet, hängt an einem Studio
        for c in COMMANDS { if studio_words.iter().any(|w| c.label.to_lowercase().contains(w)) { assert!(!needs_studio(c.cmd).is_empty(), "{:?} „{}“ nennt ein Studio, hat aber kein needs_studio", c.cmd, c.label); } }
        let sets = [r#"{"format":1,"studios":["sprite"]}"#, r#"{"format":1,"studios":[]}"#, r#"{"format":1,"ide":true,"studios":["disk","memmap"]}"#, r#"{"format":1,"studios":["char","tracker"]}"#];
        let ms = MenuState { project: true, studio: true, detached: true, ..Default::default() };
        for text in sets {
            let f = Features::from_json(text);
            let mut shown: Vec<Cmd> = vec![];
            fn collect(items: &[M], out: &mut Vec<Cmd>) { for m in items { match m { M::Item(c) | M::Check(c) => out.push(*c), M::Sub(_, s) => collect(s, out), _ => {} } } }
            for (_, items) in visible_menus(&f) { collect(&items, &mut shown); }
            for e in command_entries(&f, &ms, false) { if let Item::Cmd(c) = e.item { shown.push(c); } }
            for c in shown {
                let n = needs_studio(c);
                assert!(visible_with(c, &f) && (n.is_empty() || n.iter().any(|s| f.studio(s))), "{text}: {c:?} zeigt auf ein abgewähltes Studio");
            }
            // Studio-Einträge der Palette nur für gewählte Studios
            for e in command_entries(&f, &ms, false) {
                for (id, n) in [("sprite", "Sprites"), ("char", "Zeichensatz"), ("screen", "Bildschirm"), ("tiles", "Kacheln"), ("tracker", "Tracker"), ("palette", "Palette")] { if e.label == format!("Studio: {n} – neue Datei …") { assert!(f.studio(id), "{text}: {}", e.label); } }
                if e.item == Item::PickImage { assert!(f.studio("disk")); }
            }
        }
        // ohne jedes Medien-Studio: Menü „Fenster“ entfällt, „Neu: Sprite, …“ und „Exportieren“ ebenso
        let f = Features::from_json(r#"{"format":1,"studios":["disk"]}"#);
        assert!(visible_menus(&f).iter().all(|(n, _)| *n != "Fenster"));
        assert!(!visible_with(Cmd::NewMedia, &f) && !visible_with(Cmd::Export, &f) && !visible_with(Cmd::StdCharset, &f));
        assert!(!visible_with(Cmd::StdCharset, &Features::from_json(r#"{"format":1,"studios":["sprite"]}"#)) && visible_with(Cmd::NewMedia, &Features::from_json(r#"{"format":1,"studios":["sprite"]}"#)));
        // alles an: jeder Befehl sichtbar
        assert!(COMMANDS.iter().all(|c| visible_with(c.cmd, &Features::all())));
    }

    #[test]
    fn limit_und_tabelle_der_befehle() {
        assert_eq!(rank(&list(), "", 3).len(), 3);
        // jeder Befehl der Menüleiste steht genau einmal in der Tabelle
        for c in crate::nativemenu::COMMANDS { assert_eq!(crate::nativemenu::COMMANDS.iter().filter(|x| x.cmd == c.cmd).count(), 1); }
        assert_eq!(crate::nativemenu::keys_text("Cmd+Umschalt+Y", true), "⇧⌘Y");
        assert_eq!(crate::nativemenu::keys_text("Cmd+Umschalt+Y", false), "Strg+Umschalt+Y");
        assert_eq!(crate::nativemenu::keys_text("F7", false), "F7");
    }
}
