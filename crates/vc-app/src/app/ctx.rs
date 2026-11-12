// SPDX-License-Identifier: GPL-3.0-or-later
//! Inhalte von Inspektor und Fußpanel je nach aktivem Dokument.
//!
//! Der **Kontext** (Quelltext, Studio, Debug) bestimmt, welche Reiter der Inspektor zeigt. Jeder Kontext merkt sich seinen aktiven
//! Reiter; ein Wechsel des Dokuments im Editorbereich ändert nur dann den Kontext, wenn die Art wechselt (Quelltext ↔ Studio).
//! Alles, was entscheidet (Kontext, Zahlenmarken, Verhalten des Fußpanels beim Bauen, aktuelles Label, Fundstellen), sind reine
//! Funktionen mit Tests am Ende der Datei.
use super::*;
use crate::ui::tokens::t;
use vc_asm::OutlineKind;

// ---- Kontext ----------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Ctx { Code, Studio, Debug }

/// Was im Editorbereich gerade aktiv ist
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Focus { Doc, Studio, Other }

/// Kontext des Inspektors. Im Debug-Arbeitsplatz gilt immer Debug. Ist weder ein Quelltext noch ein Studio aktiv (Willkommen,
/// Übersicht, Einstellungen, nichts offen), bleibt der bisherige Kontext: der Inspektor springt nicht ohne Grund.
pub(super) fn context_for(debug: bool, focus: Focus, prev: Ctx) -> Ctx {
    if debug { return Ctx::Debug; }
    match focus {
        Focus::Doc => Ctx::Code,
        Focus::Studio => Ctx::Studio,
        Focus::Other => if prev == Ctx::Debug { Ctx::Code } else { prev },
    }
}

/// Reiter des Inspektors je Kontext, in der Reihenfolge der Leiste
pub(super) fn inspector_tabs(c: Ctx) -> Vec<Tab> {
    inspector_tabs_with(c, &vc_ide::features::features())
}

/// Wie [`inspector_tabs`] für eine bestimmte Auswahl (die Memory Map ist ein Studio und fehlt, wenn es abgewählt ist)
pub(super) fn inspector_tabs_with(c: Ctx, f: &vc_ide::features::Features) -> Vec<Tab> {
    let mut v = match c {
        Ctx::Code => vec![Tab::Outline, Tab::MemMap, Tab::Ref, Tab::Help],
        Ctx::Studio => vec![Tab::StudioInfo, Tab::MemMap, Tab::Help],
        Ctx::Debug => vec![Tab::Regs, Tab::Watch, Tab::Breaks, Tab::Help],
    };
    if !f.studio("memmap") { v.retain(|t| *t != Tab::MemMap); }
    v
}

/// Reiter des Fußpanels je Arbeitsplatz
pub(super) fn footer_tabs(debug: bool) -> Vec<Tab> { footer_tabs_with(debug, &vc_ide::features::features()) }

/// Wie [`footer_tabs`] für eine bestimmte Auswahl: „Live-Bild“ gibt es nur mit einer Anbindung mit `caps.live_image` (Ultimate)
pub(super) fn footer_tabs_with(debug: bool, f: &vc_ide::features::Features) -> Vec<Tab> {
    if debug {
        let mut v = vec![Tab::Disasm, Tab::Memory, Tab::Emulator, Tab::Snapshot, Tab::Live, Tab::Output, Tab::Problems];
        if !f.any_live_image() { v.retain(|t| *t != Tab::Live); }
        v
    } else { vec![Tab::Output, Tab::BuildLog, Tab::Problems, Tab::Tests] }
}

// ---- Zahlenmarken -----------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum BadgeKind { Err, Warn, Ok }

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct Badge { pub text: String, pub kind: BadgeKind }

/// Marke auf dem Reiter „Probleme“: die Zahl der Fehler (rot), sonst die der Warnungen (gelb), sonst keine
pub(super) fn problems_badge(errors: usize, warnings: usize) -> Option<Badge> {
    if errors > 0 { Some(Badge { text: errors.to_string(), kind: BadgeKind::Err }) }
    else if warnings > 0 { Some(Badge { text: warnings.to_string(), kind: BadgeKind::Warn }) }
    else { None }
}

/// Marke auf dem Reiter „Tests“: „bestanden/alle“, grün, bei Fehlschlägen rot; ohne Ergebnisse keine
pub(super) fn tests_badge(passed: usize, total: usize, failed: usize) -> Option<Badge> {
    if total == 0 { return None; }
    Some(Badge { text: format!("{passed}/{total}"), kind: if failed > 0 { BadgeKind::Err } else { BadgeKind::Ok } })
}

