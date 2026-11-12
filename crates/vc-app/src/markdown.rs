// SPDX-License-Identifier: GPL-3.0-or-later
//! Kleiner Markdown-Renderer für die Hilfe: Überschriften, Listen, Codeblöcke, Tabellen (als Text), Fett, Kursiv, Code, Links
//! (`help:thema` / `hilfe:thema`, `kapitel:id` für Handbuchkapitel, `https://…` im Browser), Hinweisblöcke (`> **Tipp:**`, `**Achtung:**`,
//! `**Info:**`) und Bilder `![Alt](bild:id "Unterschrift")` (über [`Images`]; ohne Bildquelle weggelassen).
use crate::ui::tokens::t as tok;
use eframe::egui::{self, RichText};

fn heading(text: &str, size: f32) -> RichText { RichText::new(text).font(egui::FontId::new(size, crate::fonts::ui_strong())).color(tok().fg) }

/// Bildquelle für den Renderer (Handbuchbilder, siehe `help_images`): zeichnet das Bild, einen Platzhalter oder nichts
pub trait Images { fn image(&mut self, ui: &mut egui::Ui, id: &str, alt: &str, caption: &str); }

enum Span { Text(String), Bold(String), Italic(String), Code(String), Link(String, String) }

/// Bildzeile `![Alt](bild:id "Unterschrift")` → (Alt, Kennung, Unterschrift)
pub fn parse_image(line: &str) -> Option<(String, String, String)> {
    let rest = line.trim().strip_prefix("![")?;
    let (alt, rest) = rest.split_once("](bild:")?;
    let rest = rest.strip_suffix(')')?;
    let (id, cap) = match rest.split_once(' ') { Some((id, c)) => (id, c.trim().trim_matches('"')), None => (rest, "") };
    if id.is_empty() { return None; }
    Some((alt.to_string(), id.to_string(), cap.to_string()))
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hint { Tip, Warning, Info }

/// `> **Tipp:** Text` (auch Achtung/Info) → Art und Zeile ohne `> `
pub fn parse_hint(line: &str) -> Option<(Hint, &str)> {
    let q = line.trim_start().strip_prefix("> ")?;
    for (name, kind) in [("**Tipp:**", Hint::Tip), ("**Achtung:**", Hint::Warning), ("**Info:**", Hint::Info)] { if q.starts_with(name) { return Some((kind, q)); } }
    None
}

/// Ziel eines Links als Hilfeschlüssel: `help:x`/`hilfe:x` → `x`, `kapitel:x` → `kapitel:x`; Webadressen und anderes: `None`
pub fn link_key(url: &str) -> Option<String> {
    if let Some(h) = url.strip_prefix("help:").or_else(|| url.strip_prefix("hilfe:")) { return Some(h.to_string()); }
    if url.starts_with("kapitel:") { return Some(url.to_string()); }
    None
}

fn spans(line: &str) -> Vec<Span> {
    let c: Vec<char> = line.chars().collect(); let mut out = vec![]; let mut buf = String::new(); let mut i = 0;
    let flush = |buf: &mut String, out: &mut Vec<Span>| { if !buf.is_empty() { out.push(Span::Text(std::mem::take(buf))); } };
    while i < c.len() {
        if c[i] == '`' { if let Some(j) = c[i + 1..].iter().position(|x| *x == '`') { flush(&mut buf, &mut out); out.push(Span::Code(c[i + 1..i + 1 + j].iter().collect())); i += j + 2; continue; } }
        if c[i] == '*' && c.get(i + 1) == Some(&'*') { if let Some(j) = (i + 2..c.len().saturating_sub(1)).find(|k| c[*k] == '*' && c[k + 1] == '*') { flush(&mut buf, &mut out); out.push(Span::Bold(c[i + 2..j].iter().collect())); i = j + 2; continue; } }
        if c[i] == '*' && c.get(i + 1).map_or(false, |n| !n.is_whitespace() && *n != '*') { if let Some(j) = (i + 2..c.len()).find(|k| c[*k] == '*' && !c[k - 1].is_whitespace() && c.get(k + 1) != Some(&'*')) { flush(&mut buf, &mut out); out.push(Span::Italic(c[i + 1..j].iter().collect())); i = j + 1; continue; } }
        if c[i] == '[' { if let Some(j) = c[i..].iter().position(|x| *x == ']') { if c.get(i + j + 1) == Some(&'(') { if let Some(k) = c[i + j + 2..].iter().position(|x| *x == ')') {
            flush(&mut buf, &mut out); out.push(Span::Link(c[i + 1..i + j].iter().collect(), c[i + j + 2..i + j + 2 + k].iter().collect())); i += j + k + 3; continue; } } } }
        buf.push(c[i]); i += 1;
    }
    flush(&mut buf, &mut out); out
}

/// Zeichnet einen Absatz aus Spans; Klick auf einen Link setzt `clicked` (Hilfeschlüssel) bzw. öffnet die Webadresse
fn paragraph(ui: &mut egui::Ui, sp: Vec<Span>, indent: f32, clicked: &mut Option<String>) {
    let mono = egui::TextStyle::Monospace;
    if !sp.iter().any(|s| matches!(s, Span::Link(..))) {
        // Absatz ohne Links: ein einziger Text mit Mischformat, damit Umbrüche sauber eingerückt bleiben
        let k = tok();
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = (ui.available_width() - indent).max(40.0);
        let body_font = egui::TextStyle::Body.resolve(ui.style());
        let code_font = egui::TextStyle::Monospace.resolve(ui.style());
        for s in &sp {
            match s {
                Span::Text(x) => job.append(x, 0.0, egui::TextFormat { font_id: body_font.clone(), color: ui.visuals().text_color(), ..Default::default() }),
                Span::Bold(x) => job.append(x, 0.0, egui::TextFormat { font_id: egui::FontId::new(body_font.size, crate::fonts::ui_strong()), color: k.fg, ..Default::default() }),
                Span::Italic(x) => job.append(x, 0.0, egui::TextFormat { font_id: body_font.clone(), color: ui.visuals().text_color(), italics: true, ..Default::default() }),
                Span::Code(x) => job.append(x, 0.0, egui::TextFormat { font_id: code_font.clone(), color: k.cy, background: crate::theme::code_bg(1), ..Default::default() }),
                Span::Link(..) => {}
            }
        }
        ui.horizontal(|ui| { if indent > 0.0 { ui.add_space(indent); } ui.add(egui::Label::new(job).wrap()); });
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if indent > 0.0 { ui.add_space(indent); }
        for s in sp {
            match s {
                Span::Text(x) => { ui.label(x); }
                Span::Bold(x) => { ui.label(RichText::new(x).font(egui::FontId::new(egui::TextStyle::Body.resolve(ui.style()).size, crate::fonts::ui_strong())).color(tok().fg)); }
                Span::Italic(x) => { ui.label(RichText::new(x).italics()); }
                Span::Code(x) => { ui.label(RichText::new(x).text_style(mono.clone()).color(tok().cy).background_color(crate::theme::code_bg(1))); }
                Span::Link(text, url) => { if link_key(&url).map_or(false, |k| !vc_ide::help::visible(&k)) { ui.label(RichText::new(text)); } else if ui.link(RichText::new(text).color(tok().cy)).clicked() { if let Some(h) = link_key(&url) { *clicked = Some(h); } else if url.starts_with("https://") { ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone())); } } }
            }
        }
    });
}

