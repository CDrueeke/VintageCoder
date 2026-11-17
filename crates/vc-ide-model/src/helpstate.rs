// SPDX-License-Identifier: GPL-3.0-or-later
//! Hilfe ohne Oberfläche: Themenliste, Suche und Nachschlagen eines Themas oder Wortes (Befehl, Direktive, Register, BASIC-Wort).
//! Die Inhalte kommen aus `vc_ide::help` (Handbuchkapitel und `data/help`).
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Topic { pub id: String, pub group: String, pub title: String }
#[derive(Clone, Debug, PartialEq)]
pub struct Result { pub id: String, pub title: String, pub snippet: String }
#[derive(Clone, Debug, PartialEq)]
pub struct Doc { pub title: String, pub markdown: String, /** Kennung des Themas (leer bei Nachschlagen eines Worts) */ pub id: String }

/// Alle sichtbaren Themen (nach Gruppen geordnet, wie sie `vc_ide::help` liefert)
pub fn topics() -> Vec<Topic> {
    vc_ide::help::topic_list()["topics"].as_array().map(|a| a.iter().map(|t| Topic { id: t["id"].as_str().unwrap_or("").into(), group: t["group"].as_str().unwrap_or("").into(), title: t["title"].as_str().unwrap_or("").into() }).collect()).unwrap_or_default()
}

/// Suche über Themen, Befehle, Direktiven, Register und BASIC-Wörter im Kontext des Ziels
pub fn search(data: &Path, target: &str, q: &str) -> Vec<Result> {
    if q.trim().is_empty() { return vec![] }
    vc_ide::help::search(data, target, q).ok().and_then(|v| v["results"].as_array().map(|a| a.iter().map(|x| Result { id: x["id"].as_str().unwrap_or("").into(), title: x["title"].as_str().unwrap_or("").into(), snippet: x["snippet"].as_str().unwrap_or("").into() }).collect())).unwrap_or_default()
}

/// Thema (`id`) oder Wort nachschlagen; `lang` ist `asm` oder `basic`. `None`: nichts gefunden
pub fn open(data: &Path, target: &str, lang: &str, key: &str) -> Option<Doc> {
    if let Some(t) = vc_ide::help::topic(key) { return Some(Doc { title: t["title"].as_str()?.into(), markdown: t["markdown"].as_str()?.into(), id: key.into() }); }
    let v = vc_ide::help::lookup(data, target, key, lang).ok()?;
    if v["kind"] == "none" && v["markdown"].as_str().map_or(true, |m| m.is_empty()) { return None; }
    Some(Doc { title: v["title"].as_str().unwrap_or(key).into(), markdown: v["markdown"].as_str()?.into(), id: String::new() })
}

/// Ziel für die Hilfe: erstes System des Projekts, sonst `c64`
pub fn target_of(systems: &[String]) -> String { systems.first().cloned().unwrap_or_else(|| "c64".into()) }

/// Markdown in Anzeigezeilen zerlegen: (Art, Text) – Überschriften, Listenpunkte, Codezeilen, Absätze; Auszeichnungen (`**`, `` ` ``) entfallen
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line { H1, H2, Bullet, Code, Para, Blank }
pub fn lines(md: &str) -> Vec<(Line, String)> {
    let (mut out, mut code) = (vec![], false);
    let strip = |s: &str| s.replace("**", "").replace('`', "");
    for l in md.lines() {
        if l.trim_start().starts_with("```") { code = !code; continue }
        if code { out.push((Line::Code, l.to_string())); continue }
        let t = l.trim_end();
        if t.trim().is_empty() { if out.last().map_or(false, |x| x.0 != Line::Blank) { out.push((Line::Blank, String::new())) } }
        else if let Some(h) = t.strip_prefix("# ") { out.push((Line::H1, strip(h))) }
        else if let Some(h) = t.strip_prefix("## ").or_else(|| t.strip_prefix("### ")) { out.push((Line::H2, strip(h))) }
        else if let Some(b) = t.trim_start().strip_prefix("- ").or_else(|| t.trim_start().strip_prefix("* ")) { out.push((Line::Bullet, strip(b))) }
        else { out.push((Line::Para, strip(t))) }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap() }
    #[test] fn themen_und_suche_und_nachschlagen() {
        let t = topics(); assert!(!t.is_empty() && t.iter().all(|x| !x.id.is_empty() && !x.title.is_empty()));
        let r = search(&data(), "c64", "lda"); assert!(!r.is_empty(), "lda wird gefunden");
        assert!(search(&data(), "c64", "  ").is_empty());
        let d = open(&data(), "c64", "asm", "lda").expect("lda nachschlagbar"); assert!(!d.markdown.is_empty());
        let erstes = &t[0].id; assert!(open(&data(), "c64", "asm", erstes).is_some(), "Thema {erstes}");
    }
    #[test] fn markdown_wird_zu_zeilen() {
        let l = lines("# Titel\n\nText mit **fett** und `code`.\n\n- Punkt\n```\nlda #1\n```\n## Unter");
        assert_eq!(l[0], (Line::H1, "Titel".into())); assert_eq!(l[2], (Line::Para, "Text mit fett und code.".into()));
        assert!(l.contains(&(Line::Bullet, "Punkt".into())) && l.contains(&(Line::Code, "lda #1".into())) && l.last() == Some(&(Line::H2, "Unter".into())));
    }
}