/// Reiterbeschriftung: Name, dahinter die Marke in ihrer Farbe
pub(super) fn titled(name: &str, badge: Option<Badge>) -> egui::WidgetText {
    let Some(b) = badge else { return name.into() };
    let k = t();
    let font = egui::FontId::proportional(13.0);
    let mut job = egui::text::LayoutJob::default();
    job.append(name, 0.0, egui::TextFormat { font_id: font.clone(), color: egui::Color32::PLACEHOLDER, valign: egui::Align::Center, ..Default::default() });
    let col = match b.kind { BadgeKind::Err => k.rd, BadgeKind::Warn => k.am, BadgeKind::Ok => k.gr };
    job.append(&b.text, 6.0, egui::TextFormat { font_id: egui::FontId::monospace(11.0), color: col, valign: egui::Align::Center, ..Default::default() });
    job.into()
}

// ---- Fußpanel beim Bauen ----------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FooterAct {
    /// Probleme zeigen und das Fußpanel dafür öffnen
    ShowProblems,
    /// Bauprotokoll nach vorn holen, ohne ein eingeklapptes Fußpanel zu öffnen
    SelectBuildLog,
    Nothing,
}

/// Was das Fußpanel nach einem Bau tut: Fehler öffnen es immer; ein fehlerfreier Bau zeigt höchstens das Protokoll, und nur, wenn das
/// Panel ohnehin offen ist (hat der Nutzer es eingeklappt, bleibt es zu).
pub(super) fn build_footer_action(errors: usize, footer_visible: bool) -> FooterAct {
    if errors > 0 { FooterAct::ShowProblems } else if footer_visible { FooterAct::SelectBuildLog } else { FooterAct::Nothing }
}

// ---- Gliederung -------------------------------------------------------------------------------------------------------------------

/// Eintrag der Gliederung: Name, Zeile (1-basiert), Art
pub(super) type OutlineRow = (String, usize, OutlineKind);

/// Das Label, in dem der Cursor steht: das letzte Label (keine Konstante, kein Makro) in oder vor der Cursorzeile (1-basiert)
pub(super) fn current_label(rows: &[OutlineRow], cursor_line: usize) -> Option<usize> {
    rows.iter().enumerate().filter(|(_, r)| r.2 == OutlineKind::Label && r.1 <= cursor_line).map(|(i, _)| i).last()
}

/// Gliederung für BASIC: Zeilen, die Ziel von GOTO/GOSUB/THEN/ELSE/ON…GOTO/GOSUB (u. a.) sind, und Zeilen mit REM-Überschrift.
/// Name: Zeilennummer, bei REM mit dem Kommentartext. Zeile = Textzeile (1-basiert).
pub(super) fn basic_outline(text: &str) -> Vec<OutlineRow> {
    let lines: Vec<(usize, Option<u32>, &str)> = text.lines().enumerate().map(|(i, l)| {
        let t = l.trim_start();
        let d = t.bytes().take_while(u8::is_ascii_digit).count();
        (i + 1, t[..d].parse().ok(), t[d..].trim_start())
    }).collect();
    let targets: std::collections::HashSet<u32> = lines.iter().flat_map(|(_, _, body)| basic_targets(body)).collect();
    let mut out = vec![];
    for (line, num, body) in &lines {
        let Some(n) = num else { continue };
        if let Some(rem) = basic_rem(body) {
            let rem = rem.trim_matches(|c: char| c.is_whitespace() || "-=*_".contains(c));
            if !rem.is_empty() { out.push((format!("{n} {rem}"), *line, OutlineKind::Label)); continue; }
        }
        if targets.contains(n) { out.push((n.to_string(), *line, OutlineKind::Label)); }
    }
    out
}

/// Kommentartext, wenn die Anweisung (nach der Zeilennummer) mit REM oder ' beginnt
fn basic_rem(body: &str) -> Option<&str> {
    if let Some(r) = body.strip_prefix('\'') { return Some(r); }
    if body.len() >= 3 && body[..3].eq_ignore_ascii_case("rem") { return Some(&body[3..]); }
    None
}