/// Hinweiskasten: Tipp (grün), Achtung (bernstein), Info (blau/cyan)
fn hint_box(ui: &mut egui::Ui, kind: Hint, line: &str, clicked: &mut Option<String>) {
    let k = tok();
    let col = match kind { Hint::Tip => k.gr, Hint::Warning => k.am, Hint::Info => k.cy };
    ui.add_space(2.0);
    egui::Frame::new().fill(col.gamma_multiply(0.13)).stroke(egui::Stroke::new(1.0, col.gamma_multiply(0.55))).corner_radius(6.0).inner_margin(egui::Margin::symmetric(10, 7)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        paragraph(ui, spans(line), 0.0, clicked);
    });
    ui.add_space(2.0);
}

/// Zeichnet `md` (ohne Bilder); gibt das angeklickte Hilfethema (`help:xyz` → `xyz`, `kapitel:id`) zurück
pub fn render(ui: &mut egui::Ui, md: &str) -> Option<String> { render_with(ui, md, None) }

/// Wie [`render`], mit Bildquelle für `![…](bild:id "…")`
pub fn render_with(ui: &mut egui::Ui, md: &str, mut imgs: Option<&mut dyn Images>) -> Option<String> {
    let mut clicked = None; let mut in_code = false; let mono = egui::TextStyle::Monospace;
    let mut code_buf: Vec<&str> = vec![];
    let mut table: Vec<&str> = vec![]; let mut tcount = 0;
    let flush_table = |ui: &mut egui::Ui, table: &mut Vec<&str>, n: &mut usize| {
        if table.is_empty() { return; }
        *n += 1;
        let k = tok();
        ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, k.line);
        egui::ScrollArea::horizontal().id_salt(("md-scroll", *n)).auto_shrink([true, true]).show(ui, |ui| {
        egui::Frame::new().stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(6)).show(ui, |ui| {
        egui::Grid::new(("md-table", *n, table.len())).striped(true).spacing([14.0, 3.0]).show(ui, |ui| {
            for (i, row) in table.iter().enumerate() {
                let cells: Vec<&str> = row.trim().trim_matches('|').split('|').map(|c| c.trim()).collect();
                if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':')) { continue; }
                for c in cells { let tx = c.replace('`', "").replace("**", ""); ui.label(if i == 0 { heading(&tx, 13.0) } else { RichText::new(tx) }); }
                ui.end_row();
            }
        });
        });
        });
        table.clear();
    };
    for line in md.lines() {
        if !in_code && line.trim_start().starts_with('|') { table.push(line); continue; } else { flush_table(ui, &mut table, &mut tcount); }
        if line.trim_start().starts_with("```") {
            if in_code { egui::Frame::new().fill(crate::theme::code_bg(0)).stroke(egui::Stroke::new(1.0, tok().line)).corner_radius(4.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| { ui.set_width(ui.available_width()); ui.label(RichText::new(code_buf.join("\n")).text_style(mono.clone())); }); code_buf.clear(); }
            in_code = !in_code; continue;
        }
        if in_code { code_buf.push(line); continue; }
        let t = line.trim_end();
        if t.is_empty() { ui.add_space(4.0); continue; }
        if let Some((alt, id, cap)) = parse_image(t) { if let Some(im) = imgs.as_deref_mut() { im.image(ui, &id, &alt, &cap); } continue; }
        if let Some((kind, q)) = parse_hint(t) { hint_box(ui, kind, q, &mut clicked); continue; }
        if let Some(h) = t.strip_prefix("### ") { ui.add_space(4.0); ui.label(heading(h, 15.0)); continue; }
        if let Some(h) = t.strip_prefix("## ") { ui.add_space(6.0); ui.label(heading(h, 17.0)); continue; }
        if let Some(h) = t.strip_prefix("# ") { ui.add_space(2.0); ui.label(heading(h, 21.0)); ui.separator(); continue; }
        // Einrückung der Quelle (verschachtelte Listen, Fortsetzungszeilen) wird zu 6 Punkten je Leerzeichen
        let tt = t.trim_start(); let lead = (t.len() - tt.len()) as f32 * 6.0;
        let (indent, body) = if let Some(b) = tt.strip_prefix("- ").or_else(|| tt.strip_prefix("* ")) { (10.0 + lead, format!("•  {b}")) } else if tt.chars().next().map_or(false, |c| c.is_ascii_digit()) && tt.find(". ").map_or(false, |p| p < 3) { (10.0 + lead, tt.to_string()) } else if let Some(q) = tt.strip_prefix("> ") { (10.0 + lead, q.to_string()) } else { (lead, tt.to_string()) };
        paragraph(ui, spans(&body), indent, &mut clicked);
    }
    flush_table(ui, &mut table, &mut tcount);
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bildzeilen_werden_zerlegt() {
        assert_eq!(parse_image("![Ein Bild (mit Klammer)](bild:ui-gesamt \"Die Unterschrift (ganz)\")"), Some(("Ein Bild (mit Klammer)".into(), "ui-gesamt".into(), "Die Unterschrift (ganz)".into())));
        assert_eq!(parse_image("![Alt](bild:x)"), Some(("Alt".into(), "x".into(), "".into())));
        assert_eq!(parse_image("![Alt](https://x/y.png)"), None);
        assert_eq!(parse_image("Text ![Alt](bild:x)"), None);
        assert_eq!(parse_image("![Alt](bild:)"), None);
    }

    #[test]
    fn hinweisbloecke_werden_erkannt() {
        assert_eq!(parse_hint("> **Tipp:** Text").map(|h| h.0), Some(Hint::Tip));
        assert_eq!(parse_hint("> **Achtung:** Text").map(|h| h.0), Some(Hint::Warning));
        assert_eq!(parse_hint("> **Info:** Text").map(|h| h.0), Some(Hint::Info));
        assert_eq!(parse_hint("> Nur ein Zitat"), None);
        assert_eq!(parse_hint("**Tipp:** ohne Zitatzeichen"), None);
    }

    #[test]
    fn links_werden_zu_hilfeschluesseln() {
        assert_eq!(link_key("help:editor").as_deref(), Some("editor"));
        assert_eq!(link_key("hilfe:editor").as_deref(), Some("editor"));
        assert_eq!(link_key("kapitel:willkommen").as_deref(), Some("kapitel:willkommen"));
        assert_eq!(link_key("https://example.org"), None);
        // jede Kennung, die das Handbuch verlinkt, löst die Hilfe auf
        for t in vc_ide::help::handbook() {
            for part in t.body.split("](").skip(1) {
                let url: String = part.chars().take_while(|c| *c != ')' && *c != ' ').collect();
                if let Some(k) = link_key(&url) { assert!(vc_ide::help::topic(&k).is_some(), "{}: Link {url}", t.id); }
            }
        }
    }

    #[test]
    fn kursiv_fett_und_listenzeichen() {
        let sp = spans("ein *kursives* und **fettes** Wort, 2*3*4");
        assert!(sp.iter().any(|s| matches!(s, Span::Italic(x) if x == "kursives")));
        assert!(sp.iter().any(|s| matches!(s, Span::Bold(x) if x == "fettes")));
        assert!(!spans("a * b").iter().any(|s| matches!(s, Span::Italic(_))));
    }
}
