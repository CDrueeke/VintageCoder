// SPDX-License-Identifier: GPL-3.0-or-later
//! Zentrale Namenstabelle der VCASM-Direktiven (`data/asm/directives.json`).
//!
//! Die Tabelle ist die einzige Quelle für kanonische Namen, Alias, Blockenden und Gruppen. Der Assembler löst jeden
//! Namen schon beim Lesen der Zeile auf den kanonischen Namen auf und kennt danach nur diese; Sprachserver, Faltung
//! und Editor-Erweiterung lesen dieselbe Tabelle. Direktivennamen werden nie übersetzt.
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Ein Eintrag der Namenstabelle
#[derive(Debug, Clone, Deserialize)]
pub struct Directive {
    /// kanonischer Name (mit Punkt, kleingeschrieben)
    pub name: String,
    /// gleichwertige alte oder kurze Schreibweisen (auch Formen ohne Punkt wie `*=` oder `#include`)
    #[serde(default)]
    pub aliases: Vec<String>,
    /// kanonisches Blockende, falls die Direktive einen Block öffnet
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub end_aliases: Vec<String>,
    pub group: String,
    /// Argumentmuster für Tooltips
    pub args: String,
    /// Version, ab der der kanonische Name gilt
    pub since: String,
    /// Schlüssel der Kurzbeschreibung im Katalog (`lsp-directive-<name>`)
    pub summary_id: String,
}

const JSON: &str = include_str!("../../../data/asm/directives.json");

/// Alle Einträge in der Reihenfolge der Tabelle
pub fn table() -> &'static [Directive] {
    static T: [OnceLock<Vec<Directive>>; 2] = [OnceLock::new(), OnceLock::new()];
    T[usize::from(vc_i18n::language() == vc_i18n::Lang::En)].get_or_init(|| {
        let mut v: serde_json::Value = serde_json::from_str(JSON).expect("data/asm/directives.json is valid JSON");
        vc_i18n::localize_json(&mut v, "asm/directives.json");
        serde_json::from_value(v).expect("data/asm/directives.json is valid JSON")
    })
}

fn map() -> &'static HashMap<String, &'static str> {
    static M: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        for d in table() {
            let mut put = |from: &str, to: &'static str| { if from.starts_with('.') { m.insert(from.to_string(), to); } };
            put(&d.name, &d.name);
            for a in &d.aliases { put(a, &d.name); }
            if let Some(e) = &d.end {
                put(e, e);
                for a in &d.end_aliases { put(a, e); }
            }
        }
        m
    })
}

/// Kanonischer Name zu `word` (mit Punkt, kleingeschrieben): Name, Alias, Blockende oder Alias des Blockendes. `None` bei Unbekanntem.
pub fn canonical(word: &str) -> Option<&'static str> { map().get(word).copied() }

/// Ist `word` ein Alias (also nicht schon der kanonische Name)?
pub fn is_alias(word: &str) -> bool { canonical(word).map_or(false, |c| c != word) }

/// Alle kanonischen Namen (Direktiven und Blockenden), je einmal, in Tabellenreihenfolge
pub fn canonical_names() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = vec![];
    for d in table() {
        for n in std::iter::once(d.name.as_str()).chain(d.end.as_deref()) { if !v.contains(&n) { v.push(n); } }
    }
    v
}

/// Alle Alias (nur Formen mit Punkt) samt kanonischem Namen
pub fn aliases() -> Vec<(&'static str, &'static str)> {
    let mut v = vec![];
    for d in table() {
        for a in d.aliases.iter().filter(|a| a.starts_with('.')) { v.push((a.as_str(), d.name.as_str())); }
        if let Some(e) = &d.end { for a in d.end_aliases.iter().filter(|a| a.starts_with('.')) { v.push((a.as_str(), e.as_str())); } }
    }
    v
}

/// Der Eintrag, zu dem `word` (Name, Alias oder Blockende) gehört; bei gemeinsamem Ende (`.endif`) der erste
pub fn entry(word: &str) -> Option<&'static Directive> {
    let c = canonical(word)?;
    table().iter().find(|d| d.name == c).or_else(|| table().iter().find(|d| d.end.as_deref() == Some(c)))
}

/// Namen, die einen Block öffnen (kanonisch und Alias), und Namen, die ihn schließen (kanonisch und Alias) – für die Faltung
pub fn block_words() -> (Vec<&'static str>, Vec<&'static str>) {
    let (mut opens, mut closes) = (vec![], vec![]);
    for d in table().iter().filter(|d| d.end.is_some()) {
        for n in std::iter::once(&d.name).chain(d.aliases.iter()) { if n.starts_with('.') && !opens.contains(&n.as_str()) { opens.push(n.as_str()); } }
        for n in std::iter::once(d.end.as_ref().unwrap()).chain(d.end_aliases.iter()) { if n.starts_with('.') && !closes.contains(&n.as_str()) { closes.push(n.as_str()); } }
    }
    (opens, closes)
}

/// Löst Name und Argumente einer Direktivenzeile auf die kanonische Form auf (`word` kleingeschrieben, mit Punkt).
/// `.page [n]` wird zu `.align n, skip` (Standard 256). Unbekannte Namen bleiben unverändert.
pub fn normalize(word: &str, args: &str) -> (String, String) {
    if word == ".page" {
        let a = args.trim();
        return (".align".to_string(), format!("{}, skip", if a.is_empty() { "256" } else { a }));
    }
    (canonical(word).unwrap_or(word).to_string(), args.to_string())
}