/// Zeilennummern, die die Anweisungen eines BASIC-Zeilenrests anspringen (Zeichenketten und REM-Reste werden übergangen)
fn basic_targets(body: &str) -> Vec<u32> {
    let mut out = vec![];
    let code = basic_code(body);
    let up = code.to_ascii_uppercase();
    let b = up.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let kw = ["GOTO", "GOSUB", "THEN", "ELSE", "GO TO", "RESTORE", "RUN", "RESUME"].iter().find(|k| b[i..].starts_with(k.as_bytes()));   // Bytevergleich: `up[i..]` stürzte ab, wenn i mitten in einem Umlaut lag
        let Some(k) = kw else { i += 1; continue };
        i += k.len();
        // Liste von Nummern (ON … GOTO a,b,c); bei THEN/ELSE nur eine
        loop {
            while i < b.len() && b[i] == b' ' { i += 1; }
            let d = b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
            if d == 0 { break; }
            if let Ok(n) = up[i..i + d].parse() { out.push(n); }
            i += d;
            while i < b.len() && b[i] == b' ' { i += 1; }
            if i < b.len() && b[i] == b',' && !matches!(*k, "THEN" | "ELSE" | "RESTORE" | "RUN" | "RESUME") { i += 1 } else { break }
        }
    }
    out
}

/// Zeilenrest ohne Zeichenketten (durch Leerzeichen ersetzt) und ohne Kommentar am Zeilenende (REM / ')
fn basic_code(body: &str) -> String {
    let mut s = String::new();
    let mut in_str = false;
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' { in_str = !in_str; s.push(' '); }
        else if in_str { s.push(' '); }
        else if c == '\'' { break; }
        else if c.eq_ignore_ascii_case(&'r') && i + 2 < chars.len() && chars[i + 1].eq_ignore_ascii_case(&'e') && chars[i + 2].eq_ignore_ascii_case(&'m')
            && s.chars().last().map_or(true, |p| p == ':' || p == ' ') { break; }
        else { s.push(c); }
        i += 1;
    }
    s
}

// ---- Verwendung eines Studio-Dokuments im Quelltext ---------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Needle {
    /// Teilzeichenfolge, z. B. der Dateiname
    Text(String),
    /// ganzes Wort (Label)
    Word(String),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Use { pub file: String, pub line: usize, pub text: String }

fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' }

fn has_word(line: &str, w: &str) -> bool {
    let mut from = 0;
    while let Some(p) = line[from..].find(w) {
        let (a, b) = (from + p, from + p + w.len());
        let before = line[..a].chars().next_back().map_or(true, |c| !is_word(c));
        let after = line[b..].chars().next().map_or(true, |c| !is_word(c));
        if before && after { return true; }
        from = b;
    }
    false
}

/// Fundstellen der Suchbegriffe in den Quelltexten (Datei, Text); höchstens `max`
/// Kommentarzeile? Assembler: beginnt mit `;`; BASIC: nach der Zeilennummer REM oder `'`
fn is_comment_line(file: &str, line: &str) -> bool {
    let t = line.trim_start();
    if matches!(file.rsplit_once('.').map(|(_, e)| e.to_lowercase()).as_deref(), Some("bas" | "sbas")) {
        let body = t.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start();
        basic_rem(body).is_some()
    } else { t.starts_with(';') }
}

pub(super) fn find_uses(sources: &[(String, String)], needles: &[Needle], max: usize) -> Vec<Use> {
    let mut out = vec![];
    for (file, text) in sources {
        for (i, l) in text.lines().enumerate() {
            if is_comment_line(file, l) { continue; }
            let hit = needles.iter().any(|n| match n { Needle::Text(s) => !s.is_empty() && l.contains(s.as_str()), Needle::Word(w) => !w.is_empty() && has_word(l, w) });
            if hit {
                out.push(Use { file: file.clone(), line: i + 1, text: l.trim().chars().take(90).collect() });
                if out.len() >= max { return out; }
            }
        }
    }
    out
}

/// Suchbegriffe für ein Studio-Dokument `rel` (Pfad im Projekt) mit dem Namen `name` aus der Datei
pub(super) fn needles_for(rel: &str, name: &str) -> Vec<Needle> {
    let file = rel.rsplit('/').next().unwrap_or(rel);
    let stem = file.rsplit_once('.').map_or(file, |(s, _)| s);
    let mut v = vec![Needle::Text(file.to_string()), Needle::Text(format!("export/{stem}."))];
    let label = name.replace('-', "_");
    if label.chars().count() >= 3 { v.push(Needle::Word(label)); }
    v
}

/// Alles, was das Panel „Studio“ über ein Dokument zeigt
#[derive(Clone, Default)]
pub(super) struct StudioInfo { pub size: Option<u64>, pub exports: Vec<(String, u64)>, pub uses: Vec<Use> }

fn is_source(rel: &str) -> bool { matches!(rel.rsplit_once('.').map(|(_, e)| e.to_lowercase()).as_deref(), Some("asm" | "inc" | "s" | "bas" | "sbas" | "lst" | "tur")) }

/// Aus dem Projekt zusammentragen. `name` ist der Name aus der Mediendatei. Liest Quelltexte (offene Dokumente zuerst) – darum nicht jedes Bild.
pub(super) fn collect_studio_info(project: &Project, docs: &[Doc], rel: &str, name: &str) -> StudioInfo {
    let size = std::fs::metadata(project.root.join(rel)).ok().map(|m| m.len());
    let stem = { let f = rel.rsplit('/').next().unwrap_or(rel); f.rsplit_once('.').map_or(f, |(s, _)| s).to_string() };
    let mut exports: Vec<(String, u64)> = std::fs::read_dir(project.root.join("build/export")).into_iter().flatten().flatten()
        .filter_map(|e| { let n = e.file_name().to_string_lossy().into_owned(); if n.starts_with(&format!("{stem}.")) { Some((format!("build/export/{n}"), e.metadata().ok()?.len())) } else { None } }).collect();
    exports.sort();
    let mut sources = vec![];
    for e in &project.entries {
        if e.rel.starts_with("build/") || !is_source(&e.rel) { continue; }
        let text = match docs.iter().find(|d| d.rel == e.rel) { Some(d) => d.text.clone(), None => match std::fs::read_to_string(project.root.join(&e.rel)) { Ok(t) if t.len() <= 1 << 20 => t, _ => continue } };
        sources.push((e.rel.clone(), text));
    }
    StudioInfo { size, exports, uses: find_uses(&sources, &needles_for(rel, name), 60) }
}

fn human(n: u64) -> String { if n >= 10_000 { format!("{:.1} KB", n as f64 / 1024.0) } else { format!("{n} Bytes") } }

// ---- Oberfläche -------------------------------------------------------------------------------------------------------------------

impl App {
    /// Kontext des Inspektors nachführen und merken, welcher Quelltext bzw. welches Studio aktiv ist
    pub(super) fn update_context(&mut self) {
        let (focus, doc, studio) = match self.dock.find_active_focused() {
            Some((_, Tab::Doc(r))) => (Focus::Doc, Some(r.clone()), None),
            Some((_, Tab::Studio(g))) => (Focus::Studio, None, Some(*g)),
            _ => (Focus::Other, None, None),
        };
        if doc.is_some() { self.st.ctx_doc = doc; }
        if studio.is_some() { self.st.ctx_studio = studio; }
        // geschlossene Dokumente vergessen
        if let Some(r) = &self.st.ctx_doc { if !self.st.docs.iter().any(|d| &d.rel == r) { self.st.ctx_doc = None; } }
        self.ctx = context_for(self.mode == Mode::Debug, focus, self.ctx);
        self.zones[0].studio_on = self.ctx == Ctx::Studio;
    }
}

fn hint(ui: &mut egui::Ui, text: &str) { ui.label(RichText::new(text).small().color(t().dim)); }

fn heading(ui: &mut egui::Ui, text: &str) { ui.add_space(4.0); ui.label(RichText::new(text).small().strong().color(t().mute)); }

impl State {
    /// Inspektor „Gliederung“: Labels, Konstanten und Makros des aktiven Quelltexts; Klick springt zur Zeile
    pub(super) fn outline_ui(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let Some(rel) = self.ctx_doc.clone() else { hint(ui, "Kein Quelltext aktiv. Öffne eine Datei, dann erscheint hier ihre Gliederung."); return };
        let Some(doc) = self.docs.iter().find(|d| d.rel == rel) else { return };
        let lang = doc.lang;
        if lang == crate::highlight::Lang::Plain { hint(ui, "Für diese Dateiart gibt es keine Gliederung (nur Assembler und BASIC)."); return; }
        let id = egui::Id::new(("outline", &rel));
        let key = (doc.version, doc.text.len());
        let cached: Option<std::sync::Arc<((i64, usize), Vec<OutlineRow>)>> = ui.data(|d| d.get_temp(id));
        let rows = match cached { Some(c) if c.0 == key => c, _ => {
            let rows: Vec<OutlineRow> = if lang == crate::highlight::Lang::Basic { basic_outline(&doc.text) } else { vc_asm::outline(&doc.text).into_iter().map(|i| (i.name, i.line, i.kind)).collect() };
            let c = std::sync::Arc::new((key, rows)); ui.data_mut(|d| d.insert_temp(id, c.clone())); c
        } };
        let rows = &rows.1;
        if rows.is_empty() { hint(ui, if lang == crate::highlight::Lang::Basic { "Keine Sprungziele (GOTO/GOSUB/THEN) und keine REM-Überschriften." } else { "Diese Datei hat keine Labels, Konstanten oder Makros." }); return; }
        let cur = current_label(rows, doc.cursor.0);
        let mut jump = None;
        egui::ScrollArea::vertical().id_salt("outline-scroll").auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (i, (name, line, kind)) in rows.iter().enumerate() {
                let (rect, r) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
                if Some(i) == cur { ui.painter().rect_filled(rect, 0.0, crate::ui::tokens::tint(k.cy).gamma_multiply(0.7)); ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height())), 0.0, k.cy); }
                else if r.hovered() { ui.painter().rect_filled(rect, 0.0, k.bg2); }
                let (mark, col) = match kind { OutlineKind::Label => ("▸", k.cy), OutlineKind::Const => ("=", k.am), OutlineKind::Macro => ("◆", k.gr) };
                ui.painter().text(egui::pos2(rect.left() + 10.0, rect.center().y), egui::Align2::CENTER_CENTER, mark, egui::FontId::monospace(11.0), col);
                ui.painter().with_clip_rect(rect).text(egui::pos2(rect.left() + 22.0, rect.center().y), egui::Align2::LEFT_CENTER, name, egui::FontId::monospace(12.0), if Some(i) == cur { k.cy } else { k.fg });
                ui.painter().text(egui::pos2(rect.right() - 8.0, rect.center().y), egui::Align2::RIGHT_CENTER, line.to_string(), egui::FontId::monospace(10.5), k.dim);
                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() { jump = Some(*line); }
            }
        });
        if let Some(l) = jump { self.open_request = Some(rel.clone()); self.pending_reveal = Some((rel, l - 1)); }
    }

    /// Inspektor „Studio“: Datei, Export und Verwendung im Quelltext des gerade aktiven Dokuments eines Studios
    pub(super) fn studio_ui(&mut self, ui: &mut egui::Ui, studios: &mut Vec<MediaWindow>) {
        let k = t();
        let Some(g) = self.ctx_studio else { hint(ui, "Kein Studio aktiv."); return };
        let Some(w) = studios.iter_mut().find(|w| w.group as u8 == g) else { hint(ui, "Kein Studio aktiv."); return };
        let focused = w.dock.find_active_focused().map(|(_, r)| r.clone());
        let rel = match focused.or_else(|| w.dock.iter_leaves().find_map(|(_, l)| l.tabs.get(l.active.0).cloned())) { Some(r) => r, None => { hint(ui, "In diesem Studio ist keine Datei geöffnet."); return } };
        let Some(md) = w.docs.iter().find(|d| d.rel == rel) else { return };
        let (kind, profile, name, note) = (md.kind.clone(), md.profile.clone(), md.name.clone(), md.note.clone());
        let Some(project) = self.project.as_ref() else { return };
        let now = ui.input(|i| i.time);
        // Fundstellen lesen Dateien: höchstens alle 1,5 s und bei Wechsel des Dokuments neu
        if !matches!(&self.studio_cache, Some((r, at, _)) if *r == rel && now - *at < 1.5) {
            self.studio_cache = Some((rel.clone(), now, collect_studio_info(project, &self.docs, &rel, &name)));
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(1500));
        let Some((_, _, info)) = &self.studio_cache else { return };
        let mut open: Option<(String, Option<usize>)> = None;
        egui::ScrollArea::vertical().id_salt("studio-info").auto_shrink([false, false]).show(ui, |ui| {
            ui.label(RichText::new(rel.rsplit('/').next().unwrap_or(&rel)).font(egui::FontId::new(14.0, crate::fonts::mono_strong())));
            ui.label(RichText::new(format!("{} · Profil {}", Group::of_kind(&kind).title(), if profile.is_empty() { "–" } else { &profile })).small().color(k.mute));
            if !note.is_empty() { ui.label(RichText::new(&note).small().color(k.am)); }
            heading(ui, "DATEI");
            ui.label(RichText::new(&rel).monospace().size(11.5));
            if let Some(s) = info.size { ui.label(RichText::new(human(s)).small().color(k.dim)); }
            heading(ui, "EXPORT");
            if info.exports.is_empty() { hint(ui, "Noch nicht exportiert. Im Studio: Exportieren (Menü Datei)."); }
            for (p, s) in &info.exports {
                ui.horizontal(|ui| { if ui.link(RichText::new(p).monospace().size(11.5)).on_hover_text("im Editor öffnen").clicked() { open = Some((p.clone(), None)); } ui.label(RichText::new(human(*s)).small().color(k.dim)); });
            }
            heading(ui, &format!("VERWENDUNG IM QUELLTEXT ({})", info.uses.len()));
            if info.uses.is_empty() { hint(ui, "Kein Quelltext des Projekts nennt diese Datei oder ihren Namen."); }
            for u in &info.uses {
                let (rect, r) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
                if r.hovered() { ui.painter().rect_filled(rect, 0.0, k.bg2); }
                ui.painter().text(egui::pos2(rect.left() + 4.0, rect.top() + 3.0), egui::Align2::LEFT_TOP, format!("{}:{}", u.file, u.line), egui::FontId::monospace(11.0), k.cy);
                ui.painter().with_clip_rect(rect).text(egui::pos2(rect.left() + 4.0, rect.top() + 18.0), egui::Align2::LEFT_TOP, &u.text, egui::FontId::monospace(11.0), k.mute);
                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() { open = Some((u.file.clone(), Some(u.line))); }
            }
        });
        if let Some((f, line)) = open { self.open_request = Some(f.clone()); if let Some(l) = line { self.pending_reveal = Some((f, l - 1)); } }
    }
}

// ---- Tests ------------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kontext_folgt_der_art_des_aktiven_dokuments() {
        assert_eq!(context_for(false, Focus::Doc, Ctx::Code), Ctx::Code);
        assert_eq!(context_for(false, Focus::Studio, Ctx::Code), Ctx::Studio);
        assert_eq!(context_for(false, Focus::Doc, Ctx::Studio), Ctx::Code);
        assert_eq!(context_for(true, Focus::Doc, Ctx::Code), Ctx::Debug);
        assert_eq!(context_for(true, Focus::Studio, Ctx::Studio), Ctx::Debug);
    }

    #[test]
    fn kontext_springt_nicht_ohne_grund() {
        // Willkommen/Übersicht/Einstellungen aktiv: der bisherige Kontext bleibt
        assert_eq!(context_for(false, Focus::Other, Ctx::Studio), Ctx::Studio);
        assert_eq!(context_for(false, Focus::Other, Ctx::Code), Ctx::Code);
        // aus dem Debug-Arbeitsplatz zurück: Quelltext
        assert_eq!(context_for(false, Focus::Other, Ctx::Debug), Ctx::Code);
    }

    #[test]
    fn jeder_kontext_hat_seine_reiter() {
        assert_eq!(inspector_tabs(Ctx::Code), vec![Tab::Outline, Tab::MemMap, Tab::Ref, Tab::Help]);
        assert_eq!(inspector_tabs(Ctx::Studio), vec![Tab::StudioInfo, Tab::MemMap, Tab::Help]);
        assert_eq!(inspector_tabs(Ctx::Debug), vec![Tab::Regs, Tab::Watch, Tab::Breaks, Tab::Help]);
        for c in [Ctx::Code, Ctx::Studio, Ctx::Debug] { assert!(inspector_tabs(c).iter().all(|t| t.zone() == Some(Zone::Inspector))); }
        for d in [false, true] { assert!(footer_tabs(d).iter().all(|t| t.zone() == Some(Zone::Footer))); }
        assert_eq!(footer_tabs(false), vec![Tab::Output, Tab::BuildLog, Tab::Problems, Tab::Tests]);
        assert_eq!(footer_tabs(true), vec![Tab::Disasm, Tab::Memory, Tab::Emulator, Tab::Snapshot, Tab::Live, Tab::Output, Tab::Problems]);
        let nur_vice = vc_ide::features::Features::from_json(r#"{"format":1,"links":["vice"]}"#);
        assert_eq!(footer_tabs_with(true, &nur_vice), vec![Tab::Disasm, Tab::Memory, Tab::Emulator, Tab::Snapshot, Tab::Output, Tab::Problems], "ohne Ultimate kein Live-Bild");
        assert_eq!(footer_tabs_with(false, &nur_vice), footer_tabs(false));
    }

    /// Selbsttest mit Mindestauswahl (Konzept Release-Auswahl, Abschnitt 9; Risiko 1): drei Auswahlen wie die Testdateien unter `tools/testdata`.
    /// Alle Verbraucher, die ohne Fenster prüfbar sind, zeigen nichts Abgewähltes: Menüs und Befehlspalette, Reiter, Hilfe, Anbindungen, Ziele.
    #[test]
    fn mindestauswahl_zeigt_nichts_abgewaehltes() {
        use crate::nativemenu::{visible_with, Cmd, COMMANDS};
        use vc_ide::features::Features;
        const ALLE_STUDIOS: &str = r#""studios":["sprite","char","screen","tiles","palette","tracker","play","disk","memmap"]"#;
        let ide_c64 = Features::from_json(&format!(r#"{{"format":1,"ide":true,{ALLE_STUDIOS},"links":[],"targets":["c64"],"examples":[]}}"#));
        let ide_atari = Features::from_json(r#"{"format":1,"ide":true,"studios":[],"links":["atari800"],"targets":["atari8"],"examples":["EX-HELLO"]}"#);
        let ohne_ide = Features::from_json(r#"{"format":1,"ide":false,"studios":[],"links":["atari800"],"targets":["atari8"],"examples":["EX-HELLO"]}"#);
        let sichtbar = |f: &Features| -> Vec<Cmd> { COMMANDS.iter().map(|i| i.cmd).filter(|c| visible_with(*c, f)).collect() };
        let in_menus = |f: &Features, c: Cmd| crate::nativemenu::menu_has(f, c);

        // IDE + C64, kein Emulator, keine Hardware: kein Starten/Debuggen/Live-Bild/Emulator-Einrichtung, aber alle Studios
        let v = sichtbar(&ide_c64);
        for c in [Cmd::Debug, Cmd::DebugActive, Cmd::DbgCont, Cmd::DbgStepInto, Cmd::DbgEmulator, Cmd::WorkDebug, Cmd::LiveImage, Cmd::SetupEmulators, Cmd::SetupHardware] {
            assert!(!v.contains(&c) && !in_menus(&ide_c64, c), "{c:?} ohne Anbindung sichtbar");
        }
        assert!(v.contains(&Cmd::NewMedia) && v.contains(&Cmd::Export), "Studio-Befehle fehlen");
        assert!(!ide_c64.any_debugger() && !ide_c64.any_live_image() && !ide_c64.target("atari8") && !ide_c64.any_examples());
        assert!(!footer_tabs_with(true, &ide_c64).contains(&Tab::Live));
        assert!(inspector_tabs_with(Ctx::Code, &ide_c64).contains(&Tab::MemMap));
        assert!(!vc_ide::help::visible_with("emulators", &ide_c64) && !vc_ide::help::visible_with("mega65", &ide_c64) && vc_ide::help::visible_with("sprite", &ide_c64));

        // IDE + Atari, keine Studios, nur atari800: Debuggen ja, Live-Bild nein, keine Studio-Befehle, keine Memory Map
        let v = sichtbar(&ide_atari);
        assert!(v.contains(&Cmd::Debug) && v.contains(&Cmd::DbgCont) && v.contains(&Cmd::SetupEmulators));
        for c in [Cmd::LiveImage, Cmd::SetupHardware, Cmd::NewMedia, Cmd::Export, Cmd::StdCharset, Cmd::ImportRom, Cmd::StudioDetach] { assert!(!v.contains(&c) && !in_menus(&ide_atari, c), "{c:?} sichtbar"); }
        assert!(ide_atari.any_debugger() && !ide_atari.any_live_image());
        assert!(!inspector_tabs_with(Ctx::Code, &ide_atari).contains(&Tab::MemMap) && !inspector_tabs_with(Ctx::Studio, &ide_atari).contains(&Tab::MemMap));
        assert!(vc_ide::help::visible_with("emulators", &ide_atari) && !vc_ide::help::visible_with("sprite", &ide_atari) && !vc_ide::help::visible_with("tracker", &ide_atari) && !vc_ide::help::visible_with("vic20", &ide_atari));
        assert_eq!(vc_ide::emulators::available_in(&ide_atari).iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["atari800"]);

        // ohne IDE: keine Studios und keine Studio-Befehle, auch wenn Studios „gewählt“ stehen
        let ohne_ide_mit_studios = Features::from_json(&format!(r#"{{"format":1,"ide":false,{ALLE_STUDIOS}}}"#));
        assert!(!ohne_ide_mit_studios.studio("sprite") && !sichtbar(&ohne_ide_mit_studios).contains(&Cmd::NewMedia) && !sichtbar(&ohne_ide).contains(&Cmd::NewMedia));
        assert!(!vc_ide::help::visible_with("sprite", &ohne_ide_mit_studios));
    }

    #[test]
    fn reiter_wird_je_kontext_gemerkt() {
        let mut z = Zones::defaults(Mode::Code);
        z.insp().show(&Tab::Ref);                       // im Quelltext: Referenz
        z.studio_on = true;
        assert_eq!(z.insp().active_tab(), Some(&Tab::StudioInfo));
        z.insp().show(&Tab::MemMap);                    // im Studio: Memory Map
        z.studio_on = false;                            // zurück zum Quelltext: wieder die Referenz
        assert_eq!(z.insp().active_tab(), Some(&Tab::Ref));
        z.studio_on = true;
        assert_eq!(z.insp().active_tab(), Some(&Tab::MemMap));
        // Debug hat eigene Reiter
        assert_eq!(Zones::defaults(Mode::Debug).insp().active_tab(), Some(&Tab::Regs));
    }

    #[test]
    fn marke_der_probleme() {
        assert_eq!(problems_badge(0, 0), None);
        assert_eq!(problems_badge(3, 5), Some(Badge { text: "3".into(), kind: BadgeKind::Err }));
        assert_eq!(problems_badge(0, 2), Some(Badge { text: "2".into(), kind: BadgeKind::Warn }));
    }

    #[test]
    fn marke_der_tests() {
        assert_eq!(tests_badge(0, 0, 0), None);
        assert_eq!(tests_badge(16, 16, 0), Some(Badge { text: "16/16".into(), kind: BadgeKind::Ok }));
        assert_eq!(tests_badge(13, 16, 3), Some(Badge { text: "13/16".into(), kind: BadgeKind::Err }));
    }

    #[test]
    fn fusspanel_beim_bauen() {
        assert_eq!(build_footer_action(2, true), FooterAct::ShowProblems);
        assert_eq!(build_footer_action(2, false), FooterAct::ShowProblems);   // Fehler öffnen es auch, wenn es eingeklappt war
        assert_eq!(build_footer_action(0, true), FooterAct::SelectBuildLog);
        assert_eq!(build_footer_action(0, false), FooterAct::Nothing);          // bewusst eingeklappt, kein Fehler: bleibt zu
    }

    #[test]
    fn aktuelles_label() {
        let rows: Vec<OutlineRow> = vec![("SCREEN".into(), 1, OutlineKind::Const), ("start".into(), 3, OutlineKind::Label), ("m".into(), 8, OutlineKind::Macro), ("loop".into(), 10, OutlineKind::Label)];
        assert_eq!(current_label(&rows, 1), None);
        assert_eq!(current_label(&rows, 3), Some(1));
        assert_eq!(current_label(&rows, 9), Some(1));
        assert_eq!(current_label(&rows, 40), Some(3));
        assert_eq!(current_label(&[], 5), None);
    }

    #[test]
    fn fundstellen_im_quelltext() {
        let src = vec![("main.asm".to_string(), "  lda #0\n.include \"build/export/hero.asm\"\n  ldx #hero_w\n  jsr hero_frames_draw\n; hero.vcspr\n".to_string())];
        let n = needles_for("graphics/hero.vcspr", "hero-frames");
        let u = find_uses(&src, &n, 10);
        assert_eq!(u.iter().map(|u| u.line).collect::<Vec<_>>(), vec![2]);   // nur der Include; die Kommentarzeile `; hero.vcspr` zählt nicht
        let u = find_uses(&src, &needles_for("x/hero.vcspr", "hero_w"), 10);
        assert!(u.iter().any(|u| u.line == 3));
        assert_eq!(find_uses(&src, &n, 1).len(), 1);
    }

    #[test]
    fn basic_gliederung() {
        let src = "10 REM --- Titel ---\n20 PRINT \"GOTO 99\"\n30 IF A=1 THEN 100\n40 ON A GOTO 50,60:GOSUB 200 : rem 300\n50 X=1\n60 X=2\n100 REM\n110 'Schleife\n200 RETURN\n300 END\nREM ohne Nummer\n";
        let rows = basic_outline(src);
        let got: Vec<(String, usize)> = rows.iter().map(|r| (r.0.clone(), r.1)).collect();
        assert_eq!(got, vec![("10 Titel".to_string(), 1), ("50".into(), 5), ("60".into(), 6), ("100".into(), 7), ("110 Schleife".into(), 8), ("200".into(), 9)]);
    }

    #[test]
    fn verwendung_ueberspringt_kommentare() {
        let n = vec![Needle::Word("hero".into())];
        let asm = vec![("a.asm".to_string(), "; hero ist hier\n  jsr hero\n   ; hero\n".to_string())];
        assert_eq!(find_uses(&asm, &n, 10).iter().map(|u| u.line).collect::<Vec<_>>(), vec![2]);
        let bas = vec![("a.bas".to_string(), "10 REM hero\n20 PRINT \"hero\"\n30 \'hero\n".to_string())];
        assert_eq!(find_uses(&bas, &n, 10).iter().map(|u| u.line).collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn ganzes_wort_statt_teilwort() {
        assert!(has_word("lda hero,x", "hero"));
        assert!(!has_word("lda heroine", "hero"));
        assert!(!has_word("lda my_hero", "hero"));
        assert!(has_word("heroheroine hero", "hero"));
    }
}
